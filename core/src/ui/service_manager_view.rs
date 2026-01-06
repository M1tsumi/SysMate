//! Service Manager View
//! 
//! Lists systemd services grouped by status with expandable details.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;

pub struct ServiceManagerView {
    root: GtkBox,
}

impl ServiceManagerView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);

        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Service Manager", "");
        header.set_title_widget(Some(&title));
        root.append(&header);

        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        match service_manager::list_services() {
            Ok(services) => {
                if services.is_empty() {
                    let status = adw::StatusPage::new();
                    status.set_icon_name(Some("preferences-system-symbolic"));
                    status.set_title("No Services Found");
                    status.set_description(Some("Unable to list systemd services"));
                    content.append(&status);
                } else {
                    // Group services by state
                    let active: Vec<_> = services.iter().filter(|s| s.state == service_manager::ServiceState::Active).collect();
                    let failed: Vec<_> = services.iter().filter(|s| s.state == service_manager::ServiceState::Failed).collect();
                    let inactive: Vec<_> = services.iter().filter(|s| s.state == service_manager::ServiceState::Inactive).collect();
                    
                    // Active services
                    if !active.is_empty() {
                        let active_group = adw::PreferencesGroup::new();
                        active_group.set_title(&format!("Active Services ({})", active.len()));
                        
                        for service in active.iter().take(15) {
                            Self::add_service_row(&active_group, service);
                        }
                        
                        content.append(&active_group);
                    }
                    
                    // Failed services
                    if !failed.is_empty() {
                        let failed_group = adw::PreferencesGroup::new();
                        failed_group.set_title(&format!("Failed Services ({})", failed.len()));
                        
                        for service in failed.iter() {
                            Self::add_service_row(&failed_group, service);
                        }
                        
                        content.append(&failed_group);
                    }
                    
                    // Inactive services (limited)
                    if !inactive.is_empty() {
                        let inactive_group = adw::PreferencesGroup::new();
                        inactive_group.set_title(&format!("Inactive Services (showing 10 of {})", inactive.len()));
                        
                        for service in inactive.iter().take(10) {
                            Self::add_service_row(&inactive_group, service);
                        }
                        
                        content.append(&inactive_group);
                    }
                }
            }
            Err(e) => {
                let status = adw::StatusPage::new();
                status.set_icon_name(Some("dialog-error-symbolic"));
                status.set_title("Error Loading Services");
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
    
    fn add_service_row(group: &adw::PreferencesGroup, service: &service_manager::ServiceInfo) {
        let expander = adw::ExpanderRow::new();
        expander.set_title(&service.name);
        
        if !service.description.is_empty() {
            expander.set_subtitle(&service.description);
        }
        
        // Status icon
        let status_icon = gtk4::Image::from_icon_name(service.state.icon());
        expander.add_prefix(&status_icon);
        
        // Details in expander
        let enabled_row = adw::ActionRow::new();
        enabled_row.set_title("Enabled");
        let enabled_label = Label::new(Some(if service.enabled { "Yes" } else { "No" }));
        enabled_label.add_css_class("dim-label");
        enabled_row.add_suffix(&enabled_label);
        expander.add_row(&enabled_row);
        
        let state_row = adw::ActionRow::new();
        state_row.set_title("Status");
        let state_detail_label = Label::new(Some(service.state.as_str()));
        state_detail_label.add_css_class("dim-label");
        state_row.add_suffix(&state_detail_label);
        expander.add_row(&state_row);
        
        group.add(&expander);
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
