use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation};
use libadwaita as adw;

pub struct StartupManagerView {
    root: GtkBox,
}

impl StartupManagerView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);

        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Startup Manager", "");
        header.set_title_widget(Some(&title));
        root.append(&header);

        let status = adw::StatusPage::new();
        status.set_icon_name(Some("system-run-symbolic"));
        status.set_title("Startup Manager");
        status.set_description(Some("Coming soon: Manage autostart applications"));
        status.set_vexpand(true);

        root.append(&status);

        Self { root }
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
