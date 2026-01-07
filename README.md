# SysMate

A Linux system management tool that puts all your common maintenance tasks in one place. Think of it as a control center for keeping your system running smoothly.

## What it does

SysMate brings together the tools you need for system maintenance without constantly switching between different apps. Each feature runs as its own module, so you only load what you need.

The UI is built with GTK4 and Libadwaita, so it looks right at home on modern Linux desktops. For anything that needs root access, we use PolicyKit to handle permissions the proper way.

## Current features

**Working now:**
- System information dashboard
- Task manager with live resource monitoring
- Disk space analyzer
- Package manager interface
- Service management
- Startup programs control
- System cleanup tools

**On the roadmap:**
- Backup management
- Network monitoring
- Log viewer

## Building from source

You'll need GTK4 and Libadwaita development files installed:

```bash
# Ubuntu/Debian
sudo apt install libgtk-4-dev libadwaita-1-dev build-essential

# Build and run
cargo build --release
cargo run --bin sysmate
```

## Project structure

```
sysmate/
├── core/           # Main app and UI
└── modules/        # Individual feature modules
    ├── disk_analyzer/
    ├── package_manager/
    ├── service_manager/
    ├── startup_manager/
    └── system_cleaner/
```

Each module is independent, making it easier to work on features without touching the core app.

## License

GPL-3.0-or-later
