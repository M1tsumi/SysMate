use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, ScrolledWindow, ListBox, ProgressBar, glib};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use sysinfo::System;

pub struct TaskManagerView {
    root: GtkBox,
}

impl TaskManagerView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.add_css_class("view");

        // Create system info wrapper
        let system = Rc::new(RefCell::new(System::new_all()));

        // Resource overview section
        let overview_box = GtkBox::new(Orientation::Vertical, 12);
        overview_box.set_margin_start(24);
        overview_box.set_margin_end(24);
        overview_box.set_margin_top(24);
        overview_box.set_margin_bottom(12);

        // CPU usage
        let cpu_group = adw::PreferencesGroup::new();
        cpu_group.set_title("CPU Usage");
        
        let cpu_bar = ProgressBar::new();
        cpu_bar.set_show_text(true);
        
        let cpu_label = Label::new(None);
        cpu_label.set_halign(gtk4::Align::Start);
        cpu_label.add_css_class("dim-label");
        
        let cpu_box = GtkBox::new(Orientation::Vertical, 6);
        cpu_box.append(&cpu_bar);
        cpu_box.append(&cpu_label);
        
        let cpu_row = adw::ActionRow::new();
        cpu_row.set_child(Some(&cpu_box));
        cpu_group.add(&cpu_row);
        
        overview_box.append(&cpu_group);

        // Memory usage
        let mem_group = adw::PreferencesGroup::new();
        mem_group.set_title("Memory");
        
        let mem_bar = ProgressBar::new();
        mem_bar.set_show_text(true);
        
        let mem_label = Label::new(None);
        mem_label.set_halign(gtk4::Align::Start);
        mem_label.add_css_class("dim-label");
        
        let mem_box = GtkBox::new(Orientation::Vertical, 6);
        mem_box.append(&mem_bar);
        mem_box.append(&mem_label);
        
        let mem_row = adw::ActionRow::new();
        mem_row.set_child(Some(&mem_box));
        mem_group.add(&mem_row);
        
        overview_box.append(&mem_group);

        root.append(&overview_box);

        // Process list section
        let process_group = adw::PreferencesGroup::new();
        process_group.set_title("Running Processes");
        process_group.set_margin_start(24);
        process_group.set_margin_end(24);
        process_group.set_margin_bottom(24);

        let process_list = ListBox::new();
        process_list.add_css_class("boxed-list");
        
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&process_list));
        scrolled.set_min_content_height(300);
        
        process_group.add(&scrolled);
        root.append(&process_group);

        // Setup periodic refresh
        let system_clone = system.clone();
        let cpu_bar_clone = cpu_bar.clone();
        let cpu_label_clone = cpu_label.clone();
        let mem_bar_clone = mem_bar.clone();
        let mem_label_clone = mem_label.clone();
        let process_list_clone = process_list.clone();

        // Initial update
        Self::update_system_info(
            &system_clone,
            &cpu_bar_clone,
            &cpu_label_clone,
            &mem_bar_clone,
            &mem_label_clone,
            &process_list_clone,
        );

        // Refresh every 2 seconds
        glib::timeout_add_local(Duration::from_secs(2), move || {
            Self::update_system_info(
                &system_clone,
                &cpu_bar_clone,
                &cpu_label_clone,
                &mem_bar_clone,
                &mem_label_clone,
                &process_list_clone,
            );
            glib::ControlFlow::Continue
        });

        Self { root }
    }

    fn update_system_info(
        system: &Rc<RefCell<System>>,
        cpu_bar: &ProgressBar,
        cpu_label: &Label,
        mem_bar: &ProgressBar,
        mem_label: &Label,
        process_list: &ListBox,
    ) {
        let mut sys = system.borrow_mut();
        sys.refresh_cpu();
        sys.refresh_memory();
        sys.refresh_processes();

        // Update CPU
        let global_cpu_usage = sys.global_cpu_info().cpu_usage();
        let cpu_count = sys.cpus().len();
        cpu_bar.set_fraction((global_cpu_usage / 100.0) as f64);
        cpu_bar.set_text(Some(&format!("{:.1}%", global_cpu_usage)));
        cpu_label.set_text(&format!("{} logical processors", cpu_count));

        // Update Memory
        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let mem_percent = (used_mem as f64 / total_mem as f64) * 100.0;
        mem_bar.set_fraction(used_mem as f64 / total_mem as f64);
        mem_bar.set_text(Some(&format!("{:.1}%", mem_percent)));
        mem_label.set_text(&format!(
            "{} / {} ({} available)",
            Self::format_bytes(used_mem),
            Self::format_bytes(total_mem),
            Self::format_bytes(sys.available_memory())
        ));

        // Update process list
        // Clear existing items
        while let Some(child) = process_list.first_child() {
            process_list.remove(&child);
        }

        // Get top processes by CPU usage
        let mut processes: Vec<_> = sys.processes().iter().collect();
        processes.sort_by(|a, b| {
            b.1.cpu_usage()
                .partial_cmp(&a.1.cpu_usage())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Show top 20 processes
        for (pid, process) in processes.iter().take(20) {
            let row = adw::ActionRow::new();
            
            let name = process.name();
            let cpu = process.cpu_usage();
            let mem = process.memory();
            
            row.set_title(name);
            row.set_subtitle(&format!(
                "PID: {} • CPU: {:.1}% • Memory: {}",
                pid,
                cpu,
                Self::format_bytes(mem)
            ));

            process_list.append(&row);
        }
    }

    fn format_bytes(bytes: u64) -> String {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;
        const KB: u64 = 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn build(&self) -> GtkBox {
        self.root.clone()
    }
}
