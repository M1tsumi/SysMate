use std::collections::HashMap;
use anyhow::Result;

/// Trait that all modules must implement (reserved for future dynamic module loading)
#[allow(dead_code)]
pub trait Module: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn icon_name(&self) -> &str;
    fn initialize(&mut self) -> Result<()>;
}

/// Manages all loaded modules (reserved for future dynamic module loading)
pub struct ModuleManager {
    #[allow(dead_code)]
    modules: HashMap<String, Box<dyn Module>>,
}

impl ModuleManager {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Register a new module
    #[allow(dead_code)]
    pub fn register_module(&mut self, module: Box<dyn Module>) -> Result<()> {
        let name = module.name().to_string();
        self.modules.insert(name, module);
        Ok(())
    }

    /// Get all registered modules
    #[allow(dead_code)]
    pub fn get_modules(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }

    /// Get a specific module by name
    #[allow(dead_code)]
    pub fn get_module(&self, name: &str) -> Option<&Box<dyn Module>> {
        self.modules.get(name)
    }
}

impl Default for ModuleManager {
    fn default() -> Self {
        Self::new()
    }
}
