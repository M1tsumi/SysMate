//! Service Manager Module
//! 
//! Provides systemd service information and management capabilities.

use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    pub state: ServiceState,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
    Unknown,
}

impl ServiceState {
    pub fn as_str(&self) -> &str {
        match self {
            ServiceState::Active => "Active",
            ServiceState::Inactive => "Inactive",
            ServiceState::Failed => "Failed",
            ServiceState::Unknown => "Unknown",
        }
    }
    
    pub fn icon(&self) -> &str {
        match self {
            ServiceState::Active => "emblem-ok-symbolic",
            ServiceState::Inactive => "media-playback-pause-symbolic",
            ServiceState::Failed => "dialog-error-symbolic",
            ServiceState::Unknown => "dialog-question-symbolic",
        }
    }
}

/// List systemd services with their current status
/// 
/// Queries systemctl for service information. Limited to 50 services for alpha.
pub fn list_services() -> Result<Vec<ServiceInfo>> {
    let mut services = Vec::new();
    
    // Get list of services using systemctl
    let output = Command::new("systemctl")
        .args(&["list-unit-files", "--type=service", "--no-pager", "--no-legend"])
        .output()?;
    
    if !output.status.success() {
        return Ok(services);
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    for line in output_str.lines().take(50) { // Limit to first 50 services for alpha
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        
        let name = parts[0].trim_end_matches(".service").to_string();
        let enabled = parts[1] == "enabled";
        
        // Get service status
        let status_output = Command::new("systemctl")
            .args(&["is-active", &format!("{}.service", name)])
            .output()
            .ok();
        
        let state = if let Some(output) = status_output {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            match status.as_str() {
                "active" => ServiceState::Active,
                "inactive" => ServiceState::Inactive,
                "failed" => ServiceState::Failed,
                _ => ServiceState::Unknown,
            }
        } else {
            ServiceState::Unknown
        };
        
        // Get service description
        let desc_output = Command::new("systemctl")
            .args(&["show", &format!("{}.service", name), "-p", "Description", "--value"])
            .output()
            .ok();
        
        let description = if let Some(output) = desc_output {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            String::new()
        };
        
        services.push(ServiceInfo {
            name,
            description,
            state,
            enabled,
        });
    }
    
    // Sort by state (active first) then by name
    services.sort_by(|a, b| {
        if a.state == b.state {
            a.name.cmp(&b.name)
        } else if a.state == ServiceState::Active {
            std::cmp::Ordering::Less
        } else if b.state == ServiceState::Active {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });
    
    Ok(services)
}
