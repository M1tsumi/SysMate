use gtk4::{Application, prelude::*};
use libadwaita as adw;
use adw::prelude::*;

use crate::ui::MainWindow;
use crate::module_loader::ModuleManager;

pub struct HealthCenterApp {
    window: adw::ApplicationWindow,
    _module_manager: ModuleManager,
}

impl HealthCenterApp {
    pub fn new(app: &Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .default_width(1000)
            .default_height(700)
            .title("SysMate")
            .build();

        // Create module manager
        let module_manager = ModuleManager::new();

        // Build the UI
        let main_window = MainWindow::new();
        window.set_content(Some(&main_window.build()));

        Self {
            window,
            _module_manager: module_manager,
        }
    }

    pub fn show(&self) {
        self.window.present();
    }
}
