//! Service Manager View
//! 
//! Lists systemd services grouped by status with expandable details and management controls.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Button, SearchEntry, DropDown, MessageDialog, ButtonsType, MessageType};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ServiceManagerView {
    root: GtkBox,
}

impl ServiceManagerView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);

        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Service Manager", "Manage systemd services");
        header.set_title_widget(Some(&title));
        
        let refresh_btn = Button::with_label("Refresh");
        refresh_btn.set_icon_name("view-refresh-symbolic");
        header.pack_end(&refresh_btn);
        
        root.append(&header);

        // Filter and search bar
        let toolbar = GtkBox::new(Orientation::Horizontal, 12);
        toolbar.set_margin_top(12);
        toolbar.set_margin_bottom(12);
        toolbar.set_margin_start(12);
        toolbar.set_margin_end(12);

        let filter_label = Label::new(Some("Filter:"));
        toolbar.append(&filter_label);

        let filter_list = gtk4::StringList::new(&["All", "Active", "Failed", "Inactive"]);
        let filter_dropdown = DropDown::new(Some(filter_list), None::<gtk4::Expression>);
        toolbar.append(&filter_dropdown);

        let search_entry = SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search services..."));
        search_entry.set_hexpand(true);
        toolbar.append(&search_entry);

        root.append(&toolbar);

        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_top(12);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        let services_rc = Rc::new(RefCell::new(Vec::new()));

        let populate_services = |content: &GtkBox, services_rc: &Rc<RefCell<Vec<service_manager::ServiceInfo>>>, filter_state: Option<service_manager::ServiceState>, search_query: Option<String>| {
            // Clear existing content
            while let Some(child) = content.first_child() {
                content.remove(&child);
            }

            match service_manager::list_services_with_limit(Some(100)) {
                Ok(mut services) => {
                    *services_rc.borrow_mut() = services.clone();

                    // Apply filters
                    if let Some(state) = filter_state {
                        services = service_manager::filter_services_by_state(&services, &state);
                    }

                    if let Some(query) = search_query {
                        if !query.is_empty() {
                            services.retain(|s| s.name.to_lowercase().contains(&query.to_lowercase()));
                        }
                    }

                    if services.is_empty() {
                        let status = adw::StatusPage::new();
                        status.set_icon_name(Some("preferences-system-symbolic"));
                        status.set_title("No Services Found");
                        status.set_description(Some("No services match your filter criteria"));
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
                            
                            for service in active.iter().take(30) {
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
                            inactive_group.set_title(&format!("Inactive Services (showing 20 of {})", inactive.len()));
                            
                            for service in inactive.iter().take(20) {
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
        };

        // Initial population
        populate_services(&content, &services_rc, None, None);

        // Refresh button handler
        let content_clone = content.clone();
        let services_clone = services_rc.clone();
        refresh_btn.connect_clicked(move |_| {
            populate_services(&content_clone, &services_clone, None, None);
        });

        // Filter dropdown handler
        let content_clone = content.clone();
        let services_clone = services_rc.clone();
        filter_dropdown.connect_selected_notify(move |dropdown| {
            let filter_state = match dropdown.selected() {
                1 => Some(service_manager::ServiceState::Active),
                2 => Some(service_manager::ServiceState::Failed),
                3 => Some(service_manager::ServiceState::Inactive),
                _ => None,
            };
            populate_services(&content_clone, &services_clone, filter_state, None);
        });

        // Search handler
        let content_clone = content.clone();
        let services_clone = services_rc.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            populate_services(&content_clone, &services_clone, None, Some(query));
        });

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

        // Control buttons
        let controls_row = adw::ActionRow::new();
        controls_row.set_title("Controls");

        let controls_box = GtkBox::new(Orientation::Horizontal, 6);

        // Start button
        if service.state != service_manager::ServiceState::Active {
            let start_btn = Button::with_label("Start");
            start_btn.set_icon_name("media-playback-start-symbolic");
            start_btn.add_css_class("flat");
            start_btn.add_css_class("suggested-action");
            
            let service_name = service.name.clone();
            start_btn.connect_clicked(move |btn| {
                let service_name_clone = service_name.clone();
                
                // Show confirmation dialog
                if let Some(window) = btn.root().and_downcast::<gtk4::Window>() {
                    let dialog = MessageDialog::new(
                        Some(&window),
                        gtk4::DialogFlags::MODAL,
                        MessageType::Question,
                        ButtonsType::OkCancel,
                        &format!("Start service '{}'?", service_name)
                    );
                    dialog.set_title(Some("Start Service"));
                    
                    dialog.connect_response(move |dialog, response| {
                        if response == gtk4::ResponseType::Ok {
                            if let Err(e) = service_manager::start_service(&service_name_clone) {
                                eprintln!("Failed to start service: {}", e);
                            }
                        }
                        dialog.close();
                    });
                    
                    dialog.present();
                }
            });
            
            controls_box.append(&start_btn);
        }

        // Stop button
        if service.state == service_manager::ServiceState::Active {
            let stop_btn = Button::with_label("Stop");
            stop_btn.set_icon_name("media-playback-stop-symbolic");
            stop_btn.add_css_class("flat");
            stop_btn.add_css_class("destructive-action");
            
            let service_name = service.name.clone();
            stop_btn.connect_clicked(move |btn| {
                let service_name_clone = service_name.clone();
                
                // Show confirmation dialog
                if let Some(window) = btn.root().and_downcast::<gtk4::Window>() {
                    let dialog = MessageDialog::new(
                        Some(&window),
                        gtk4::DialogFlags::MODAL,
                        MessageType::Warning,
                        ButtonsType::OkCancel,
                        &format!("Stop service '{}'? This may affect system functionality.", service_name)
                    );
                    dialog.set_title(Some("Stop Service"));
                    
                    dialog.connect_response(move |dialog, response| {
                        if response == gtk4::ResponseType::Ok {
                            if let Err(e) = service_manager::stop_service(&service_name_clone) {
                                eprintln!("Failed to stop service: {}", e);
                            }
                        }
                        dialog.close();
                    });
                    
                    dialog.present();
                }
            });
            
            controls_box.append(&stop_btn);
        }

        // Restart button
        let restart_btn = Button::with_label("Restart");
        restart_btn.set_icon_name("view-refresh-symbolic");
        restart_btn.add_css_class("flat");
        
        let service_name = service.name.clone();
        restart_btn.connect_clicked(move |_| {
            if let Err(e) = service_manager::restart_service(&service_name) {
                eprintln!("Failed to restart service: {}", e);
            }
        });
        
        controls_box.append(&restart_btn);

        // Enable/Disable toggle
        let toggle_btn = Button::with_label(if service.enabled { "Disable" } else { "Enable" });
        toggle_btn.add_css_class("flat");
        
        let service_name = service.name.clone();
        let enabled = service.enabled;
        toggle_btn.connect_clicked(move |_| {
            let result = if enabled {
                service_manager::disable_service(&service_name)
            } else {
                service_manager::enable_service(&service_name)
            };
            
            if let Err(e) = result {
                eprintln!("Failed to toggle service: {}", e);
            }
        });
        
        controls_box.append(&toggle_btn);

        controls_row.add_suffix(&controls_box);
        expander.add_row(&controls_row);

        // Logs button
        let logs_row = adw::ActionRow::new();
        logs_row.set_property("title", "View Logs");
        
        let logs_btn = Button::with_label("Show Logs");
        logs_btn.set_icon_name("utilities-terminal-symbolic");
        logs_btn.add_css_class("flat");
        
        let service_name = service.name.clone();
        logs_btn.connect_clicked(move |btn| {
            if let Ok(logs) = service_manager::get_service_logs(&service_name, 100) {
                // Create a dialog to show logs
                let window = btn.root().and_downcast::<gtk4::Window>();
                
                let dialog = adw::Window::new();
                dialog.set_title(Some(&format!("Logs: {}", service_name)));
                dialog.set_default_size(800, 600);
                dialog.set_modal(true);
                if let Some(parent) = window {
                    dialog.set_transient_for(Some(&parent));
                }
                
                let dialog_box = GtkBox::new(Orientation::Vertical, 0);
                
                // Header
                let header = adw::HeaderBar::new();
                let title_widget = adw::WindowTitle::new(&format!("Logs: {}", service_name), "Last 100 lines");
                header.set_title_widget(Some(&title_widget));
                
                let close_btn = Button::with_label("Close");
                close_btn.add_css_class("suggested-action");
                header.pack_end(&close_btn);
                
                dialog_box.append(&header);
                
                // Logs text view
                let scrolled = gtk4::ScrolledWindow::new();
                scrolled.set_vexpand(true);
                scrolled.set_hexpand(true);
                
                let text_view = gtk4::TextView::new();
                text_view.set_editable(false);
                text_view.set_monospace(true);
                text_view.set_margin_top(12);
                text_view.set_margin_bottom(12);
                text_view.set_margin_start(12);
                text_view.set_margin_end(12);
                
                let buffer = text_view.buffer();
                buffer.set_text(&logs);
                
                scrolled.set_child(Some(&text_view));
                dialog_box.append(&scrolled);
                
                dialog.set_content(Some(&dialog_box));
                
                let dialog_clone = dialog.clone();
                close_btn.connect_clicked(move |_| {
                    dialog_clone.close();
                });
                
                dialog.present();
            } else {
                eprintln!("Failed to get logs for {}", service_name);
            }
        });
        
        logs_row.add_suffix(&logs_btn);
        expander.add_row(&logs_row);
        
        group.add(&expander);
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
