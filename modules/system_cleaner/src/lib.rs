//! System Cleaner Module
//! 
//! Provides system cleaning capabilities for freeing up disk space.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum CleanupCategory {
    PackageCache,
    Thumbnails,
    Trash,
    Logs,
    OldKernels,
    BrowserCache,
    TempFiles,
}

impl CleanupCategory {
    pub fn name(&self) -> &str {
        match self {
            CleanupCategory::PackageCache => "Package Cache",
            CleanupCategory::Thumbnails => "Thumbnail Cache",
            CleanupCategory::Trash => "Trash",
            CleanupCategory::Logs => "Old System Logs",
            CleanupCategory::OldKernels => "Old Kernels",
            CleanupCategory::BrowserCache => "Browser Cache",
            CleanupCategory::TempFiles => "Temporary Files",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            CleanupCategory::PackageCache => "APT package cache and downloaded .deb files",
            CleanupCategory::Thumbnails => "Cached thumbnail images",
            CleanupCategory::Trash => "Files in trash bin",
            CleanupCategory::Logs => "Rotated and old system log files",
            CleanupCategory::OldKernels => "Old kernel versions (keeps current and one previous)",
            CleanupCategory::BrowserCache => "Firefox and Chrome cache files",
            CleanupCategory::TempFiles => "Temporary files in /tmp and /var/tmp",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CleanupItem {
    pub category: CleanupCategory,
    pub size: u64,
    pub count: usize,
    pub paths: Vec<PathBuf>,
}

/// Calculate the size of a directory recursively
fn calculate_dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += calculate_dir_size(&entry.path());
                }
            }
        }
    }
    
    total
}

/// Count files in a directory recursively
fn count_files_in_dir(path: &Path) -> usize {
    let mut count = 0;
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    count += 1;
                } else if metadata.is_dir() {
                    count += count_files_in_dir(&entry.path());
                }
            }
        }
    }
    
    count
}

/// Scan system for cleanable items
pub fn scan_cleanable_items() -> Result<Vec<CleanupItem>> {
    let mut items = Vec::new();

    // Package cache
    let apt_cache = PathBuf::from("/var/cache/apt/archives");
    if apt_cache.exists() {
        let size = calculate_dir_size(&apt_cache);
        let count = count_files_in_dir(&apt_cache);
        items.push(CleanupItem {
            category: CleanupCategory::PackageCache,
            size,
            count,
            paths: vec![apt_cache],
        });
    }

    // Thumbnails
    let home = std::env::var("HOME").unwrap_or_default();
    let thumbnails = PathBuf::from(format!("{}/.cache/thumbnails", home));
    if thumbnails.exists() {
        let size = calculate_dir_size(&thumbnails);
        let count = count_files_in_dir(&thumbnails);
        items.push(CleanupItem {
            category: CleanupCategory::Thumbnails,
            size,
            count,
            paths: vec![thumbnails],
        });
    }

    // Trash
    let trash_files = PathBuf::from(format!("{}/.local/share/Trash/files", home));
    if trash_files.exists() {
        let size = calculate_dir_size(&trash_files);
        let count = count_files_in_dir(&trash_files);
        items.push(CleanupItem {
            category: CleanupCategory::Trash,
            size,
            count,
            paths: vec![trash_files],
        });
    }

    // Old logs (journalctl)
    let journal_path = PathBuf::from("/var/log/journal");
    if journal_path.exists() {
        let size = calculate_dir_size(&journal_path);
        let count = count_files_in_dir(&journal_path);
        items.push(CleanupItem {
            category: CleanupCategory::Logs,
            size,
            count,
            paths: vec![journal_path],
        });
    }

    // Browser caches
    let firefox_cache = PathBuf::from(format!("{}/.cache/mozilla/firefox", home));
    let chrome_cache = PathBuf::from(format!("{}/.cache/google-chrome", home));
    let mut browser_size = 0u64;
    let mut browser_count = 0usize;
    let mut browser_paths = Vec::new();

    if firefox_cache.exists() {
        browser_size += calculate_dir_size(&firefox_cache);
        browser_count += count_files_in_dir(&firefox_cache);
        browser_paths.push(firefox_cache);
    }
    if chrome_cache.exists() {
        browser_size += calculate_dir_size(&chrome_cache);
        browser_count += count_files_in_dir(&chrome_cache);
        browser_paths.push(chrome_cache);
    }

    if browser_size > 0 {
        items.push(CleanupItem {
            category: CleanupCategory::BrowserCache,
            size: browser_size,
            count: browser_count,
            paths: browser_paths,
        });
    }

    // Temp files
    let tmp = PathBuf::from("/tmp");
    if tmp.exists() {
        let size = calculate_dir_size(&tmp);
        let count = count_files_in_dir(&tmp);
        if count > 0 {
            items.push(CleanupItem {
                category: CleanupCategory::TempFiles,
                size,
                count,
                paths: vec![tmp],
            });
        }
    }

    Ok(items)
}

/// Clean package cache using apt-get clean
pub fn clean_package_cache() -> Result<()> {
    Command::new("pkexec")
        .args(&["apt-get", "clean"])
        .output()?;
    Ok(())
}

/// Clean thumbnails cache
pub fn clean_thumbnails() -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_default();
    let thumbnails = PathBuf::from(format!("{}/.cache/thumbnails", home));
    if thumbnails.exists() {
        fs::remove_dir_all(&thumbnails)?;
        fs::create_dir_all(&thumbnails)?;
    }
    Ok(())
}

/// Empty trash
pub fn empty_trash() -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_default();
    let trash_files = PathBuf::from(format!("{}/.local/share/Trash/files", home));
    let trash_info = PathBuf::from(format!("{}/.local/share/Trash/info", home));
    
    if trash_files.exists() {
        fs::remove_dir_all(&trash_files)?;
        fs::create_dir_all(&trash_files)?;
    }
    if trash_info.exists() {
        fs::remove_dir_all(&trash_info)?;
        fs::create_dir_all(&trash_info)?;
    }
    Ok(())
}

/// Clean old system logs using journalctl
pub fn clean_old_logs() -> Result<()> {
    Command::new("pkexec")
        .args(&["journalctl", "--vacuum-time=7d"])
        .output()?;
    Ok(())
}

/// Clean browser caches
pub fn clean_browser_cache() -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_default();
    let firefox_cache = PathBuf::from(format!("{}/.cache/mozilla/firefox", home));
    let chrome_cache = PathBuf::from(format!("{}/.cache/google-chrome", home));
    
    if firefox_cache.exists() {
        fs::remove_dir_all(&firefox_cache)?;
        fs::create_dir_all(&firefox_cache)?;
    }
    if chrome_cache.exists() {
        fs::remove_dir_all(&chrome_cache)?;
        fs::create_dir_all(&chrome_cache)?;
    }
    Ok(())
}

/// Clean temporary files (requires root)
pub fn clean_temp_files() -> Result<()> {
    Command::new("pkexec")
        .args(&["rm", "-rf", "/tmp/*"])
        .output()?;
    Ok(())
}

/// Clean a specific category
pub fn clean_category(category: &CleanupCategory) -> Result<()> {
    match category {
        CleanupCategory::PackageCache => clean_package_cache(),
        CleanupCategory::Thumbnails => clean_thumbnails(),
        CleanupCategory::Trash => empty_trash(),
        CleanupCategory::Logs => clean_old_logs(),
        CleanupCategory::BrowserCache => clean_browser_cache(),
        CleanupCategory::TempFiles => clean_temp_files(),
        CleanupCategory::OldKernels => {
            Command::new("pkexec")
                .args(&["apt-get", "autoremove", "--purge", "-y"])
                .output()?;
            Ok(())
        }
    }
}

/// Format bytes into human-readable size string
pub fn format_size(bytes: u64) -> String {
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
        format!("{} bytes", bytes)
    }
}
