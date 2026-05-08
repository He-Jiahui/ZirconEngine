use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::engine_module::EngineModule;

pub trait PluginGroup: Sized {
    fn build(self) -> Result<PluginGroupBuilder, PluginGroupError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginGroupError {
    DuplicateKey { group: String, key: String },
    MissingKey { group: String, key: String },
    MissingAnchor { group: String, key: String },
    DisabledAnchor { group: String, key: String },
}

impl fmt::Display for PluginGroupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKey { group, key } => {
                write!(f, "plugin group {group} already contains module {key}")
            }
            Self::MissingKey { group, key } => {
                write!(f, "plugin group {group} does not contain module {key}")
            }
            Self::MissingAnchor { group, key } => {
                write!(f, "plugin group {group} has no ordering anchor {key}")
            }
            Self::DisabledAnchor { group, key } => write!(
                f,
                "plugin group {group} cannot order relative to disabled module {key}"
            ),
        }
    }
}

impl Error for PluginGroupError {}

#[derive(Clone, Debug)]
struct PluginEntry {
    module: Arc<dyn EngineModule>,
    enabled: bool,
}

#[derive(Clone, Debug)]
pub struct PluginGroupBuilder {
    group_name: String,
    entries: HashMap<String, PluginEntry>,
    order: Vec<String>,
}

impl PluginGroupBuilder {
    pub fn start(group_name: impl Into<String>) -> Self {
        Self {
            group_name: group_name.into(),
            entries: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn from_modules(
        group_name: impl Into<String>,
        modules: impl IntoIterator<Item = Arc<dyn EngineModule>>,
    ) -> Result<Self, PluginGroupError> {
        let mut builder = Self::start(group_name);
        for module in modules {
            builder = builder.add(module)?;
        }
        Ok(builder)
    }

    pub fn add(mut self, module: Arc<dyn EngineModule>) -> Result<Self, PluginGroupError> {
        let key = module.module_name().to_string();
        if self.entries.contains_key(&key) {
            return Err(self.duplicate_key(key));
        }
        self.order.push(key.clone());
        self.entries.insert(
            key,
            PluginEntry {
                module,
                enabled: true,
            },
        );
        Ok(self)
    }

    pub fn add_group(self, group: impl PluginGroup) -> Result<Self, PluginGroupError> {
        let mut builder = self;
        for module in group.build()?.finish().into_modules() {
            builder = builder.add(module)?;
        }
        Ok(builder)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn module_keys(&self) -> Vec<String> {
        self.order
            .iter()
            .filter(|key| self.entries.get(*key).is_some_and(|entry| entry.enabled))
            .cloned()
            .collect()
    }

    pub fn set(mut self, module: Arc<dyn EngineModule>) -> Result<Self, PluginGroupError> {
        let key = module.module_name().to_string();
        let Some(entry) = self.entries.get_mut(&key) else {
            return Err(self.missing_key(key));
        };
        entry.module = module;
        Ok(self)
    }

    pub fn disable(mut self, key: &str) -> Result<Self, PluginGroupError> {
        let Some(entry) = self.entries.get_mut(key) else {
            return Err(self.missing_key(key.to_string()));
        };
        entry.enabled = false;
        Ok(self)
    }

    pub fn enable(mut self, key: &str) -> Result<Self, PluginGroupError> {
        let Some(entry) = self.entries.get_mut(key) else {
            return Err(self.missing_key(key.to_string()));
        };
        entry.enabled = true;
        Ok(self)
    }

    pub fn add_before(
        mut self,
        anchor: &str,
        module: Arc<dyn EngineModule>,
    ) -> Result<Self, PluginGroupError> {
        let key = module.module_name().to_string();
        if self.entries.contains_key(&key) {
            return Err(self.duplicate_key(key));
        }
        let anchor_index = self.enabled_anchor_index(anchor)?;
        self.order.insert(anchor_index, key.clone());
        self.entries.insert(
            key,
            PluginEntry {
                module,
                enabled: true,
            },
        );
        Ok(self)
    }

    pub fn add_after(
        mut self,
        anchor: &str,
        module: Arc<dyn EngineModule>,
    ) -> Result<Self, PluginGroupError> {
        let key = module.module_name().to_string();
        if self.entries.contains_key(&key) {
            return Err(self.duplicate_key(key));
        }
        let anchor_index = self.enabled_anchor_index(anchor)? + 1;
        self.order.insert(anchor_index, key.clone());
        self.entries.insert(
            key,
            PluginEntry {
                module,
                enabled: true,
            },
        );
        Ok(self)
    }

    pub fn finish(mut self) -> ResolvedPluginGroup {
        let modules = self
            .order
            .into_iter()
            .filter_map(|key| {
                let entry = self.entries.remove(&key)?;
                entry.enabled.then_some(entry.module)
            })
            .collect();
        ResolvedPluginGroup {
            name: self.group_name,
            modules,
        }
    }

    fn enabled_anchor_index(&self, key: &str) -> Result<usize, PluginGroupError> {
        let Some(index) = self.order.iter().position(|candidate| candidate == key) else {
            return Err(self.missing_anchor(key.to_string()));
        };
        if self.entries.get(key).is_some_and(|entry| !entry.enabled) {
            return Err(self.disabled_anchor(key.to_string()));
        }
        Ok(index)
    }

    fn duplicate_key(&self, key: String) -> PluginGroupError {
        PluginGroupError::DuplicateKey {
            group: self.group_name.clone(),
            key,
        }
    }

    fn missing_key(&self, key: String) -> PluginGroupError {
        PluginGroupError::MissingKey {
            group: self.group_name.clone(),
            key,
        }
    }

    fn missing_anchor(&self, key: String) -> PluginGroupError {
        PluginGroupError::MissingAnchor {
            group: self.group_name.clone(),
            key,
        }
    }

    fn disabled_anchor(&self, key: String) -> PluginGroupError {
        PluginGroupError::DisabledAnchor {
            group: self.group_name.clone(),
            key,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedPluginGroup {
    name: String,
    modules: Vec<Arc<dyn EngineModule>>,
}

impl ResolvedPluginGroup {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn modules(&self) -> &[Arc<dyn EngineModule>] {
        &self.modules
    }

    pub fn module_keys(&self) -> Vec<&'static str> {
        self.modules
            .iter()
            .map(|module| module.module_name())
            .collect()
    }

    pub fn module_descriptors(&self) -> Vec<ModuleDescriptor> {
        self.modules
            .iter()
            .map(|module| module.descriptor())
            .collect()
    }

    pub fn into_modules(self) -> Vec<Arc<dyn EngineModule>> {
        self.modules
    }
}
