use gtk4::{Stack, StackTransitionType};
use libadwaita as adw;

use super::{Sidebar, SystemView, DiskAnalyzerView, PackageManagerView, 
            ServiceManagerView, StartupManagerView, SystemCleanerView};

pub struct MainWindow {
    root: adw::Flap,
}

impl MainWindow {
    pub fn new() -> Self {
        // Create stack for switching views
        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::Crossfade);
        stack.set_transition_duration(200);

        // Add all views to stack
        let system_view = SystemView::new();
        stack.add_titled(&system_view.build(), Some("system"), "System Info");

        let disk_view = DiskAnalyzerView::new();
        stack.add_titled(&disk_view.build(), Some("disk"), "Disk Analyzer");

        let package_view = PackageManagerView::new();
        stack.add_titled(&package_view.build(), Some("packages"), "Package Manager");

        let service_view = ServiceManagerView::new();
        stack.add_titled(&service_view.build(), Some("services"), "Service Manager");

        let startup_view = StartupManagerView::new();
        stack.add_titled(&startup_view.build(), Some("startup"), "Startup Manager");

        let cleaner_view = SystemCleanerView::new();
        stack.add_titled(&cleaner_view.build(), Some("cleaner"), "System Cleaner");

        // Create sidebar with stack reference
        let sidebar = Sidebar::new_with_stack(&stack);

        // Create flap (collapsible sidebar)
        let flap = adw::Flap::new();
        flap.set_flap(Some(&sidebar.build()));
        flap.set_content(Some(&stack));
        flap.set_flap_position(gtk4::PackType::Start);
        flap.set_fold_policy(adw::FlapFoldPolicy::Auto);
        flap.set_locked(false);
        flap.set_reveal_flap(true);
        flap.set_swipe_to_open(true);
        flap.set_swipe_to_close(true);

        Self { root: flap }
    }

    pub fn build(&self) -> adw::Flap {
        self.root.clone()
    }
}
