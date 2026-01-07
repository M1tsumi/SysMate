//! Package Manager Module
//! 
//! Provides information about installed packages across multiple package managers.

use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PackageStats {
    pub total_installed: usize,
    pub upgradeable: usize,
    pub auto_removable: usize,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
/// Get package statistics from APT
/// 
/// Queries dpkg and apt for installed, upgradeable, and auto-removable packages.
    pub version: String,
    pub description: String,
}

pub fn get_package_stats() -> Result<PackageStats> {
    let mut stats = PackageStats {
        total_installed: 0,
        upgradeable: 0,
        auto_removable: 0,
    };
    
    // Count installed packages
    if let Ok(output) = Command::new("dpkg-query")
        .args(&["-l"])
        .output()
    {
        stats.total_installed = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| line.starts_with("ii"))
            .count();
    }
    
    // Count upgradeable packages
    if let Ok(output) = Command::new("apt")
        .args(&["list", "--upgradable"])
        .output()
    {
        stats.upgradeable = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| line.contains("upgradable"))
            .count();
    }
    
    // Count auto-removable packages
    if let Ok(output) = Command::new("apt")
        .args(&["autoremove", "--dry-run"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("no longer required") {
                if let Some(count_str) = line.split_whitespace().next() {
                    if let Ok(count) = count_str.parse() {
                        stats.auto_removable = count;
                    }
                }
            }
        }
    }
    
    Ok(stats)
}

/// List recently installed or upgraded packages
/// 
/// Parses /var/log/apt/history.log for recent package activity.
pub fn list_recent_packages(limit: usize) -> Result<Vec<PackageInfo>> {
    let mut packages = Vec::new();
    
    // Get recently installed/upgraded packages from apt history
    if let Ok(output) = Command::new("grep")
        .args(&["-E", "^(Install|Upgrade):", "/var/log/apt/history.log"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines().take(limit) {
            if let Some(pkg_part) = line.split(':').nth(1) {
                for pkg_info in pkg_part.split(',').take(3) {
                    let parts: Vec<&str> = pkg_info.trim().split_whitespace().collect();
                    if parts.len() >= 2 {
                        packages.push(PackageInfo {
                            name: parts[0].trim_matches(|c| c == '(' || c == ')').to_string(),
                            version: parts[1].trim_matches(|c| c == '(' || c == ')').to_string(),
                            description: String::new(),
                        });
                    }
                }
            }
            
            if packages.len() >= limit {
                break;
            }
        }
    }
    
    Ok(packages)
}

/// Count installed Snap packages
pub fn get_snap_count() -> usize {
    if let Ok(output) = Command::new("snap")
        .args(&["list"])
        .output()
    {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(1) // Skip header
            .count()
    } else {
        0
    }
}

/// Count installed Flatpak applications
pub fn get_flatpak_count() -> usize {
    if let Ok(output) = Command::new("flatpak")
        .args(&["list", "--app"])
        .output()
    {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .count()
    } else {
        0
    }
}
