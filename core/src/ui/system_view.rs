//! System Information View
//! 
//! Displays OS info, hardware specs, and memory/swap usage with expandable details.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;

use crate::system_info::SystemInfo;

pub struct SystemView {
    root: GtkBox,
}

impl SystemView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);

        // Header
        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("System Information", "");
        header.set_title_widget(Some(&title));
        
        root.append(&header);

        // Content area
        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_vexpand(true);

        // Get system info
        let sys_info = SystemInfo::new();

        // System details group
        let system_group = adw::PreferencesGroup::new();
        system_group.set_title("System Details");
        
        Self::add_info_row(&system_group, "Operating System", &sys_info.os_version());
        Self::add_info_row(&system_group, "Hostname", &sys_info.hostname());
        Self::add_info_row(&system_group, "Kernel Version", &sys_info.kernel_version());
        Self::add_info_row(&system_group, "Uptime", &sys_info.format_uptime());

        content.append(&system_group);

        // Hardware group
        let hardware_group = adw::PreferencesGroup::new();
        hardware_group.set_title("Hardware");
        
        Self::add_info_row(&hardware_group, "CPU Cores", &sys_info.cpu_count().to_string());
        
        // Expandable Memory Row
        let memory_expander = adw::ExpanderRow::new();
        memory_expander.set_title("Memory");
        
        let total_mem = sys_info.total_memory();
        let used_mem = sys_info.used_memory();
        let available_mem = sys_info.available_memory();
        let free_mem = sys_info.free_memory();
        let used_percentage = (used_mem as f64 / total_mem as f64) * 100.0;
        
        memory_expander.set_subtitle(&format!(
            "{} / {} ({:.1}% used)",
            SystemInfo::format_memory(used_mem),
            SystemInfo::format_memory(total_mem),
            used_percentage
        ));
        
        // Progress bar for memory
        let mem_progress_box = GtkBox::new(Orientation::Vertical, 6);
        mem_progress_box.set_margin_top(12);
        mem_progress_box.set_margin_bottom(12);
        mem_progress_box.set_margin_start(12);
        mem_progress_box.set_margin_end(12);
        
        let mem_progress = ProgressBar::new();
        mem_progress.set_fraction(used_percentage / 100.0);
        mem_progress.set_show_text(true);
        mem_progress.set_text(Some(&format!("{:.1}%", used_percentage)));
        mem_progress_box.append(&mem_progress);
        
        memory_expander.add_row(&mem_progress_box);
        
        // Memory breakdown rows
        Self::add_expander_detail(&memory_expander, "Total Memory", &SystemInfo::format_memory(total_mem));
        Self::add_expander_detail(&memory_expander, "Used Memory", &SystemInfo::format_memory(used_mem));
        Self::add_expander_detail(&memory_expander, "Available Memory", &SystemInfo::format_memory(available_mem));
        Self::add_expander_detail(&memory_expander, "Free Memory", &SystemInfo::format_memory(free_mem));
        Self::add_expander_detail(&memory_expander, "Buffers/Cache", 
            &SystemInfo::format_memory(used_mem.saturating_sub(total_mem - available_mem)));
        
        hardware_group.add(&memory_expander);
        
        // Expandable Swap Row if swap exists
        let total_swap = sys_info.total_swap();
        if total_swap > 0 {
            let swap_expander = adw::ExpanderRow::new();
            swap_expander.set_title("Swap Memory");
            
            let used_swap = sys_info.used_swap();
            let swap_percentage = (used_swap as f64 / total_swap as f64) * 100.0;
            
            swap_expander.set_subtitle(&format!(
                "{} / {} ({:.1}% used)",
                SystemInfo::format_memory(used_swap),
                SystemInfo::format_memory(total_swap),
                swap_percentage
            ));
            
            // Progress bar for swap
            let swap_progress_box = GtkBox::new(Orientation::Vertical, 6);
            swap_progress_box.set_margin_top(12);
            swap_progress_box.set_margin_bottom(12);
            swap_progress_box.set_margin_start(12);
            swap_progress_box.set_margin_end(12);
            
            let swap_progress = ProgressBar::new();
            swap_progress.set_fraction(swap_percentage / 100.0);
            swap_progress.set_show_text(true);
            swap_progress.set_text(Some(&format!("{:.1}%", swap_percentage)));
            swap_progress_box.append(&swap_progress);
            
            swap_expander.add_row(&swap_progress_box);
            
            Self::add_expander_detail(&swap_expander, "Total Swap", &SystemInfo::format_memory(total_swap));
            Self::add_expander_detail(&swap_expander, "Used Swap", &SystemInfo::format_memory(used_swap));
            Self::add_expander_detail(&swap_expander, "Free Swap", &SystemInfo::format_memory(total_swap - used_swap));
            
            hardware_group.add(&swap_expander);
        }

        content.append(&hardware_group);
        
        // Temperature group
        let temperatures = SystemInfo::get_temperatures();
        if !temperatures.is_empty() {
            let temp_group = adw::PreferencesGroup::new();
            temp_group.set_title("System Temperatures");
            
            for sensor in temperatures.iter().take(10) {
                let temp_expander = adw::ExpanderRow::new();
                temp_expander.set_title(&sensor.name);
                
                let status = SystemInfo::temperature_status(sensor.temperature);
                let temp_display = format!("{} ({})", 
                    SystemInfo::format_temperature(sensor.temperature),
                    status
                );
                temp_expander.set_subtitle(&temp_display);
                
                // Add icon based on temperature
                let icon_name = if sensor.temperature < 50.0 {
                    "temperature-cold-symbolic"
                } else if sensor.temperature < 70.0 {
                    "temperature-warm-symbolic"
                } else {
                    "temperature-hot-symbolic"
                };
                
                let temp_icon = gtk4::Image::from_icon_name(icon_name);
                temp_expander.add_prefix(&temp_icon);
                
                // Add detailed info
                Self::add_expander_detail(&temp_expander, "Temperature", 
                    &SystemInfo::format_temperature(sensor.temperature));
                Self::add_expander_detail(&temp_expander, "Status", status);
                Self::add_expander_detail(&temp_expander, "Sensor Label", &sensor.label);
                
                temp_group.add(&temp_expander);
            }
            
            content.append(&temp_group);
        }

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&content));

        root.append(&scrolled);

        Self { root }
    }

    fn add_info_row(group: &adw::PreferencesGroup, title: &str, value: &str) {
        let row = adw::ActionRow::new();
        row.set_title(title);
        
        let value_label = Label::new(Some(value));
        value_label.add_css_class("dim-label");
        row.add_suffix(&value_label);
        
        group.add(&row);
    }
    
    fn add_expander_detail(expander: &adw::ExpanderRow, title: &str, value: &str) {
        let row = adw::ActionRow::new();
        row.set_title(title);
        
        let value_label = Label::new(Some(value));
        value_label.add_css_class("dim-label");
        row.add_suffix(&value_label);
        
        expander.add_row(&row);
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
