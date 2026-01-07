//! Disk Analyzer Module
//! 
//! Provides disk usage information for mounted filesystems and folder analysis.

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct MountPoint {
    pub device: String,
    pub mount_point: PathBuf,
    pub fs_type: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
}

impl MountPoint {
    /// Calculate the percentage of disk space used
    pub fn used_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.used as f64 / self.total as f64) * 100.0
        }
    }

    /// Format bytes into human-readable size string
    pub fn format_size(bytes: u64) -> String {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;
        
        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else {
            format!("{} KB", bytes / 1024)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FolderInfo {
    pub path: PathBuf,
    pub size: u64,
    pub file_count: usize,
    pub dir_count: usize,
}

impl FolderInfo {
    pub fn format_size(&self) -> String {
        MountPoint::format_size(self.size)
    }
}

/// Get information about all mounted filesystems
/// 
/// Reads /proc/mounts and queries disk usage statistics.
/// Virtual filesystems (proc, sysfs, tmpfs, etc.) are filtered out.
pub fn get_mount_points() -> Result<Vec<MountPoint>> {
    let mut mounts = Vec::new();
    
    // Read /proc/mounts for mounted filesystems
    if let Ok(contents) = std::fs::read_to_string("/proc/mounts") {
        for line in contents.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }
            
            let device = parts[0].to_string();
            let mount_point = PathBuf::from(parts[1]);
            let fs_type = parts[2].to_string();
            
            // Skip virtual filesystems
            if fs_type.starts_with("fuse") || 
               fs_type == "tmpfs" || 
               fs_type == "devtmpfs" ||
               fs_type == "proc" ||
               fs_type == "sysfs" ||
               fs_type == "cgroup" ||
               fs_type == "devpts" {
                continue;
            }
            
            // Get disk usage stats using statvfs
            if let Ok(stat) = nix::sys::statvfs::statvfs(&mount_point) {
                let block_size = stat.block_size();
                let total = stat.blocks() * block_size;
                let available = stat.blocks_available() * block_size;
                let used = total - (stat.blocks_free() * block_size);
                
                mounts.push(MountPoint {
                    device,
                    mount_point,
                    fs_type,
                    total,
                    used,
                    available,
                });
            }
        }
    }
    
    Ok(mounts)
}

/// Analyze folder sizes in a given directory
/// 
/// Scans immediate subdirectories and calculates their sizes.
pub fn analyze_folder(path: &Path, max_depth: usize) -> Result<Vec<FolderInfo>> {
    let mut folders = Vec::new();
    
    if !path.is_dir() {
        return Ok(folders);
    }
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let folder_path = entry.path();
                    let (size, file_count, dir_count) = calculate_folder_size(&folder_path, 0, max_depth);
                    
                    folders.push(FolderInfo {
                        path: folder_path,
                        size,
                        file_count,
                        dir_count,
                    });
                }
            }
        }
    }
    
    // Sort by size (largest first)
    folders.sort_by(|a, b| b.size.cmp(&a.size));
    
    Ok(folders)
}

/// Calculate the total size of a folder recursively
fn calculate_folder_size(path: &Path, current_depth: usize, max_depth: usize) -> (u64, usize, usize) {
    let mut total_size = 0u64;
    let mut file_count = 0usize;
    let mut dir_count = 0usize;
    
    if current_depth > max_depth {
        return (total_size, file_count, dir_count);
    }
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                    file_count += 1;
                } else if metadata.is_dir() {
                    dir_count += 1;
                    let (sub_size, sub_files, sub_dirs) = calculate_folder_size(&entry.path(), current_depth + 1, max_depth);
                    total_size += sub_size;
                    file_count += sub_files;
                    dir_count += sub_dirs;
                }
            }
        }
    }
    
    (total_size, file_count, dir_count)
}

/// Get common large directories in home folder
pub fn get_common_large_folders() -> Result<Vec<FolderInfo>> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut folders = Vec::new();
    
    let common_paths = vec![
        format!("{}/.cache", home),
        format!("{}/.local/share", home),
        format!("{}/Downloads", home),
        format!("{}/Documents", home),
        format!("{}/Pictures", home),
        format!("{}/Videos", home),
        format!("{}/Music", home),
        format!("{}/.config", home),
    ];
    
    for path_str in common_paths {
        let path = PathBuf::from(&path_str);
        if path.exists() && path.is_dir() {
            let (size, file_count, dir_count) = calculate_folder_size(&path, 0, 3);
            folders.push(FolderInfo {
                path,
                size,
                file_count,
                dir_count,
            });
        }
    }
    
    // Sort by size (largest first)
    folders.sort_by(|a, b| b.size.cmp(&a.size));
    
    Ok(folders)
}

/// Get suggestions for disk cleanup
pub fn get_cleanup_suggestions() -> Vec<String> {
    vec![
        "Clear package cache: sudo apt-get clean".to_string(),
        "Remove old kernels: sudo apt-get autoremove".to_string(),
        "Clear thumbnail cache: rm -rf ~/.cache/thumbnails/*".to_string(),
        "Empty trash: rm -rf ~/.local/share/Trash/*".to_string(),
        "Clear browser cache in ~/.cache/mozilla or ~/.cache/google-chrome".to_string(),
        "Remove unused Flatpak runtimes: flatpak uninstall --unused".to_string(),
    ]
}
