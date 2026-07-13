use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use zircon_runtime::scene::{EntityId, ReflectComponent, TypeRegistry, World};
use zircon_runtime_interface::reflect::ReflectedValue;

use super::{CallSiteError, CompiledCallSite, ParamLayout};

#[derive(Clone)]
struct CompiledType {
    member_count: usize,
    component: Option<ReflectComponent>,
}

#[derive(Clone, Default)]
pub struct ScriptCallTable {
    types: Arc<Vec<CompiledType>>,
    by_name: Arc<HashMap<(String, String), CompiledCallSite>>,
    resolution_count: Arc<AtomicUsize>,
}

impl ScriptCallTable {
    pub fn compile(registry: &TypeRegistry) -> Result<Self, CallSiteError> {
        let mut types = Vec::new();
        let mut by_name = HashMap::new();
        for (type_index, runtime) in registry.iter().enumerate() {
            let type_slot =
                u32::try_from(type_index).map_err(|_| CallSiteError::SlotCapacityExceeded {
                    slot_kind: "type",
                    count: type_index + 1,
                })?;
            let registration = &runtime.registration;
            for (member_index, field) in registration.type_info.fields.iter().enumerate() {
                let member_slot = u32::try_from(member_index).map_err(|_| {
                    CallSiteError::SlotCapacityExceeded {
                        slot_kind: "member",
                        count: member_index + 1,
                    }
                })?;
                let site = CompiledCallSite::new(
                    type_slot,
                    member_slot,
                    ParamLayout::new(field.value_type_path.as_str(), field.editable),
                );
                by_name.insert(
                    (registration.type_path.type_path.clone(), field.name.clone()),
                    site,
                );
            }
            types.push(CompiledType {
                member_count: registration.type_info.fields.len(),
                component: runtime.component.clone(),
            });
        }
        Ok(Self {
            types: Arc::new(types),
            by_name: Arc::new(by_name),
            resolution_count: Arc::new(AtomicUsize::new(0)),
        })
    }

    pub fn resolve(
        &self,
        type_path: &str,
        member_name: &str,
    ) -> Result<CompiledCallSite, CallSiteError> {
        self.resolution_count.fetch_add(1, Ordering::Relaxed);
        self.by_name
            .get(&(type_path.to_string(), member_name.to_string()))
            .cloned()
            .ok_or_else(|| CallSiteError::UnknownMember {
                type_path: type_path.to_string(),
                member_name: member_name.to_string(),
            })
    }

    pub fn resolution_count(&self) -> usize {
        self.resolution_count.load(Ordering::Relaxed)
    }

    pub fn read(
        &self,
        site: &CompiledCallSite,
        world: &World,
        entity: EntityId,
    ) -> Result<ReflectedValue, CallSiteError> {
        let compiled_type = self.entry(site)?;
        let Some(component) = &compiled_type.component else {
            return Err(CallSiteError::NoComponentAdapter {
                type_slot: site.type_slot,
            });
        };
        component
            .read_field_by_slot(world, entity, site.member_slot)
            .map_err(CallSiteError::from)
    }

    pub fn write(
        &self,
        site: &CompiledCallSite,
        world: &mut World,
        entity: EntityId,
        value: ReflectedValue,
    ) -> Result<bool, CallSiteError> {
        let compiled_type = self.entry(site)?;
        let Some(component) = &compiled_type.component else {
            return Err(CallSiteError::NoComponentAdapter {
                type_slot: site.type_slot,
            });
        };
        component
            .write_field_by_slot(world, entity, site.member_slot, value)
            .map_err(CallSiteError::from)
    }

    fn entry(&self, site: &CompiledCallSite) -> Result<&CompiledType, CallSiteError> {
        let Some(compiled_type) = self.types.get(site.type_slot as usize) else {
            return Err(CallSiteError::InvalidTypeSlot {
                type_slot: site.type_slot,
            });
        };
        if site.member_slot as usize >= compiled_type.member_count {
            return Err(CallSiteError::InvalidMemberSlot {
                type_slot: site.type_slot,
                member_slot: site.member_slot,
            });
        }
        Ok(compiled_type)
    }
}

impl std::fmt::Debug for ScriptCallTable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptCallTable")
            .field("type_count", &self.types.len())
            .field("call_site_count", &self.by_name.len())
            .field("resolution_count", &self.resolution_count())
            .finish()
    }
}
