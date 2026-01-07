//! Startup Manager Module
//! 
//! Manages autostart applications on system login.

use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AutostartApp {
    pub name: String,
    pub exec: String,
    pub comment: String,
    pub enabled: bool,
    pub path: PathBuf,
    pub hidden: bool,
}

/// Get list of autostart applications from XDG autostart directories
/// 
/// Searches in:
/// - ~/.config/autostart/
/// - /etc/xdg/autostart/
pub fn list_autostart_apps() -> Result<Vec<AutostartApp>> {
    let mut apps = Vec::new();
    
    // User autostart directory
    let user_dir = dirs::config_dir()
        .map(|p| p.join("autostart"))
        .unwrap_or_else(|| PathBuf::from(format!("{}/.config/autostart", 
            std::env::var("HOME").unwrap_or_default())));
    
    // System autostart directory
    let system_dir = PathBuf::from("/etc/xdg/autostart");
    
    // Read from user directory
    if user_dir.exists() {
        if let Ok(entries) = fs::read_dir(&user_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "desktop" {
                        if let Ok(app) = parse_desktop_file(&entry.path()) {
                            apps.push(app);
                        }
                    }
                }
            }
        }
    }
    
    // Read from system directory
    if system_dir.exists() {
        if let Ok(entries) = fs::read_dir(&system_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "desktop" {
                        let file_name = entry.file_name();
                        // Skip if already exists in user directory (user overrides system)
                        let user_override = user_dir.join(&file_name);
                        if !user_override.exists() {
                            if let Ok(app) = parse_desktop_file(&entry.path()) {
                                apps.push(app);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Sort by name
    apps.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(apps)
}

/// Parse a .desktop file to extract autostart information
fn parse_desktop_file(path: &Path) -> Result<AutostartApp> {
    let contents = fs::read_to_string(path)
        .context(format!("Failed to read {:?}", path))?;
    
    let mut name = String::new();
    let mut exec = String::new();
    let mut comment = String::new();
    let mut hidden = false;
    
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with("Name=") {
            name = line.strip_prefix("Name=").unwrap_or("").to_string();
        } else if line.starts_with("Exec=") {
            exec = line.strip_prefix("Exec=").unwrap_or("").to_string();
        } else if line.starts_with("Comment=") {
            comment = line.strip_prefix("Comment=").unwrap_or("").to_string();
        } else if line.starts_with("Hidden=") {
            hidden = line.strip_prefix("Hidden=").unwrap_or("false") == "true";
        }
    }
    
    if name.is_empty() {
        name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
    }
    
    Ok(AutostartApp {
        name,
        exec,
        comment,
        enabled: !hidden,
        path: path.to_path_buf(),
        hidden,
    })
}

/// Enable an autostart application
/// 
/// For system apps, creates an override file in ~/.config/autostart/
/// For user apps, removes the Hidden=true line
pub fn enable_autostart(app: &AutostartApp) -> Result<()> {
    let user_dir = dirs::config_dir()
        .map(|p| p.join("autostart"))
        .unwrap_or_else(|| PathBuf::from(format!("{}/.config/autostart", 
            std::env::var("HOME").unwrap_or_default())));
    
    fs::create_dir_all(&user_dir)?;
    
    let file_name = app.path.file_name()
        .context("Invalid path")?;
    let target_path = user_dir.join(file_name);
    
    // If it's a system file, copy it to user directory
    if !app.path.starts_with(&user_dir) {
        fs::copy(&app.path, &target_path)?;
    }
    
    // Remove Hidden=true line
    let contents = fs::read_to_string(&target_path)?;
    let new_contents: Vec<String> = contents
        .lines()
        .filter(|line| !line.trim().starts_with("Hidden="))
        .map(|s| s.to_string())
        .collect();
    
    fs::write(&target_path, new_contents.join("\n"))?;
    
    Ok(())
}

/// Disable an autostart application
/// 
/// Adds Hidden=true to the desktop file
pub fn disable_autostart(app: &AutostartApp) -> Result<()> {
    let user_dir = dirs::config_dir()
        .map(|p| p.join("autostart"))
        .unwrap_or_else(|| PathBuf::from(format!("{}/.config/autostart", 
            std::env::var("HOME").unwrap_or_default())));
    
    fs::create_dir_all(&user_dir)?;
    
    let file_name = app.path.file_name()
        .context("Invalid path")?;
    let target_path = user_dir.join(file_name);
    
    // If it's a system file, copy it to user directory
    if !app.path.starts_with(&user_dir) {
        fs::copy(&app.path, &target_path)?;
    }
    
    // Add Hidden=true line
    let contents = fs::read_to_string(&target_path)?;
    let mut lines: Vec<String> = contents
        .lines()
        .filter(|line| !line.trim().starts_with("Hidden="))
        .map(|s| s.to_string())
        .collect();
    
    // Add Hidden=true after [Desktop Entry] section
    let mut inserted = false;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "[Desktop Entry]" {
            lines.insert(i + 1, "Hidden=true".to_string());
            inserted = true;
            break;
        }
    }
    
    if !inserted {
        lines.push("Hidden=true".to_string());
    }
    
    fs::write(&target_path, lines.join("\n"))?;
    
    Ok(())
}

/// Remove an autostart application
/// 
/// Deletes the .desktop file from user autostart directory
pub fn remove_autostart(app: &AutostartApp) -> Result<()> {
    let user_dir = dirs::config_dir()
        .map(|p| p.join("autostart"))
        .unwrap_or_else(|| PathBuf::from(format!("{}/.config/autostart", 
            std::env::var("HOME").unwrap_or_default())));
    
    // Only allow removing from user directory
    if app.path.starts_with(&user_dir) {
        fs::remove_file(&app.path)?;
    } else {
        // For system files, disable instead of remove
        disable_autostart(app)?;
    }
    
    Ok(())
}
