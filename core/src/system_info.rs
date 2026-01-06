use sysinfo::System;
use std::time::Duration;
use std::fs;

#[derive(Debug, Clone)]
pub struct TemperatureSensor {
    pub name: String,
    pub temperature: f32,
    pub label: String,
}

pub struct SystemInfo {
    system: System,
}

impl SystemInfo {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }

    /// Refresh system information (reserved for future live updates)
    #[allow(dead_code)]
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn os_version(&self) -> String {
        System::long_os_version().unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn hostname(&self) -> String {
        System::host_name().unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn kernel_version(&self) -> String {
        System::kernel_version().unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn uptime(&self) -> Duration {
        Duration::from_secs(System::uptime())
    }

    pub fn total_memory(&self) -> u64 {
        self.system.total_memory()
    }

    pub fn used_memory(&self) -> u64 {
        self.system.used_memory()
    }

    pub fn total_swap(&self) -> u64 {
        self.system.total_swap()
    }

    pub fn used_swap(&self) -> u64 {
        self.system.used_swap()
    }
    
    pub fn available_memory(&self) -> u64 {
        self.system.available_memory()
    }
    
    pub fn free_memory(&self) -> u64 {
        self.system.free_memory()
    }

    pub fn cpu_count(&self) -> usize {
        self.system.cpus().len()
    }

    pub fn format_uptime(&self) -> String {
        let uptime = self.uptime();
        let days = uptime.as_secs() / 86400;
        let hours = (uptime.as_secs() % 86400) / 3600;
        let minutes = (uptime.as_secs() % 3600) / 60;

        if days > 0 {
            format!("{} days, {} hours", days, hours)
        } else if hours > 0 {
            format!("{} hours, {} minutes", hours, minutes)
        } else {
            format!("{} minutes", minutes)
        }
    }

    pub fn format_memory(bytes: u64) -> String {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;
        
        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        }
    }
    
    /// Get CPU and hardware temperatures
    pub fn get_temperatures() -> Vec<TemperatureSensor> {
        let mut sensors = Vec::new();
        
        // Read thermal zones (CPU temps)
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && entry.file_name().to_string_lossy().starts_with("thermal_zone") {
                    if let Ok(temp_str) = fs::read_to_string(path.join("temp")) {
                        if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                            let temp_celsius = temp_millidegrees as f32 / 1000.0;
                            
                            // Try to get the zone type/label
                            let label = fs::read_to_string(path.join("type"))
                                .unwrap_or_else(|_| entry.file_name().to_string_lossy().to_string())
                                .trim()
                                .to_string();
                            
                            sensors.push(TemperatureSensor {
                                name: entry.file_name().to_string_lossy().to_string(),
                                temperature: temp_celsius,
                                label,
                            });
                        }
                    }
                }
            }
        }
        
        // Read hwmon sensors (additional hardware monitoring)
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Look for temp inputs
                    for i in 1..20 {
                        let temp_input = path.join(format!("temp{}_input", i));
                        if temp_input.exists() {
                            if let Ok(temp_str) = fs::read_to_string(&temp_input) {
                                if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                                    let temp_celsius = temp_millidegrees as f32 / 1000.0;
                                    
                                    // Try to get label
                                    let label_path = path.join(format!("temp{}_label", i));
                                    let label = if label_path.exists() {
                                        fs::read_to_string(label_path)
                                            .unwrap_or_else(|_| format!("Sensor {}", i))
                                            .trim()
                                            .to_string()
                                    } else {
                                        format!("Sensor {}", i)
                                    };
                                    
                                    // Get hwmon name
                                    let name = fs::read_to_string(path.join("name"))
                                        .unwrap_or_else(|_| entry.file_name().to_string_lossy().to_string())
                                        .trim()
                                        .to_string();
                                    
                                    sensors.push(TemperatureSensor {
                                        name: format!("{} - {}", name, label),
                                        temperature: temp_celsius,
                                        label,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Remove duplicates and sort by temperature (hottest first)
        sensors.sort_by(|a, b| b.temperature.partial_cmp(&a.temperature).unwrap());
        sensors.dedup_by(|a, b| (a.temperature - b.temperature).abs() < 0.1 && a.label == b.label);
        
        sensors
    }
    
    /// Format temperature with color coding
    pub fn format_temperature(celsius: f32) -> String {
        format!("{:.1}°C", celsius)
    }
    
    /// Get temperature status (normal, warm, hot)
    pub fn temperature_status(celsius: f32) -> &'static str {
        if celsius < 50.0 {
            "Normal"
        } else if celsius < 70.0 {
            "Warm"
        } else if celsius < 85.0 {
            "Hot"
        } else {
            "Critical"
        }
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::new()
    }
}
