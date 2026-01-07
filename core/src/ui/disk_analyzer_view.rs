//! Disk Analyzer View
//! 
//! Shows disk usage for all mounted filesystems with visual progress indicators and folder analysis.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;

pub struct DiskAnalyzerView {
    root: GtkBox,
}

impl DiskAnalyzerView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);

        // Content area
        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Get disk information
        match disk_analyzer::get_mount_points() {
            Ok(mounts) => {
                let is_empty = mounts.is_empty();
                
                for mount in mounts {
                    let group = adw::PreferencesGroup::new();
                    group.set_title(&mount.mount_point.display().to_string());
                    group.set_description(Some(&format!("{} ({})", mount.device, mount.fs_type)));

                    // Usage row with progress bar
                    let usage_row = adw::ActionRow::new();
                    usage_row.set_title("Disk Usage");

                    let usage_box = GtkBox::new(Orientation::Vertical, 6);
                    usage_box.set_hexpand(true);

                    let progress = ProgressBar::new();
                    progress.set_fraction(mount.used_percentage() / 100.0);
                    progress.set_show_text(true);
                    progress.set_text(Some(&format!("{:.1}%", mount.used_percentage())));
                    
                    // Color code the progress bar based on usage
                    if mount.used_percentage() > 90.0 {
                        progress.add_css_class("error");
                    } else if mount.used_percentage() > 75.0 {
                        progress.add_css_class("warning");
                    }
                    
                    usage_box.append(&progress);

                    let info_label = Label::new(Some(&format!(
                        "{} used of {} ({} available)",
                        disk_analyzer::MountPoint::format_size(mount.used),
                        disk_analyzer::MountPoint::format_size(mount.total),
                        disk_analyzer::MountPoint::format_size(mount.available)
                    )));
                    info_label.add_css_class("dim-label");
                    info_label.set_xalign(0.0);
                    usage_box.append(&info_label);

                    usage_row.add_suffix(&usage_box);
                    group.add(&usage_row);

                    content.append(&group);
                }

                if is_empty {
                    let status = adw::StatusPage::new();
                    status.set_icon_name(Some("drive-harddisk-symbolic"));
                    status.set_title("No Disks Found");
                    status.set_description(Some("No mounted filesystems detected"));
                    content.append(&status);
                }
            }
            Err(e) => {
                let status = adw::StatusPage::new();
                status.set_icon_name(Some("dialog-error-symbolic"));
                status.set_title("Error Loading Disk Information");
                status.set_description(Some(&e.to_string()));
                content.append(&status);
            }
        }

        // Large folders section
        let large_folders_group = adw::PreferencesGroup::new();
        large_folders_group.set_title("Large Folders in Home Directory");
        large_folders_group.set_description(Some("Folders taking up significant disk space"));

        match disk_analyzer::get_common_large_folders() {
            Ok(folders) => {
                for folder in folders.iter().take(10) {
                    let row = adw::ActionRow::new();
                    
                    if let Some(name) = folder.path.file_name() {
                        row.set_title(&name.to_string_lossy());
                    } else {
                        row.set_title(&folder.path.display().to_string());
                    }
                    
                    row.set_subtitle(&format!(
                        "{} ({} files, {} folders)",
                        folder.format_size(),
                        folder.file_count,
                        folder.dir_count
                    ));
                    
                    let icon = gtk4::Image::from_icon_name("folder-symbolic");
                    row.add_prefix(&icon);
                    
                    let size_label = Label::new(Some(&folder.format_size()));
                    size_label.add_css_class("title-3");
                    row.add_suffix(&size_label);
                    
                    large_folders_group.add(&row);
                }
            }
            Err(e) => {
                let error_row = adw::ActionRow::new();
                error_row.set_title("Error scanning folders");
                error_row.set_subtitle(&e.to_string());
                large_folders_group.add(&error_row);
            }
        }

        content.append(&large_folders_group);

        // Cleanup suggestions
        let suggestions_group = adw::PreferencesGroup::new();
        suggestions_group.set_title("Cleanup Suggestions");
        suggestions_group.set_description(Some("Common ways to free up disk space"));

        for suggestion in disk_analyzer::get_cleanup_suggestions() {
            let row = adw::ActionRow::new();
            row.set_title(&suggestion);
            
            let icon = gtk4::Image::from_icon_name("user-trash-symbolic");
            row.add_prefix(&icon);
            
            suggestions_group.add(&row);
        }

        content.append(&suggestions_group);

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&content));

        root.append(&scrolled);

        Self { root }
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
