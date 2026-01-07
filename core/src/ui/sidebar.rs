use gtk4::prelude::*;
use gtk4::{Box as GtkBox, ListBox, Orientation, SelectionMode, Stack};
use libadwaita as adw;
use adw::prelude::*;

pub struct Sidebar {
    root: GtkBox,
}

impl Sidebar {
    pub fn new_with_stack(stack: &Stack) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.set_width_request(250);
        root.add_css_class("sidebar");

        // Header
        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Modules", "");
        header.set_title_widget(Some(&title));
        root.append(&header);

        // Module list
        let list_box = ListBox::new();
        list_box.set_selection_mode(SelectionMode::Single);
        list_box.add_css_class("navigation-sidebar");

        // Add modules with their stack page names
        let modules = vec![
            ("System Info", "computer-symbolic", "system"),
            ("Disk Analyzer", "drive-harddisk-symbolic", "disk"),
            ("Package Manager", "package-x-generic-symbolic", "packages"),
            ("Service Manager", "preferences-system-symbolic", "services"),
            ("Startup Manager", "system-run-symbolic", "startup"),
            ("System Cleaner", "edit-clear-all-symbolic", "cleaner"),
        ];

        for (name, icon, _page_name) in modules {
            let row = adw::ActionRow::new();
            row.set_title(name);
            
            let icon_widget = gtk4::Image::from_icon_name(icon);
            row.add_prefix(&icon_widget);

            list_box.append(&row);
        }

        // Connect row selection to stack switching
        let stack_clone = stack.clone();
        list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let index = row.index() as usize;
                let page_names = ["system", "disk", "packages", "services", "startup", "cleaner"];
                if let Some(&page_name) = page_names.get(index) {
                    stack_clone.set_visible_child_name(page_name);
                }
            }
        });

        // Select first row by default
        if let Some(row) = list_box.row_at_index(0) {
            list_box.select_row(Some(&row));
        }

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&list_box));
        
        root.append(&scrolled);

        Self { root }
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
