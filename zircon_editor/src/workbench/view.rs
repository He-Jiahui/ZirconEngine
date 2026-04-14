//! View descriptors, instances, and registry for the editor workbench.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::autolayout::PaneConstraints;
use crate::layout::{ActivityDrawerSlot, MainPageId};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ViewDescriptorId(pub(crate) String);

impl ViewDescriptorId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ViewInstanceId(pub(crate) String);

impl ViewInstanceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewKind {
    ActivityView,
    ActivityWindow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockPolicy {
    DrawerOnly,
    DocumentOnly,
    DrawerOrDocument,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ViewHost {
    Drawer(ActivityDrawerSlot),
    Document(MainPageId, Vec<usize>),
    FloatingWindow(MainPageId, Vec<usize>),
    ExclusivePage(MainPageId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreferredHost {
    Drawer(ActivityDrawerSlot),
    DocumentCenter,
    FloatingWindow,
    ExclusiveMainPage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewDescriptor {
    pub descriptor_id: ViewDescriptorId,
    pub kind: ViewKind,
    pub default_title: String,
    pub icon_key: String,
    pub multi_instance: bool,
    pub dock_policy: DockPolicy,
    pub preferred_drawer_slot: Option<ActivityDrawerSlot>,
    pub preferred_host: PreferredHost,
    pub persistence_key_policy: String,
    #[serde(default)]
    pub default_constraints: PaneConstraints,
}

impl ViewDescriptor {
    pub fn new(
        descriptor_id: ViewDescriptorId,
        kind: ViewKind,
        default_title: impl Into<String>,
    ) -> Self {
        let descriptor_key = descriptor_id.0.clone();
        Self {
            descriptor_id,
            kind,
            default_title: default_title.into(),
            icon_key: descriptor_key.clone(),
            multi_instance: false,
            dock_policy: DockPolicy::DrawerOrDocument,
            preferred_drawer_slot: None,
            preferred_host: PreferredHost::DocumentCenter,
            persistence_key_policy: descriptor_key,
            default_constraints: PaneConstraints::default(),
        }
    }

    pub fn with_multi_instance(mut self, multi_instance: bool) -> Self {
        self.multi_instance = multi_instance;
        self
    }

    pub fn with_dock_policy(mut self, dock_policy: DockPolicy) -> Self {
        self.dock_policy = dock_policy;
        self
    }

    pub fn with_preferred_drawer_slot(mut self, slot: ActivityDrawerSlot) -> Self {
        self.preferred_drawer_slot = Some(slot);
        self.preferred_host = PreferredHost::Drawer(slot);
        self
    }

    pub fn with_preferred_host(mut self, preferred_host: PreferredHost) -> Self {
        self.preferred_host = preferred_host;
        self
    }

    pub fn with_icon_key(mut self, icon_key: impl Into<String>) -> Self {
        self.icon_key = icon_key.into();
        self
    }

    pub fn with_default_constraints(mut self, constraints: PaneConstraints) -> Self {
        self.default_constraints = constraints;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewInstance {
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
    pub title: String,
    pub serializable_payload: Value,
    pub dirty: bool,
    pub host: ViewHost,
}

#[derive(Clone, Debug, Default)]
pub struct ViewRegistry {
    descriptors: HashMap<ViewDescriptorId, ViewDescriptor>,
    instances: HashMap<ViewInstanceId, ViewInstance>,
    single_instance_index: HashMap<ViewDescriptorId, ViewInstanceId>,
    counters: HashMap<ViewDescriptorId, usize>,
}

impl ViewRegistry {
    pub fn register_view(&mut self, descriptor: ViewDescriptor) -> Result<(), String> {
        if self.descriptors.contains_key(&descriptor.descriptor_id) {
            return Err(format!(
                "view descriptor {} already registered",
                descriptor.descriptor_id.0
            ));
        }
        self.descriptors
            .insert(descriptor.descriptor_id.clone(), descriptor);
        Ok(())
    }

    pub fn descriptor(&self, descriptor_id: &ViewDescriptorId) -> Option<&ViewDescriptor> {
        self.descriptors.get(descriptor_id)
    }

    pub fn list_descriptors(&self) -> Vec<ViewDescriptor> {
        self.descriptors.values().cloned().collect()
    }

    pub fn open_descriptor(
        &mut self,
        descriptor_id: ViewDescriptorId,
    ) -> Result<ViewInstance, String> {
        let descriptor = self
            .descriptors
            .get(&descriptor_id)
            .cloned()
            .ok_or_else(|| format!("missing view descriptor {}", descriptor_id.0))?;

        if !descriptor.multi_instance {
            if let Some(instance_id) = self.single_instance_index.get(&descriptor_id) {
                return self
                    .instances
                    .get(instance_id)
                    .cloned()
                    .ok_or_else(|| "single instance index is stale".to_string());
            }
        }

        let counter = self.counters.entry(descriptor_id.clone()).or_insert(0);
        *counter += 1;
        let instance_id = ViewInstanceId::new(format!("{}#{}", descriptor_id.0, counter));
        let instance = ViewInstance {
            instance_id: instance_id.clone(),
            descriptor_id: descriptor_id.clone(),
            title: descriptor.default_title.clone(),
            serializable_payload: Value::Null,
            dirty: false,
            host: preferred_host_to_view_host(descriptor.preferred_host),
        };

        if !descriptor.multi_instance {
            self.single_instance_index
                .insert(descriptor_id, instance_id.clone());
        }
        self.instances.insert(instance_id, instance.clone());
        Ok(instance)
    }

    pub fn restore_instance(&mut self, instance: ViewInstance) -> Result<ViewInstance, String> {
        let Some(multi_instance) = self
            .descriptors
            .get(&instance.descriptor_id)
            .map(|descriptor| descriptor.multi_instance)
        else {
            return Err(format!(
                "cannot restore missing descriptor {}",
                instance.descriptor_id.0
            ));
        };
        self.update_counter(&instance);
        if !multi_instance {
            self.single_instance_index
                .insert(instance.descriptor_id.clone(), instance.instance_id.clone());
        }
        self.instances
            .insert(instance.instance_id.clone(), instance.clone());
        Ok(instance)
    }

    pub fn instance(&self, instance_id: &ViewInstanceId) -> Option<&ViewInstance> {
        self.instances.get(instance_id)
    }

    pub fn instances(&self) -> Vec<ViewInstance> {
        self.instances.values().cloned().collect()
    }

    pub fn remove_instance(&mut self, instance_id: &ViewInstanceId) -> Option<ViewInstance> {
        let removed = self.instances.remove(instance_id)?;
        if self
            .single_instance_index
            .get(&removed.descriptor_id)
            .is_some_and(|current| current == instance_id)
        {
            self.single_instance_index.remove(&removed.descriptor_id);
        }
        Some(removed)
    }

    pub fn clear_instances(&mut self) {
        self.instances.clear();
        self.single_instance_index.clear();
        self.counters.clear();
    }

    fn update_counter(&mut self, instance: &ViewInstance) {
        let Some((_, suffix)) = instance.instance_id.0.rsplit_once('#') else {
            return;
        };
        let Ok(value) = suffix.parse::<usize>() else {
            return;
        };
        let counter = self
            .counters
            .entry(instance.descriptor_id.clone())
            .or_insert(0);
        *counter = (*counter).max(value);
    }
}

fn preferred_host_to_view_host(preferred_host: PreferredHost) -> ViewHost {
    match preferred_host {
        PreferredHost::Drawer(slot) => ViewHost::Drawer(slot),
        PreferredHost::DocumentCenter => ViewHost::Document(MainPageId::workbench(), vec![]),
        PreferredHost::FloatingWindow => {
            ViewHost::FloatingWindow(MainPageId::new("floating"), vec![])
        }
        PreferredHost::ExclusiveMainPage => ViewHost::ExclusivePage(MainPageId::new("exclusive")),
    }
}
