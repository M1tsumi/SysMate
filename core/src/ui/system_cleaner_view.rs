use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, Button, ScrolledWindow, ListBox, CheckButton, MessageDialog, ButtonsType, MessageType};
use libadwaita as adw;
use adw::prelude::*;
use system_cleaner::{scan_cleanable_items, clean_category, format_size, CleanupItem};
use std::cell::RefCell;
use std::rc::Rc;

pub struct SystemCleanerView {
    root: GtkBox,
}

impl SystemCleanerView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.add_css_class("system-cleaner-view");

        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("System Cleaner", "Free up disk space");
        header.set_title_widget(Some(&title));
        
        let scan_btn = Button::with_label("Scan");
        scan_btn.set_icon_name("system-search-symbolic");
        header.pack_end(&scan_btn);
        
        root.append(&header);

        // Warning label
        let warning_label = Label::new(Some("Cleaning system files requires administrator privileges for some operations"));
        warning_label.set_margin_top(12);
        warning_label.set_margin_bottom(12);
        warning_label.set_margin_start(12);
        warning_label.set_margin_end(12);
        warning_label.add_css_class("dim-label");
        root.append(&warning_label);

        // Info box with total size
        let info_box = GtkBox::new(Orientation::Horizontal, 12);
        info_box.set_margin_top(12);
        info_box.set_margin_bottom(12);
        info_box.set_margin_start(12);
        info_box.set_margin_end(12);

        let total_label = Label::new(Some("Total: 0 bytes (0 files)"));
        total_label.add_css_class("title-2");
        info_box.append(&total_label);

        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        info_box.append(&spacer);

        let clean_button = Button::with_label("Clean Selected");
        clean_button.set_icon_name("edit-clear-all-symbolic");
        clean_button.add_css_class("suggested-action");
        clean_button.set_sensitive(false);
        info_box.append(&clean_button);

        root.append(&info_box);

        // Scrolled window for list
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);
        scrolled.set_min_content_height(300);

        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::None);
        list_box.add_css_class("boxed-list");
        scrolled.set_child(Some(&list_box));

        root.append(&scrolled);

        let items = Rc::new(RefCell::new(Vec::new()));
        
        // Scan button handler
        let list_box_clone = list_box.clone();
        let items_clone = items.clone();
        let total_label_clone = total_label.clone();
        let clean_button_clone = clean_button.clone();
        scan_btn.connect_clicked(move |_| {
            Self::scan_items(&list_box_clone, &items_clone, &total_label_clone);
            clean_button_clone.set_sensitive(true);
        });

        // Clean button handler
        let list_box_clone = list_box.clone();
        let items_clone = items.clone();
        let total_label_clone = total_label.clone();
        clean_button.connect_clicked(move |btn| {
            let cleaned_count = Self::clean_selected(&list_box_clone, &items_clone);
            
            // Show result dialog
            if cleaned_count > 0 {
                if let Some(window) = btn.root().and_downcast::<gtk4::Window>() {
                    let dialog = MessageDialog::new(
                        Some(&window),
                        gtk4::DialogFlags::MODAL,
                        MessageType::Info,
                        ButtonsType::Ok,
                        &format!("Successfully cleaned {} categories", cleaned_count)
                    );
                    dialog.set_title(Some("Cleanup Complete"));
                    dialog.connect_response(|dialog, _| {
                        dialog.close();
                    });
                    dialog.present();
                }
            }
            
            // Rescan
            Self::scan_items(&list_box_clone, &items_clone, &total_label_clone);
            btn.set_sensitive(true);
        });

        // Initial scan
        Self::scan_items(&list_box, &items, &total_label);
        clean_button.set_sensitive(true);

        Self { root }
    }

    fn scan_items(list_box: &ListBox, items: &Rc<RefCell<Vec<CleanupItem>>>, total_label: &Label) {
        // Clear existing items
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        // Show scanning message
        let scanning_row = adw::ActionRow::new();
        scanning_row.set_property("title", "Scanning system...");
        list_box.append(&scanning_row);

        // Scan for cleanable items
        match scan_cleanable_items() {
            Ok(scanned_items) => {
                *items.borrow_mut() = scanned_items.clone();
                
                // Clear scanning message
                while let Some(child) = list_box.first_child() {
                    list_box.remove(&child);
                }
                
                if scanned_items.is_empty() {
                    let empty_row = adw::ActionRow::new();
                    empty_row.set_property("title", "No cleanable items found");
                    list_box.append(&empty_row);
                } else {
                    let mut total_size = 0u64;
                    let mut total_count = 0usize;
                    
                    for item in &scanned_items {
                        total_size += item.size;
                        total_count += item.count;
                        
                        let (row, _check) = Self::create_cleanup_row(item.clone());
                        list_box.append(&row);
                    }
                    
                    total_label.set_text(&format!(
                        "Total: {} ({} files)",
                        format_size(total_size),
                        total_count
                    ));
                }
            }
            Err(e) => {
                while let Some(child) = list_box.first_child() {
                    list_box.remove(&child);
                }
                
                let error_row = adw::ActionRow::new();
                error_row.set_property("title", &format!("Error: {}", e));
                list_box.append(&error_row);
            }
        }
    }

    fn create_cleanup_row(item: CleanupItem) -> (adw::ActionRow, CheckButton) {
        let row = adw::ActionRow::new();
        row.set_property("title", item.category.name());
        row.set_property("subtitle", &format!(
            "{}\nSize: {} | Files: {}",
            item.category.description(),
            format_size(item.size),
            item.count
        ));

        // Checkbox for selection
        let check = CheckButton::new();
        check.set_active(true);
        check.set_valign(gtk4::Align::Center);
        
        row.add_prefix(&check);

        (row, check)
    }

    fn clean_selected(list_box: &ListBox, items: &Rc<RefCell<Vec<CleanupItem>>>) -> usize {
        let items_borrowed = items.borrow();
        
        // Collect items to clean
        let mut to_clean = Vec::new();
        let mut child = list_box.first_child();
        let mut index = 0;

        while let Some(row_widget) = child {
            if index < items_borrowed.len() {
                if let Some(row) = row_widget.downcast_ref::<adw::ActionRow>() {
                    // Find checkbox by iterating through all children
                    let mut current_child = row.first_child();
                    while let Some(widget) = current_child {
                        if let Some(check) = widget.downcast_ref::<CheckButton>() {
                            if check.is_active() {
                                to_clean.push(items_borrowed[index].clone());
                            }
                            break;
                        }
                        current_child = widget.next_sibling();
                    }
                }
                index += 1;
            }
            child = row_widget.next_sibling();
        }
        
        drop(items_borrowed);
        
        // Clean selected items
        let mut cleaned_count = 0;
        for item in to_clean {
            if let Err(e) = clean_category(&item.category) {
                eprintln!("Failed to clean {}: {}", item.category.name(), e);
            } else {
                println!("Successfully cleaned: {}", item.category.name());
                cleaned_count += 1;
            }
        }
        
        cleaned_count
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
