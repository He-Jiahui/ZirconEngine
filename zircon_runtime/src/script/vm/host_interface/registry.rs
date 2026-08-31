use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::net::RpcPayloadSchema;

use super::super::{CapabilitySet, PluginSlotId, VmPluginSlotRecord, VmPluginSlotState};
use super::{
    VmBehaviorNodeRegistration, VmCallbackHandle, VmEditorOperationRegistration,
    VmHostInterfaceError, VmInterfaceCaller, VmRpcHandlerRegistration, VmSystemRegistration,
    VmSystemStage, VM_BT_NODE_CAPABILITY, VM_EDITOR_OPERATION_CAPABILITY,
    VM_RPC_HANDLER_CAPABILITY, VM_SYSTEM_CAPABILITY,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RegistrationKey {
    slot: PluginSlotId,
    generation: u32,
    id: String,
}

#[derive(Clone, Debug)]
struct CallbackModule {
    name: Arc<str>,
    functions: Vec<Arc<str>>,
    function_slots: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Default)]
struct OwnerCallbackTable {
    modules: Vec<CallbackModule>,
    module_slots: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Default)]
struct RegistryState {
    callbacks: BTreeMap<PluginSlotId, OwnerCallbackTable>,
    systems: BTreeMap<RegistrationKey, VmSystemRegistration>,
    behavior_nodes: BTreeMap<RegistrationKey, VmBehaviorNodeRegistration>,
    rpc_handlers: BTreeMap<RegistrationKey, VmRpcHandlerRegistration>,
    editor_operations: BTreeMap<RegistrationKey, VmEditorOperationRegistration>,
    active_generations: BTreeMap<PluginSlotId, u32>,
    active_package_slots: BTreeMap<String, Arc<[PluginSlotId]>>,
    active_capabilities: BTreeMap<PluginSlotId, CapabilitySet>,
    active_snapshot: Arc<VmHostInterfaceActiveSnapshot>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VmHostInterfaceGenerationSnapshot {
    callbacks: Option<OwnerCallbackTable>,
    systems: BTreeMap<RegistrationKey, VmSystemRegistration>,
    behavior_nodes: BTreeMap<RegistrationKey, VmBehaviorNodeRegistration>,
    rpc_handlers: BTreeMap<RegistrationKey, VmRpcHandlerRegistration>,
    editor_operations: BTreeMap<RegistrationKey, VmEditorOperationRegistration>,
}

#[derive(Clone, Debug)]
pub(crate) struct VmHostInterfaceActiveOwner {
    pub(crate) slot: PluginSlotId,
    pub(crate) generation: u32,
    pub(crate) package_name: String,
    pub(crate) capabilities: CapabilitySet,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VmHostInterfaceActiveSnapshot {
    active_generations: BTreeMap<PluginSlotId, u32>,
    package_slots: BTreeMap<String, Arc<[PluginSlotId]>>,
    capabilities: BTreeMap<PluginSlotId, CapabilitySet>,
    fixed_update_systems: Arc<[VmSystemRegistration]>,
    update_systems: Arc<[VmSystemRegistration]>,
    last_systems: Arc<[VmSystemRegistration]>,
    behavior_nodes: Arc<[VmBehaviorNodeRegistration]>,
    rpc_handlers: Arc<[VmRpcHandlerRegistration]>,
    editor_operations: Arc<[VmEditorOperationRegistration]>,
}

impl VmHostInterfaceActiveSnapshot {
    fn systems(&self, stage: VmSystemStage) -> Arc<[VmSystemRegistration]> {
        match stage {
            VmSystemStage::FixedUpdate => Arc::clone(&self.fixed_update_systems),
            VmSystemStage::Update => Arc::clone(&self.update_systems),
            VmSystemStage::Last => Arc::clone(&self.last_systems),
        }
    }
}

/// Shared registry for capability-gated VM extension descriptors and dense callback targets.
#[derive(Clone, Debug, Default)]
pub struct VmHostInterfaceRegistry {
    state: Arc<Mutex<RegistryState>>,
}

impl VmHostInterfaceRegistry {
    fn lock_state(&self) -> MutexGuard<'_, RegistryState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Interns a dense callback target without publishing it on an extension channel.
    ///
    /// Scene-owned callbacks use this path because their lifecycle is controlled by the
    /// scene projection rather than by a script-visible interface registration.
    pub fn intern_callback(
        &self,
        caller: &VmInterfaceCaller,
        module: &str,
        function: &str,
    ) -> Result<VmCallbackHandle, VmHostInterfaceError> {
        let mut state = self.lock_state();
        compile_callback(&mut state, caller, module, function)
    }

    /// Registers a VM callback for conservative scheduled execution.
    pub fn register_system(
        &self,
        caller: &VmInterfaceCaller,
        id: impl Into<String>,
        stage: VmSystemStage,
        module: &str,
        function: &str,
    ) -> Result<VmCallbackHandle, VmHostInterfaceError> {
        require_capability(caller, "system", VM_SYSTEM_CAPABILITY)?;
        let id = validate_identifier("system id", id.into())?;
        let mut state = self.lock_state();
        let callback = compile_callback(&mut state, caller, module, function)?;
        let key = registration_key(caller, id.clone());
        insert_registration(
            &mut state.systems,
            key,
            VmSystemRegistration {
                id,
                stage,
                callback,
            },
            "system",
        )?;
        rebuild_if_active_generation(&mut state, caller);
        Ok(callback)
    }

    /// Registers a VM callback as a behavior-tree node contribution.
    pub fn register_behavior_node(
        &self,
        caller: &VmInterfaceCaller,
        id: impl Into<String>,
        display_name: impl Into<String>,
        module: &str,
        function: &str,
    ) -> Result<VmCallbackHandle, VmHostInterfaceError> {
        require_capability(caller, "behavior node", VM_BT_NODE_CAPABILITY)?;
        let id = validate_identifier("behavior node id", id.into())?;
        let display_name = validate_identifier("behavior node display name", display_name.into())?;
        let mut state = self.lock_state();
        let callback = compile_callback(&mut state, caller, module, function)?;
        let key = registration_key(caller, id.clone());
        insert_registration(
            &mut state.behavior_nodes,
            key,
            VmBehaviorNodeRegistration {
                id,
                display_name,
                callback,
            },
            "behavior node",
        )?;
        rebuild_if_active_generation(&mut state, caller);
        Ok(callback)
    }

    /// Registers a VM callback as an RPC handler contribution.
    pub fn register_rpc_handler(
        &self,
        caller: &VmInterfaceCaller,
        id: impl Into<String>,
        payload_schema: RpcPayloadSchema,
        module: &str,
        function: &str,
    ) -> Result<VmCallbackHandle, VmHostInterfaceError> {
        require_capability(caller, "RPC handler", VM_RPC_HANDLER_CAPABILITY)?;
        let id = validate_identifier("RPC handler id", id.into())?;
        validate_identifier("RPC payload schema", payload_schema.schema_id.clone())?;
        let mut state = self.lock_state();
        let callback = compile_callback(&mut state, caller, module, function)?;
        let key = registration_key(caller, id.clone());
        insert_registration(
            &mut state.rpc_handlers,
            key,
            VmRpcHandlerRegistration {
                id,
                payload_schema,
                callback,
            },
            "RPC handler",
        )?;
        rebuild_if_active_generation(&mut state, caller);
        Ok(callback)
    }

    /// Registers a VM callback as a three-segment editor operation.
    pub fn register_editor_operation(
        &self,
        caller: &VmInterfaceCaller,
        operation: impl Into<String>,
        module: &str,
        function: &str,
    ) -> Result<VmCallbackHandle, VmHostInterfaceError> {
        require_capability(caller, "editor operation", VM_EDITOR_OPERATION_CAPABILITY)?;
        let operation = validate_editor_operation(operation.into())?;
        let mut state = self.lock_state();
        let callback = compile_callback(&mut state, caller, module, function)?;
        let key = registration_key(caller, operation.clone());
        insert_registration(
            &mut state.editor_operations,
            key,
            VmEditorOperationRegistration {
                operation,
                callback,
            },
            "editor operation",
        )?;
        rebuild_if_active_generation(&mut state, caller);
        Ok(callback)
    }

    /// Resolves a dense callback target and refreshes a stale handle generation.
    pub fn resolve_callback(
        &self,
        handle: &mut VmCallbackHandle,
        active_generation: u32,
    ) -> Result<(Arc<str>, Arc<str>), VmHostInterfaceError> {
        let state = self.lock_state();
        let owner = state.callbacks.get(&handle.slot).ok_or(
            VmHostInterfaceError::MissingCallbackTarget {
                slot: handle.slot,
                module: handle.module,
                function: handle.function,
            },
        )?;
        let module = owner.modules.get(handle.module as usize).ok_or(
            VmHostInterfaceError::MissingCallbackTarget {
                slot: handle.slot,
                module: handle.module,
                function: handle.function,
            },
        )?;
        let function = module.functions.get(handle.function as usize).ok_or(
            VmHostInterfaceError::MissingCallbackTarget {
                slot: handle.slot,
                module: handle.module,
                function: handle.function,
            },
        )?;
        handle.generation = active_generation;
        Ok((module.name.clone(), function.clone()))
    }

    /// Returns the latest active VM systems for `stage` in deterministic order.
    pub fn systems(
        &self,
        slots: &[VmPluginSlotRecord],
        stage: VmSystemStage,
    ) -> Vec<VmSystemRegistration> {
        let active = slot_active_generations(slots);
        latest_active(&self.lock_state().systems, &active, |registration| {
            registration.stage == stage
        })
    }

    pub(crate) fn active_systems(&self, stage: VmSystemStage) -> Vec<VmSystemRegistration> {
        self.systems_snapshot(stage).as_ref().to_vec()
    }

    /// Returns the latest active behavior-node contributions in deterministic order.
    pub fn behavior_nodes(&self, slots: &[VmPluginSlotRecord]) -> Vec<VmBehaviorNodeRegistration> {
        let active = slot_active_generations(slots);
        latest_active(&self.lock_state().behavior_nodes, &active, |_| true)
    }

    pub(crate) fn active_behavior_nodes(&self) -> Vec<VmBehaviorNodeRegistration> {
        self.lock_state().active_snapshot.behavior_nodes.to_vec()
    }

    /// Returns the latest active RPC-handler contributions in deterministic order.
    pub fn rpc_handlers(&self, slots: &[VmPluginSlotRecord]) -> Vec<VmRpcHandlerRegistration> {
        let active = slot_active_generations(slots);
        latest_active(&self.lock_state().rpc_handlers, &active, |_| true)
    }

    pub(crate) fn active_rpc_handlers(&self) -> Vec<VmRpcHandlerRegistration> {
        self.lock_state().active_snapshot.rpc_handlers.to_vec()
    }

    /// Returns the latest active editor-operation contributions in deterministic order.
    pub fn editor_operations(
        &self,
        slots: &[VmPluginSlotRecord],
    ) -> Vec<VmEditorOperationRegistration> {
        let active = slot_active_generations(slots);
        latest_active(&self.lock_state().editor_operations, &active, |_| true)
    }

    pub(crate) fn active_editor_operations(&self) -> Vec<VmEditorOperationRegistration> {
        self.lock_state().active_snapshot.editor_operations.to_vec()
    }

    pub(crate) fn systems_snapshot(&self, stage: VmSystemStage) -> Arc<[VmSystemRegistration]> {
        self.lock_state().active_snapshot.systems(stage)
    }

    #[cfg(test)]
    pub(crate) fn active_snapshot(&self) -> Arc<VmHostInterfaceActiveSnapshot> {
        Arc::clone(&self.lock_state().active_snapshot)
    }

    pub(crate) fn publish_active_slots(&self, active_slots: Vec<VmHostInterfaceActiveOwner>) {
        let mut state = self.lock_state();
        let mut active_generations = BTreeMap::new();
        let mut package_slots = BTreeMap::<String, Vec<PluginSlotId>>::new();
        let mut active_capabilities = BTreeMap::new();
        for active in active_slots {
            active_generations.insert(active.slot, active.generation);
            package_slots
                .entry(active.package_name)
                .or_default()
                .push(active.slot);
            active_capabilities.insert(active.slot, active.capabilities);
        }
        state.active_generations = active_generations;
        state.active_package_slots = package_slots
            .into_iter()
            .map(|(package, slots)| (package, slots.into()))
            .collect();
        state.active_capabilities = active_capabilities;
        rebuild_active_snapshot(&mut state);
    }

    pub(crate) fn active_slots_for_package(
        &self,
        package_name: &str,
    ) -> Option<Arc<[PluginSlotId]>> {
        self.lock_state()
            .active_snapshot
            .package_slots
            .get(package_name)
            .cloned()
    }

    pub(crate) fn active_owner(&self, slot: PluginSlotId) -> Option<(u32, CapabilitySet)> {
        let state = self.lock_state();
        let snapshot = &state.active_snapshot;
        Some((
            *snapshot.active_generations.get(&slot)?,
            snapshot.capabilities.get(&slot)?.clone(),
        ))
    }

    pub(crate) fn active_generation(&self, slot: PluginSlotId) -> Option<u32> {
        self.lock_state()
            .active_snapshot
            .active_generations
            .get(&slot)
            .copied()
    }

    /// Captures one owner generation so a failed reload can restore its exact registrations.
    pub(crate) fn snapshot_generation(
        &self,
        slot: PluginSlotId,
        generation: u32,
    ) -> VmHostInterfaceGenerationSnapshot {
        let state = self.lock_state();
        VmHostInterfaceGenerationSnapshot {
            callbacks: state.callbacks.get(&slot).cloned(),
            systems: generation_entries(&state.systems, slot, generation),
            behavior_nodes: generation_entries(&state.behavior_nodes, slot, generation),
            rpc_handlers: generation_entries(&state.rpc_handlers, slot, generation),
            editor_operations: generation_entries(&state.editor_operations, slot, generation),
        }
    }

    /// Replaces one owner generation and its callback table with a pre-reload snapshot.
    pub(crate) fn restore_generation(
        &self,
        slot: PluginSlotId,
        generation: u32,
        snapshot: VmHostInterfaceGenerationSnapshot,
    ) {
        let mut state = self.lock_state();
        discard_generation_entries(&mut state.systems, slot, generation);
        discard_generation_entries(&mut state.behavior_nodes, slot, generation);
        discard_generation_entries(&mut state.rpc_handlers, slot, generation);
        discard_generation_entries(&mut state.editor_operations, slot, generation);
        state.systems.extend(snapshot.systems);
        state.behavior_nodes.extend(snapshot.behavior_nodes);
        state.rpc_handlers.extend(snapshot.rpc_handlers);
        state.editor_operations.extend(snapshot.editor_operations);
        match snapshot.callbacks {
            Some(callbacks) => {
                state.callbacks.insert(slot, callbacks);
            }
            None => {
                state.callbacks.remove(&slot);
            }
        }
        rebuild_active_snapshot(&mut state);
    }

    /// Removes all descriptors published by one failed package generation.
    pub fn discard_generation(&self, slot: PluginSlotId, generation: u32) {
        let mut state = self.lock_state();
        discard_generation_entries(&mut state.systems, slot, generation);
        discard_generation_entries(&mut state.behavior_nodes, slot, generation);
        discard_generation_entries(&mut state.rpc_handlers, slot, generation);
        discard_generation_entries(&mut state.editor_operations, slot, generation);
        rebuild_active_snapshot(&mut state);
    }

    /// Removes callback targets and descriptors owned by an unloaded package slot.
    pub fn discard_slot(&self, slot: PluginSlotId) {
        let mut state = self.lock_state();
        state.callbacks.remove(&slot);
        state.systems.retain(|key, _| key.slot != slot);
        state.behavior_nodes.retain(|key, _| key.slot != slot);
        state.rpc_handlers.retain(|key, _| key.slot != slot);
        state.editor_operations.retain(|key, _| key.slot != slot);
        rebuild_active_snapshot(&mut state);
    }
}

fn rebuild_active_snapshot(state: &mut RegistryState) {
    let systems = latest_active(&state.systems, &state.active_generations, |_| true);
    let mut fixed_update_systems = Vec::new();
    let mut update_systems = Vec::new();
    let mut last_systems = Vec::new();
    for system in systems {
        match system.stage {
            VmSystemStage::FixedUpdate => fixed_update_systems.push(system),
            VmSystemStage::Update => update_systems.push(system),
            VmSystemStage::Last => last_systems.push(system),
        }
    }
    state.active_snapshot = Arc::new(VmHostInterfaceActiveSnapshot {
        active_generations: state.active_generations.clone(),
        package_slots: state.active_package_slots.clone(),
        capabilities: state.active_capabilities.clone(),
        fixed_update_systems: fixed_update_systems.into(),
        update_systems: update_systems.into(),
        last_systems: last_systems.into(),
        behavior_nodes: latest_active(&state.behavior_nodes, &state.active_generations, |_| true)
            .into(),
        rpc_handlers: latest_active(&state.rpc_handlers, &state.active_generations, |_| true)
            .into(),
        editor_operations: latest_active(
            &state.editor_operations,
            &state.active_generations,
            |_| true,
        )
        .into(),
    });
}

fn rebuild_if_active_generation(state: &mut RegistryState, caller: &VmInterfaceCaller) {
    if state.active_generations.get(&caller.slot) == Some(&caller.generation) {
        rebuild_active_snapshot(state);
    }
}

fn generation_entries<T: Clone>(
    registrations: &BTreeMap<RegistrationKey, T>,
    slot: PluginSlotId,
    generation: u32,
) -> BTreeMap<RegistrationKey, T> {
    registrations
        .iter()
        .filter(|(key, _)| key.slot == slot && key.generation == generation)
        .map(|(key, registration)| (key.clone(), registration.clone()))
        .collect()
}

fn discard_generation_entries<T>(
    registrations: &mut BTreeMap<RegistrationKey, T>,
    slot: PluginSlotId,
    generation: u32,
) {
    registrations.retain(|key, _| key.slot != slot || key.generation != generation);
}

fn compile_callback(
    state: &mut RegistryState,
    caller: &VmInterfaceCaller,
    module: &str,
    function: &str,
) -> Result<VmCallbackHandle, VmHostInterfaceError> {
    let module = validate_identifier("callback module", module.to_string())?;
    let function = validate_identifier("callback function", function.to_string())?;
    let owner = state.callbacks.entry(caller.slot).or_default();
    let module_slot = match owner.module_slots.get(&module).copied() {
        Some(slot) => slot,
        None => {
            let slot = u32::try_from(owner.modules.len())
                .map_err(|_| VmHostInterfaceError::CallbackTableExhausted("module"))?;
            owner.module_slots.insert(module.clone(), slot);
            owner.modules.push(CallbackModule {
                name: module.into(),
                functions: Vec::new(),
                function_slots: BTreeMap::new(),
            });
            slot
        }
    };
    let callback_module = &mut owner.modules[module_slot as usize];
    let function_slot = match callback_module.function_slots.get(&function).copied() {
        Some(slot) => slot,
        None => {
            let slot = u32::try_from(callback_module.functions.len())
                .map_err(|_| VmHostInterfaceError::CallbackTableExhausted("function"))?;
            callback_module
                .function_slots
                .insert(function.clone(), slot);
            callback_module.functions.push(function.into());
            slot
        }
    };
    Ok(VmCallbackHandle {
        slot: caller.slot,
        module: module_slot,
        function: function_slot,
        generation: caller.generation,
    })
}

fn registration_key(caller: &VmInterfaceCaller, id: String) -> RegistrationKey {
    RegistrationKey {
        slot: caller.slot,
        generation: caller.generation,
        id,
    }
}

fn insert_registration<T>(
    registrations: &mut BTreeMap<RegistrationKey, T>,
    key: RegistrationKey,
    registration: T,
    channel: &'static str,
) -> Result<(), VmHostInterfaceError> {
    if registrations.contains_key(&key) {
        return Err(VmHostInterfaceError::DuplicateRegistration {
            channel,
            id: key.id,
            slot: key.slot,
            generation: key.generation,
        });
    }
    registrations.insert(key, registration);
    Ok(())
}

fn require_capability(
    caller: &VmInterfaceCaller,
    channel: &'static str,
    required: &'static str,
) -> Result<(), VmHostInterfaceError> {
    if caller.capabilities.contains(required) {
        Ok(())
    } else {
        Err(VmHostInterfaceError::CapabilityDenied { channel, required })
    }
}

fn validate_identifier(label: &'static str, value: String) -> Result<String, VmHostInterfaceError> {
    if value.is_empty() || value.trim() != value {
        Err(VmHostInterfaceError::InvalidIdentifier { label, value })
    } else {
        Ok(value)
    }
}

fn validate_editor_operation(value: String) -> Result<String, VmHostInterfaceError> {
    let value = validate_identifier("editor operation", value)?;
    if !has_exact_editor_operation_segments(&value) {
        return Err(VmHostInterfaceError::InvalidIdentifier {
            label: "editor operation (expected XXX.YYY.ZZZ)",
            value,
        });
    }
    Ok(value)
}

fn has_exact_editor_operation_segments(value: &str) -> bool {
    let mut segments = value.split('.');
    segments.next().is_some_and(|segment| !segment.is_empty())
        && segments.next().is_some_and(|segment| !segment.is_empty())
        && segments.next().is_some_and(|segment| !segment.is_empty())
        && segments.next().is_none()
}

fn slot_active_generations(slots: &[VmPluginSlotRecord]) -> BTreeMap<PluginSlotId, u32> {
    slots
        .iter()
        .filter(|slot| slot.state == VmPluginSlotState::Active)
        .map(|slot| (slot.slot, slot.generation))
        .collect()
}

fn latest_active<T: Clone>(
    registrations: &BTreeMap<RegistrationKey, T>,
    active: &BTreeMap<PluginSlotId, u32>,
    include: impl Fn(&T) -> bool,
) -> Vec<T> {
    let mut selected: HashMap<(PluginSlotId, &str), (u32, T)> = HashMap::new();
    for (key, registration) in registrations {
        let Some(active_generation) = active.get(&key.slot).copied() else {
            continue;
        };
        if key.generation > active_generation || !include(registration) {
            continue;
        }
        let identity = (key.slot, key.id.as_str());
        match selected.get(&identity) {
            Some((generation, _)) if *generation >= key.generation => {}
            _ => {
                selected.insert(identity, (key.generation, registration.clone()));
            }
        }
    }
    let mut selected = selected.into_iter().collect::<Vec<_>>();
    selected.sort_by(|left, right| left.0.cmp(&right.0));
    selected
        .into_iter()
        .map(|(_, (_, registration))| registration)
        .collect()
}

#[cfg(test)]
#[path = "registry/editor_operation_segments_tests.rs"]
mod editor_operation_segments_tests;
