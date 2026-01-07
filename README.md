# SysMate

A modular system management dashboard for Linux that unifies common maintenance tasks. Your friendly system companion.

## Features

- 🎯 **Modular Architecture**: Each feature is an independent module
- 🔒 **Secure**: PolicyKit integration for privileged operations
- 🎨 **Native Look**: Built with GTK4 and Libadwaita
- 📦 **Easy Distribution**: Snap, .deb, and Flatpak support

## Modules

### Phase 1 - Foundation
- ✅ Core Dashboard with module loader
- ✅ Basic system info display

### Phase 2 - Essential Modules (In Development)
- 🔄 Disk Analyzer
- 🔄 Package Manager
- 🔄 Service Manager

### Phase 3+ - Coming Soon
- Startup Manager
- System Cleaner
- Resource Monitor
- Backup Manager

## Building

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt install libgtk-4-dev libadwaita-1-dev build-essential

# Build
cargo build --release

# Run
cargo run --bin sysmate
```

## Development

This project uses a Cargo workspace with the following structure:

```
sysmate/
├── core/           # Main application
└── modules/        # Feature modules
    ├── disk_analyzer/
    ├── package_manager/
    └── service_manager/
```

## License

GPL-3.0-or-later
