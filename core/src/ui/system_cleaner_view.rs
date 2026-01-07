use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation};
use libadwaita as adw;

pub struct SystemCleanerView {
    root: GtkBox,
}

impl SystemCleanerView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);

        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("System Cleaner", "");
        header.set_title_widget(Some(&title));
        root.append(&header);

        let status = adw::StatusPage::new();
        status.set_icon_name(Some("edit-clear-all-symbolic"));
        status.set_title("System Cleaner");
        status.set_description(Some("Coming soon: Clean up cache, old kernels, and more"));
        status.set_vexpand(true);

        root.append(&status);

        Self { root }
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
