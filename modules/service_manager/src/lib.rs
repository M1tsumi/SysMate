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
/// Queries systemctl for service information.
pub fn list_services() -> Result<Vec<ServiceInfo>> {
    list_services_with_limit(None)
}

/// List systemd services with optional limit
pub fn list_services_with_limit(limit: Option<usize>) -> Result<Vec<ServiceInfo>> {
    let mut services = Vec::new();
    
    // Get list of services using systemctl
    let output = Command::new("systemctl")
        .args(&["list-unit-files", "--type=service", "--no-pager", "--no-legend"])
        .output()?;
    
    if !output.status.success() {
        return Ok(services);
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = if let Some(lim) = limit {
        output_str.lines().take(lim).collect()
    } else {
        output_str.lines().collect()
    };
    
    for line in lines {
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

/// Filter services by state
pub fn filter_services_by_state(services: &[ServiceInfo], state: &ServiceState) -> Vec<ServiceInfo> {
    services.iter()
        .filter(|s| s.state == *state)
        .cloned()
        .collect()
}

/// Search services by name
pub fn search_services(query: &str) -> Result<Vec<ServiceInfo>> {
    let all_services = list_services()?;
    Ok(all_services.into_iter()
        .filter(|s| s.name.to_lowercase().contains(&query.to_lowercase()))
        .collect())
}

/// Start a service (requires sudo)
pub fn start_service(service: &str) -> Result<()> {
    Command::new("pkexec")
        .args(&["systemctl", "start", &format!("{}.service", service)])
        .output()?;
    Ok(())
}

/// Stop a service (requires sudo)
pub fn stop_service(service: &str) -> Result<()> {
    Command::new("pkexec")
        .args(&["systemctl", "stop", &format!("{}.service", service)])
        .output()?;
    Ok(())
}

/// Restart a service (requires sudo)
pub fn restart_service(service: &str) -> Result<()> {
    Command::new("pkexec")
        .args(&["systemctl", "restart", &format!("{}.service", service)])
        .output()?;
    Ok(())
}

/// Enable a service (requires sudo)
pub fn enable_service(service: &str) -> Result<()> {
    Command::new("pkexec")
        .args(&["systemctl", "enable", &format!("{}.service", service)])
        .output()?;
    Ok(())
}

/// Disable a service (requires sudo)
pub fn disable_service(service: &str) -> Result<()> {
    Command::new("pkexec")
        .args(&["systemctl", "disable", &format!("{}.service", service)])
        .output()?;
    Ok(())
}

/// Get service logs
pub fn get_service_logs(service: &str, lines: usize) -> Result<String> {
    let output = Command::new("journalctl")
        .args(&[
            "-u",
            &format!("{}.service", service),
            "-n",
            &lines.to_string(),
            "--no-pager"
        ])
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get detailed service status
pub fn get_service_status(service: &str) -> Result<String> {
    let output = Command::new("systemctl")
        .args(&["status", &format!("{}.service", service), "--no-pager"])
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
