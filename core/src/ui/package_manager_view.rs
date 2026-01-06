//! Package Manager View
//! 
//! Shows package statistics and recent activity for APT, Snap, and Flatpak.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;

pub struct PackageManagerView {
    root: GtkBox,
}

impl PackageManagerView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);

        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Package Manager", "");
        header.set_title_widget(Some(&title));
        root.append(&header);

        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        match package_manager::get_package_stats() {
            Ok(stats) => {
                // Overview group
                let overview_group = adw::PreferencesGroup::new();
                overview_group.set_title("Package Overview");
                
                Self::add_stat_row(&overview_group, "Total Installed Packages", 
                    &stats.total_installed.to_string(), "package-x-generic-symbolic");
                
                if stats.upgradeable > 0 {
                    Self::add_stat_row(&overview_group, "Upgradeable Packages", 
                        &stats.upgradeable.to_string(), "software-update-available-symbolic");
                }
                
                if stats.auto_removable > 0 {
                    Self::add_stat_row(&overview_group, "Auto-removable Packages", 
                        &stats.auto_removable.to_string(), "user-trash-symbolic");
                }
                
                content.append(&overview_group);
                
                // Package managers
                let managers_group = adw::PreferencesGroup::new();
                managers_group.set_title("Package Managers");
                
                Self::add_stat_row(&managers_group, "APT (Debian/Ubuntu)", 
                    &format!("{} packages", stats.total_installed), "package-x-generic-symbolic");
                
                let snap_count = package_manager::get_snap_count();
                if snap_count > 0 {
                    Self::add_stat_row(&managers_group, "Snap", 
                        &format!("{} packages", snap_count), "package-x-generic-symbolic");
                }
                
                let flatpak_count = package_manager::get_flatpak_count();
                if flatpak_count > 0 {
                    Self::add_stat_row(&managers_group, "Flatpak", 
                        &format!("{} packages", flatpak_count), "package-x-generic-symbolic");
                }
                
                content.append(&managers_group);
                
                // Recent activity
                if let Ok(recent) = package_manager::list_recent_packages(10) {
                    if !recent.is_empty() {
                        let recent_group = adw::PreferencesGroup::new();
                        recent_group.set_title("Recent Activity");
                        recent_group.set_description(Some("Recently installed or upgraded packages"));
                        
                        for pkg in recent {
                            let row = adw::ActionRow::new();
                            row.set_title(&pkg.name);
                            row.set_subtitle(&pkg.version);
                            
                            let icon = gtk4::Image::from_icon_name("package-x-generic-symbolic");
                            row.add_prefix(&icon);
                            
                            recent_group.add(&row);
                        }
                        
                        content.append(&recent_group);
                    }
                }
            }
            Err(e) => {
                let status = adw::StatusPage::new();
                status.set_icon_name(Some("dialog-error-symbolic"));
                status.set_title("Error Loading Package Information");
                status.set_description(Some(&e.to_string()));
                content.append(&status);
            }
        }

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&content));

        root.append(&scrolled);

        Self { root }
    }
    
    fn add_stat_row(group: &adw::PreferencesGroup, title: &str, value: &str, icon_name: &str) {
        let row = adw::ActionRow::new();
        row.set_title(title);
        
        let icon = gtk4::Image::from_icon_name(icon_name);
        row.add_prefix(&icon);
        
        let value_label = Label::new(Some(value));
        value_label.add_css_class("title-2");
        row.add_suffix(&value_label);
        
        group.add(&row);
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
