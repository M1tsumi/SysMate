use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

mod app;
mod module_loader;
mod ui;
mod system_info;

use app::HealthCenterApp;

const APP_ID: &str = "com.sysmate";

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create GTK application
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    // Run the application
    let _ = app.run();
    Ok(())
}

fn build_ui(app: &Application) {
    // Initialize Libadwaita
    adw::init().expect("Failed to initialize Libadwaita");

    // Enable dark theme
    let style_manager = adw::StyleManager::default();
    style_manager.set_color_scheme(adw::ColorScheme::PreferDark);

    // Create and show the main application
    let health_app = HealthCenterApp::new(app);
    health_app.show();
}
