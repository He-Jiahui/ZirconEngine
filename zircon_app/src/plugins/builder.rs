use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use zircon_runtime::core::{sort_module_activation_order, ModuleDescriptor};
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
    ModuleOrder { group: String, reason: String },
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
            Self::ModuleOrder { group, reason } => {
                write!(f, "plugin group {group} has invalid module order: {reason}")
            }
        }
    }
}

impl Error for PluginGroupError {}

#[derive(Clone, Debug)]
struct PluginEntry {
    module: Arc<dyn EngineModule>,
    enabled: bool,
    descriptor: Option<ModuleDescriptor>,
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
            builder = builder.add_module(module)?;
        }
        Ok(builder)
    }

    pub fn add_module(mut self, module: Arc<dyn EngineModule>) -> Result<Self, PluginGroupError> {
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
                descriptor: None,
            },
        );
        Ok(self)
    }

    pub fn add_group(self, group: impl PluginGroup) -> Result<Self, PluginGroupError> {
        let mut builder = self;
        for (module, descriptor) in group.build()?.try_finish()?.into_entries() {
            builder = builder.add_resolved_module(module, descriptor)?;
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
        entry.descriptor = None;
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
                descriptor: None,
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
                descriptor: None,
            },
        );
        Ok(self)
    }

    pub fn try_finish(mut self) -> Result<ResolvedPluginGroup, PluginGroupError> {
        let group_name = self.group_name.clone();
        let (modules, descriptors) = self
            .order
            .into_iter()
            .filter_map(|key| {
                let entry = self.entries.remove(&key)?;
                entry.enabled.then(|| {
                    let descriptor = entry
                        .descriptor
                        .unwrap_or_else(|| entry.module.descriptor());
                    (entry.module, descriptor)
                })
            })
            .unzip();
        let (modules, descriptors) = sort_group_modules(&group_name, modules, descriptors)?;
        Ok(ResolvedPluginGroup {
            name: self.group_name,
            modules,
            descriptors,
        })
    }

    #[cfg(test)]
    pub(crate) fn finish(self) -> ResolvedPluginGroup {
        self.try_finish()
            .expect("plugin group module descriptors must sort")
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

    fn add_resolved_module(
        mut self,
        module: Arc<dyn EngineModule>,
        descriptor: ModuleDescriptor,
    ) -> Result<Self, PluginGroupError> {
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
                descriptor: Some(descriptor),
            },
        );
        Ok(self)
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

    fn module_order_error(group: &str, reason: String) -> PluginGroupError {
        PluginGroupError::ModuleOrder {
            group: group.to_owned(),
            reason,
        }
    }
}

fn sort_group_modules(
    group_name: &str,
    modules: Vec<Arc<dyn EngineModule>>,
    descriptors: Vec<ModuleDescriptor>,
) -> Result<(Vec<Arc<dyn EngineModule>>, Vec<ModuleDescriptor>), PluginGroupError> {
    let order = sort_module_activation_order(&descriptors)
        .map_err(|error| PluginGroupBuilder::module_order_error(group_name, error.to_string()))?;
    let mut entries_by_name = modules
        .into_iter()
        .zip(descriptors)
        .map(|(module, descriptor)| (module.module_name().to_owned(), (module, descriptor)))
        .collect::<HashMap<_, _>>();
    let entries = order
        .into_iter()
        .map(|module_name| {
            entries_by_name.remove(&module_name).ok_or_else(|| {
                PluginGroupBuilder::module_order_error(
                    group_name,
                    format!("descriptor references missing module {module_name}"),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries.into_iter().unzip())
}

#[derive(Clone, Debug)]
pub struct ResolvedPluginGroup {
    name: String,
    modules: Vec<Arc<dyn EngineModule>>,
    // Invariant: descriptors and modules have identical activation-order indices.
    descriptors: Vec<ModuleDescriptor>,
}

impl ResolvedPluginGroup {
    pub(crate) fn from_compiled_module_graph(
        name: impl Into<String>,
        modules: Vec<Arc<dyn EngineModule>>,
        descriptors: Vec<ModuleDescriptor>,
    ) -> Result<Self, PluginGroupError> {
        let name = name.into();
        if modules.len() != descriptors.len()
            || modules
                .iter()
                .zip(&descriptors)
                .any(|(module, descriptor)| module.module_name() != descriptor.name)
        {
            return Err(PluginGroupError::ModuleOrder {
                group: name,
                reason: "compiled module and descriptor order diverged".to_owned(),
            });
        }
        Ok(Self {
            name,
            modules,
            descriptors,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn modules(&self) -> &[Arc<dyn EngineModule>] {
        &self.modules
    }

    pub fn module_keys(&self) -> Vec<&str> {
        self.modules
            .iter()
            .map(|module| module.module_name())
            .collect()
    }

    pub fn module_descriptors(&self) -> &[ModuleDescriptor] {
        &self.descriptors
    }

    pub fn into_modules(self) -> Vec<Arc<dyn EngineModule>> {
        self.modules
    }

    fn into_entries(self) -> Vec<(Arc<dyn EngineModule>, ModuleDescriptor)> {
        self.modules.into_iter().zip(self.descriptors).collect()
    }
}
