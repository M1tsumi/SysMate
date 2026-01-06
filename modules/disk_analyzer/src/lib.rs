//! Disk Analyzer Module
//! 
//! Provides disk usage information for mounted filesystems.

use std::path::PathBuf;
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
