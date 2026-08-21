use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use zircon_runtime::scene::{EntityId, ReflectComponent, TypeRegistry, World};
use zircon_runtime::script::VmReflectionRegistrySnapshot;
use zircon_runtime_interface::reflect::{ReflectScriptVisibility, ReflectedValue};

use super::{CallSiteError, CompiledCallSite, ParamLayout};

static NEXT_CALL_TABLE_GENERATION: AtomicU32 = AtomicU32::new(1);

#[derive(Clone)]
struct CompiledType {
    members: Vec<ParamLayout>,
    component: Option<ReflectComponent>,
}

/// Immutable package-local table that removes string lookup from reflected runtime calls.
#[derive(Clone, Default)]
pub struct ScriptCallTable {
    types: Arc<Vec<CompiledType>>,
    by_name: Arc<HashMap<(String, String), CompiledCallSite>>,
    sites: Arc<Vec<CompiledCallSite>>,
    table_generation: u32,
    catalog_snapshot: Option<Arc<VmReflectionRegistrySnapshot>>,
    resolution_count: Arc<AtomicUsize>,
}

impl ScriptCallTable {
    /// Compiles only script-public reflected types into deterministic dense slots.
    pub fn compile(registry: &TypeRegistry) -> Result<Self, CallSiteError> {
        Self::compile_inner(registry, None)
    }

    /// Compiles a table that fails closed after its catalog revision is replaced.
    pub fn compile_snapshot(
        snapshot: &VmReflectionRegistrySnapshot,
    ) -> Result<Self, CallSiteError> {
        Self::compile_inner(snapshot.registry(), Some(Arc::new(snapshot.clone())))
    }

    fn compile_inner(
        registry: &TypeRegistry,
        catalog_snapshot: Option<Arc<VmReflectionRegistrySnapshot>>,
    ) -> Result<Self, CallSiteError> {
        let table_generation = allocate_call_table_generation()?;
        let mut types = Vec::new();
        let mut by_name = HashMap::new();
        let mut sites = Vec::new();
        for (type_index, runtime) in registry
            .iter()
            .filter(|runtime| {
                runtime.registration.script_visibility == ReflectScriptVisibility::Public
            })
            .enumerate()
        {
            let type_slot =
                u32::try_from(type_index).map_err(|_| CallSiteError::SlotCapacityExceeded {
                    slot_kind: "type",
                    count: type_index + 1,
                })?;
            let registration = &runtime.registration;
            let mut members = Vec::with_capacity(registration.type_info.fields.len());
            for (member_index, field) in registration.type_info.fields.iter().enumerate() {
                let member_slot = u32::try_from(member_index).map_err(|_| {
                    CallSiteError::SlotCapacityExceeded {
                        slot_kind: "member",
                        count: member_index + 1,
                    }
                })?;
                let layout = ParamLayout::new(field.value_type_path.as_str(), field.editable);
                let token = encode_call_site_token(table_generation, sites.len())?;
                let site = CompiledCallSite::new(token, type_slot, member_slot, layout.clone());
                members.push(layout);
                by_name.insert(
                    (registration.type_path.type_path.clone(), field.name.clone()),
                    site.clone(),
                );
                sites.push(site);
            }
            types.push(CompiledType {
                members,
                component: runtime.component.clone(),
            });
        }
        Ok(Self {
            types: Arc::new(types),
            by_name: Arc::new(by_name),
            sites: Arc::new(sites),
            table_generation,
            catalog_snapshot,
            resolution_count: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Resolves one public type/member pair during package loading.
    pub fn resolve(
        &self,
        type_path: &str,
        member_name: &str,
    ) -> Result<CompiledCallSite, CallSiteError> {
        self.ensure_catalog_resolution_allowed()?;
        self.resolution_count.fetch_add(1, Ordering::Relaxed);
        self.by_name
            .get(&(type_path.to_string(), member_name.to_string()))
            .cloned()
            .ok_or_else(|| CallSiteError::UnknownMember {
                type_path: type_path.to_string(),
                member_name: member_name.to_string(),
            })
    }

    /// Returns the number of name resolutions performed by this table.
    pub fn resolution_count(&self) -> usize {
        self.resolution_count.load(Ordering::Relaxed)
    }

    /// Returns the guarded catalog revision, when this is a production snapshot table.
    pub fn catalog_revision(&self) -> Option<u64> {
        self.catalog_snapshot
            .as_ref()
            .map(|snapshot| snapshot.revision())
    }

    /// Returns whether this table can still dispatch against the process-wide catalog.
    pub fn is_catalog_current(&self) -> bool {
        self.catalog_snapshot
            .as_ref()
            .map(|snapshot| snapshot.is_current())
            .unwrap_or(true)
    }

    /// Returns whether this exact current or prepared table may resolve package-loading names.
    pub fn can_resolve_names(&self) -> bool {
        self.catalog_snapshot
            .as_ref()
            .map(|snapshot| snapshot.can_resolve_names())
            .unwrap_or(true)
    }

    /// Reads a reflected field through a previously compiled call site.
    pub fn read(
        &self,
        site: &CompiledCallSite,
        world: &World,
        entity: EntityId,
    ) -> Result<ReflectedValue, CallSiteError> {
        self.ensure_catalog_revision()?;
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

    /// Writes a reflected field through a previously compiled call site.
    pub fn write(
        &self,
        site: &CompiledCallSite,
        world: &mut World,
        entity: EntityId,
        value: ReflectedValue,
    ) -> Result<bool, CallSiteError> {
        self.ensure_catalog_revision()?;
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

    /// Reads through the opaque numeric token stored by a VM package.
    pub fn read_token(
        &self,
        token: u64,
        world: &World,
        entity: EntityId,
    ) -> Result<ReflectedValue, CallSiteError> {
        let site = self.site_from_token(token)?;
        self.read(&site, world, entity)
    }

    /// Writes through the opaque numeric token stored by a VM package.
    pub fn write_token(
        &self,
        token: u64,
        world: &mut World,
        entity: EntityId,
        value: ReflectedValue,
    ) -> Result<bool, CallSiteError> {
        let site = self.site_from_token(token)?;
        self.write(&site, world, entity, value)
    }

    fn entry(&self, site: &CompiledCallSite) -> Result<&CompiledType, CallSiteError> {
        if self.site_ref_from_token(site.token())? != site {
            return Err(CallSiteError::InvalidToken {
                token: site.token(),
            });
        }
        let Some(compiled_type) = self.types.get(site.type_slot as usize) else {
            return Err(CallSiteError::InvalidTypeSlot {
                type_slot: site.type_slot,
            });
        };
        if site.member_slot as usize >= compiled_type.members.len() {
            return Err(CallSiteError::InvalidMemberSlot {
                type_slot: site.type_slot,
                member_slot: site.member_slot,
            });
        }
        Ok(compiled_type)
    }

    fn site_from_token(&self, token: u64) -> Result<CompiledCallSite, CallSiteError> {
        self.site_ref_from_token(token).cloned()
    }

    fn site_ref_from_token(&self, token: u64) -> Result<&CompiledCallSite, CallSiteError> {
        let generation = (token >> 32) as u32;
        let ordinal = token as u32;
        if generation != self.table_generation || ordinal == 0 {
            return Err(CallSiteError::InvalidToken { token });
        }
        self.sites
            .get(ordinal.saturating_sub(1) as usize)
            .filter(|site| site.token() == token)
            .ok_or(CallSiteError::InvalidToken { token })
    }

    #[cfg(test)]
    pub(super) fn resolve_token_for_test(
        &self,
        token: u64,
    ) -> Result<CompiledCallSite, CallSiteError> {
        self.site_from_token(token)
    }

    fn ensure_catalog_revision(&self) -> Result<(), CallSiteError> {
        let Some(snapshot) = &self.catalog_snapshot else {
            return Ok(());
        };
        if snapshot.is_current() {
            return Ok(());
        }
        Err(CallSiteError::StaleCatalogRevision {
            compiled_revision: snapshot.revision(),
            current_revision: snapshot.current_revision(),
        })
    }

    fn ensure_catalog_resolution_allowed(&self) -> Result<(), CallSiteError> {
        let Some(snapshot) = &self.catalog_snapshot else {
            return Ok(());
        };
        if snapshot.can_resolve_names() {
            return Ok(());
        }
        Err(CallSiteError::StaleCatalogRevision {
            compiled_revision: snapshot.revision(),
            current_revision: snapshot.current_revision(),
        })
    }
}

impl std::fmt::Debug for ScriptCallTable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptCallTable")
            .field("type_count", &self.types.len())
            .field("call_site_count", &self.sites.len())
            .field("table_generation", &self.table_generation)
            .field(
                "catalog_revision",
                &self
                    .catalog_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.revision()),
            )
            .field("resolution_count", &self.resolution_count())
            .finish()
    }
}

fn allocate_call_table_generation() -> Result<u32, CallSiteError> {
    NEXT_CALL_TABLE_GENERATION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| CallSiteError::TokenCapacityExceeded)
}

fn encode_call_site_token(
    table_generation: u32,
    zero_based_ordinal: usize,
) -> Result<u64, CallSiteError> {
    let count = zero_based_ordinal
        .checked_add(1)
        .ok_or(CallSiteError::SlotCapacityExceeded {
            slot_kind: "token ordinal",
            count: usize::MAX,
        })?;
    let ordinal = u32::try_from(count).map_err(|_| CallSiteError::SlotCapacityExceeded {
        slot_kind: "token ordinal",
        count,
    })?;
    Ok((u64::from(table_generation) << 32) | u64::from(ordinal))
}
