use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, Button, ScrolledWindow, ListBox};
use libadwaita as adw;
use adw::prelude::*;
use startup_manager::{list_autostart_apps, enable_autostart, disable_autostart, remove_autostart, AutostartApp};
use std::cell::RefCell;
use std::rc::Rc;

pub struct StartupManagerView {
    root: GtkBox,
}

impl StartupManagerView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.add_css_class("startup-manager-view");

        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Startup Manager", "Manage autostart applications");
        header.set_title_widget(Some(&title));
        
        let refresh_btn = Button::with_label("Refresh");
        refresh_btn.set_icon_name("view-refresh-symbolic");
        header.pack_end(&refresh_btn);
        
        root.append(&header);

        // Info label
        let info_label = Label::new(Some("Autostart applications run when you log in to your desktop session"));
        info_label.set_margin_top(12);
        info_label.set_margin_bottom(12);
        info_label.set_margin_start(12);
        info_label.set_margin_end(12);
        info_label.add_css_class("dim-label");
        root.append(&info_label);

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

        let apps = Rc::new(RefCell::new(Vec::new()));
        
        // Refresh button handler
        let list_box_clone = list_box.clone();
        let apps_clone = apps.clone();
        refresh_btn.connect_clicked(move |_| {
            Self::populate_list(&list_box_clone, &apps_clone);
        });

        // Initial population
        Self::populate_list(&list_box, &apps);

        Self {
            root,
        }
    }

    fn populate_list(list_box: &ListBox, apps: &Rc<RefCell<Vec<AutostartApp>>>) {
        // Clear existing items
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        // Load autostart apps
        match list_autostart_apps() {
            Ok(app_list) => {
                *apps.borrow_mut() = app_list.clone();
                
                if app_list.is_empty() {
                    let empty_row = adw::ActionRow::new();
                    empty_row.set_property("title", "No autostart applications found");
                    list_box.append(&empty_row);
                } else {
                    for app in app_list {
                        let row = Self::create_app_row(app, list_box, apps);
                        list_box.append(&row);
                    }
                }
            }
            Err(e) => {
                let error_row = adw::ActionRow::new();
                error_row.set_property("title", &format!("Error: {}", e));
                list_box.append(&error_row);
            }
        }
    }

    fn create_app_row(app: AutostartApp, list_box: &ListBox, apps: &Rc<RefCell<Vec<AutostartApp>>>) -> adw::ActionRow {
        let row = adw::ActionRow::new();
        row.set_property("title", &app.name);
        
        let subtitle = if !app.comment.is_empty() {
            format!("{}\nCommand: {}", app.comment, app.exec)
        } else {
            format!("Command: {}", app.exec)
        };
        row.set_property("subtitle", &subtitle);

        // Enable/Disable switch
        let switch = gtk4::Switch::new();
        switch.set_active(app.enabled);
        switch.set_valign(gtk4::Align::Center);
        
        let app_clone = app.clone();
        let list_box_clone = list_box.clone();
        let apps_clone = apps.clone();
        switch.connect_state_set(move |_, state| {
            let result = if state {
                enable_autostart(&app_clone)
            } else {
                disable_autostart(&app_clone)
            };
            
            if let Err(e) = result {
                eprintln!("Failed to toggle autostart: {}", e);
            } else {
                // Refresh the list
                Self::populate_list(&list_box_clone, &apps_clone);
            }
            
            gtk4::glib::Propagation::Proceed
        });

        row.add_suffix(&switch);

        // Remove button (only for user directory apps)
        if app.path.to_str().unwrap_or("").contains(".config/autostart") {
            let remove_btn = Button::new();
            remove_btn.set_icon_name("user-trash-symbolic");
            remove_btn.set_valign(gtk4::Align::Center);
            remove_btn.add_css_class("flat");
            remove_btn.set_tooltip_text(Some("Remove"));
            
            let app_clone = app.clone();
            let list_box_clone = list_box.clone();
            let apps_clone = apps.clone();
            remove_btn.connect_clicked(move |_| {
                if let Err(e) = remove_autostart(&app_clone) {
                    eprintln!("Failed to remove autostart: {}", e);
                } else {
                    Self::populate_list(&list_box_clone, &apps_clone);
                }
            });
            
            row.add_suffix(&remove_btn);
        }

        row
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
