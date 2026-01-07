//! Package Manager View
//! 
//! Shows package statistics and recent activity for APT, Snap, and Flatpak.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Button, SearchEntry};
use libadwaita as adw;
use adw::prelude::*;

pub struct PackageManagerView {
    root: GtkBox,
}

impl PackageManagerView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);

        // Action buttons bar
        let action_bar = GtkBox::new(Orientation::Horizontal, 12);
        action_bar.set_margin_top(12);
        action_bar.set_margin_bottom(12);
        action_bar.set_margin_start(12);
        action_bar.set_margin_end(12);

        let upgrade_btn = Button::with_label("Upgrade All");
        upgrade_btn.set_icon_name("software-update-available-symbolic");
        upgrade_btn.add_css_class("suggested-action");
        action_bar.append(&upgrade_btn);

        let autoremove_btn = Button::with_label("Auto Remove");
        autoremove_btn.set_icon_name("user-trash-symbolic");
        action_bar.append(&autoremove_btn);

        let search_entry = SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search packages..."));
        search_entry.set_hexpand(true);
        action_bar.append(&search_entry);

        root.append(&action_bar);

        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_top(12);
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

                // Upgradeable packages
                if stats.upgradeable > 0 {
                    if let Ok(upgradeable) = package_manager::list_upgradeable_packages() {
                        let upgrade_group = adw::PreferencesGroup::new();
                        upgrade_group.set_title("Upgradeable Packages");
                        upgrade_group.set_description(Some("Packages with available updates"));
                        
                        for pkg in upgradeable.iter().take(20) {
                            let row = adw::ActionRow::new();
                            row.set_title(&pkg.name);
                            row.set_subtitle(&format!("Current: {}", pkg.version));
                            
                            let icon = gtk4::Image::from_icon_name("software-update-available-symbolic");
                            row.add_prefix(&icon);
                            
                            let upgrade_pkg_btn = Button::with_label("Upgrade");
                            upgrade_pkg_btn.set_valign(gtk4::Align::Center);
                            upgrade_pkg_btn.add_css_class("flat");
                            
                            let pkg_name = pkg.name.clone();
                            upgrade_pkg_btn.connect_clicked(move |_| {
                                if let Err(e) = package_manager::install_package(&pkg_name) {
                                    eprintln!("Failed to upgrade package: {}", e);
                                }
                            });
                            
                            row.add_suffix(&upgrade_pkg_btn);
                            upgrade_group.add(&row);
                        }
                        
                        content.append(&upgrade_group);
                    }
                }
                
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
                            
                            let remove_btn = Button::with_label("Remove");
                            remove_btn.set_valign(gtk4::Align::Center);
                            remove_btn.add_css_class("flat");
                            remove_btn.add_css_class("destructive-action");
                            
                            let pkg_name = pkg.name.clone();
                            remove_btn.connect_clicked(move |_| {
                                if let Err(e) = package_manager::remove_package(&pkg_name) {
                                    eprintln!("Failed to remove package: {}", e);
                                }
                            });
                            
                            row.add_suffix(&remove_btn);
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

        // Button handlers
        upgrade_btn.connect_clicked(|_| {
            gtk4::glib::MainContext::default().spawn_local(async {
                if let Err(e) = package_manager::upgrade_packages() {
                    eprintln!("Failed to upgrade packages: {}", e);
                }
            });
        });

        autoremove_btn.connect_clicked(|_| {
            gtk4::glib::MainContext::default().spawn_local(async {
                if let Err(e) = package_manager::autoremove_packages() {
                    eprintln!("Failed to autoremove packages: {}", e);
                }
            });
        });

        // Search handler
        let content_clone = content.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text();
            if query.len() < 3 {
                return;
            }
            
            if let Ok(results) = package_manager::search_packages(&query) {
                // Clear previous search results
                while let Some(child) = content_clone.last_child() {
                    if let Some(group) = child.downcast_ref::<adw::PreferencesGroup>() {
                        if group.title() == "Search Results" {
                            content_clone.remove(&child);
                            break;
                        }
                    }
                    break;
                }
                
                let search_group = adw::PreferencesGroup::new();
                search_group.set_title("Search Results");
                
                for pkg in results.iter().take(20) {
                    let row = adw::ActionRow::new();
                    row.set_title(&pkg.name);
                    row.set_subtitle(&pkg.description);
                    
                    let icon = gtk4::Image::from_icon_name("system-search-symbolic");
                    row.add_prefix(&icon);
                    
                    let install_btn = Button::with_label("Install");
                    install_btn.set_valign(gtk4::Align::Center);
                    install_btn.add_css_class("flat");
                    
                    let pkg_name = pkg.name.clone();
                    install_btn.connect_clicked(move |_| {
                        if let Err(e) = package_manager::install_package(&pkg_name) {
                            eprintln!("Failed to install package: {}", e);
                        }
                    });
                    
                    row.add_suffix(&install_btn);
                    search_group.add(&row);
                }
                
                content_clone.append(&search_group);
            }
        });

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
