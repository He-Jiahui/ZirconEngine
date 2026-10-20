use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::{
    ResourceId, ResourceKind, ResourceLocator, ResourceRecord, ResourceRegistryError,
    ResourceResult,
};

/// Process-local concrete payload identity. It is neither serialized nor a replacement for kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourcePayloadType(TypeId);

impl ResourcePayloadType {
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }

    pub(crate) fn from_data(data: &dyn crate::ResourceData) -> Self {
        Self(data.as_any().type_id())
    }
}

/// Content fields that authorize a payload declaration; publication state and locator moves
/// do not change the concrete Rust type. Dependency resolution likewise does not erase it.
pub(crate) fn same_payload_content(a: &ResourceRecord, b: &ResourceRecord) -> bool {
    a.id == b.id
        && a.kind == b.kind
        && a.artifact_locator == b.artifact_locator
        && a.source_hash == b.source_hash
        && a.importer_id == b.importer_id
        && a.importer_version == b.importer_version
        && a.config_hash == b.config_hash
}

/// 目录的不可变读取快照；克隆共享索引，后续写入以写时复制隔离已有快照，不携带运行时载荷。
#[derive(Clone, Debug, Default)]
pub struct ResourceRegistry {
    by_id: Arc<HashMap<ResourceId, Arc<ResourceRecord>>>,
    id_by_locator: Arc<HashMap<Arc<ResourceLocator>, ResourceId>>,
    payload_types: Arc<HashMap<ResourceId, ResourcePayloadType>>,
}

impl ResourceRegistry {
    // 仅供已完成定位符与身份预检的提交/离线构建调用；绕过预检可能破坏两个索引的一致归属。
    pub(crate) fn insert_unchecked(&mut self, record: ResourceRecord) -> Option<ResourceRecord> {
        debug_assert!(
            self.id_by_locator
                .get(&record.primary_locator)
                .is_none_or(|existing_id| *existing_id == record.id),
            "resource mutation preflight must reject occupied locators"
        );
        if self
            .by_id
            .get(&record.id)
            .is_none_or(|previous| !same_payload_content(previous, &record))
        {
            Arc::make_mut(&mut self.payload_types).remove(&record.id);
        }
        if let Some(existing_locator) = self
            .by_id
            .get(&record.id)
            .map(|existing| existing.primary_locator.clone())
        {
            Arc::make_mut(&mut self.id_by_locator).remove(&existing_locator);
        }

        Arc::make_mut(&mut self.id_by_locator)
            .insert(Arc::new(record.primary_locator.clone()), record.id);
        Arc::make_mut(&mut self.by_id)
            .insert(record.id, Arc::new(record))
            .map(into_owned_record)
    }

    /// Reads only the declaration published with this catalog snapshot; never loads a payload.
    pub fn payload_type(&self, id: ResourceId) -> Option<ResourcePayloadType> {
        self.get(id)?;
        self.payload_types.get(&id).copied()
    }

    pub fn contains_payload<T: 'static>(&self, id: ResourceId) -> bool {
        self.payload_type(id) == Some(ResourcePayloadType::of::<T>())
    }

    pub(crate) fn set_payload_type(
        &mut self,
        id: ResourceId,
        payload_type: Option<ResourcePayloadType>,
    ) {
        if let Some(payload_type) = payload_type.filter(|_| self.by_id.contains_key(&id)) {
            Arc::make_mut(&mut self.payload_types).insert(id, payload_type);
        } else {
            Arc::make_mut(&mut self.payload_types).remove(&id);
        }
    }

    pub fn get(&self, id: ResourceId) -> Option<&ResourceRecord> {
        self.by_id.get(&id).map(Arc::as_ref)
    }

    pub fn get_by_locator(&self, locator: &ResourceLocator) -> Option<&ResourceRecord> {
        self.id_by_locator
            .get(locator)
            .and_then(|id| self.by_id.get(id))
            .map(Arc::as_ref)
    }

    pub(crate) fn id_for_locator(&self, locator: &ResourceLocator) -> Option<ResourceId> {
        self.id_by_locator.get(locator).copied()
    }

    pub fn values(&self) -> impl Iterator<Item = &ResourceRecord> {
        self.by_id.values().map(Arc::as_ref)
    }

    pub(crate) fn remove_by_id(&mut self, id: ResourceId) -> Option<ResourceRecord> {
        let removed = Arc::make_mut(&mut self.by_id).remove(&id)?;
        Arc::make_mut(&mut self.payload_types).remove(&id);
        Arc::make_mut(&mut self.id_by_locator).remove(&removed.primary_locator);
        Some(into_owned_record(removed))
    }

    pub(crate) fn begin_staging(&self) -> ResourceRegistryStaging {
        ResourceRegistryStaging {
            registry: self.clone(),
            identity_registry: self.clone(),
            staged_identities: HashMap::new(),
        }
    }
}

fn into_owned_record(record: Arc<ResourceRecord>) -> ResourceRecord {
    Arc::try_unwrap(record).unwrap_or_else(|record| (*record).clone())
}

#[derive(Clone, Debug)]
struct ResourceRegistryIdentity {
    kind: ResourceKind,
    authorized_locator: ResourceLocator,
}

/// Offline catalog builder. Live runtime mutation remains exclusive to ResourceManager::commit.
#[derive(Clone, Debug, Default)]
pub struct ResourceRegistryStaging {
    registry: ResourceRegistry,
    identity_registry: ResourceRegistry,
    staged_identities: HashMap<ResourceId, ResourceRegistryIdentity>,
}

impl ResourceRegistryStaging {
    /// 构建离线候选目录；同一 ID 已建立的种类与定位符必须保持一致，定位符变化须先显式迁移。
    pub fn stage_record(
        &mut self,
        record: ResourceRecord,
    ) -> ResourceResult<Option<ResourceRecord>> {
        if let Some(existing_id) = self.registry.id_for_locator(&record.primary_locator) {
            if existing_id != record.id {
                return Err(ResourceRegistryError::LocatorOccupied {
                    locator: record.primary_locator.to_string(),
                    existing_id: existing_id.to_string(),
                    requested_id: record.id.to_string(),
                });
            }
        }
        if let Some((kind, authorized_locator)) = self.identity_for(record.id) {
            if kind != record.kind {
                return Err(ResourceRegistryError::KindConflict {
                    id: record.id.to_string(),
                    current_kind: kind,
                    requested_kind: record.kind,
                });
            }
            if authorized_locator != &record.primary_locator {
                return Err(ResourceRegistryError::ExplicitRenameRequired {
                    id: record.id.to_string(),
                    current_locator: authorized_locator.to_string(),
                    requested_locator: record.primary_locator.to_string(),
                });
            }
        }
        if self.identity_registry.get(record.id).is_none() {
            self.staged_identities
                .entry(record.id)
                .or_insert_with(|| ResourceRegistryIdentity {
                    kind: record.kind,
                    authorized_locator: record.primary_locator.clone(),
                });
        }
        Ok(self.registry.insert_unchecked(record))
    }

    /// Attaches an importer-proven concrete type to an already staged row.
    pub fn stage_payload_type(
        &mut self,
        id: ResourceId,
        payload_type: ResourcePayloadType,
    ) -> ResourceResult<()> {
        if self.registry.get(id).is_none() {
            return Err(ResourceRegistryError::MissingRecordForId { id: id.to_string() });
        }
        if self
            .registry
            .payload_type(id)
            .or_else(|| self.identity_registry.payload_type(id))
            .is_some_and(|existing| existing != payload_type)
        {
            return Err(ResourceRegistryError::PayloadTypeConflict { id: id.to_string() });
        }
        self.registry.set_payload_type(id, Some(payload_type));
        Ok(())
    }

    /// Moves an offline record to a new locator while retaining its established identity.
    ///
    /// This only prepares a project candidate. Publication remains exclusive to the live
    /// `ResourceManager` mutation batch, which must receive the matching explicit rename.
    pub fn stage_rename_locator(
        &mut self,
        from: &ResourceLocator,
        to: ResourceLocator,
    ) -> ResourceResult<ResourceRecord> {
        let id = self.registry.id_for_locator(from).ok_or_else(|| {
            ResourceRegistryError::MissingRecordForLocator {
                locator: from.to_string(),
            }
        })?;
        if let Some(existing_id) = self.registry.id_for_locator(&to) {
            if existing_id != id {
                return Err(ResourceRegistryError::LocatorOccupied {
                    locator: to.to_string(),
                    existing_id: existing_id.to_string(),
                    requested_id: id.to_string(),
                });
            }
        }

        let payload_type = self.registry.payload_type(id);
        let mut record = self
            .registry
            .remove_by_id(id)
            .ok_or_else(|| ResourceRegistryError::MissingRecordForId { id: id.to_string() })?;
        record.primary_locator = to;
        self.staged_identities.insert(
            id,
            ResourceRegistryIdentity {
                kind: record.kind,
                authorized_locator: record.primary_locator.clone(),
            },
        );
        self.registry.insert_unchecked(record.clone());
        self.registry.set_payload_type(id, payload_type);
        Ok(record)
    }

    fn identity_for(&self, id: ResourceId) -> Option<(ResourceKind, &ResourceLocator)> {
        self.staged_identities
            .get(&id)
            .map(|identity| (identity.kind, &identity.authorized_locator))
            .or_else(|| {
                self.identity_registry
                    .get(id)
                    .map(|record| (record.kind, &record.primary_locator))
            })
    }

    /// 从候选目录移除记录，但保留本次构建中已建立的身份约束；删除再插入不能绕过种类或定位符校验。
    pub fn stage_remove_locator(&mut self, locator: &ResourceLocator) -> Option<ResourceRecord> {
        let id = self.registry.id_for_locator(locator)?;
        self.registry.remove_by_id(id)
    }

    /// 完成离线候选快照；此结果尚未发布到运行时，在线变化仍须提交对应管理器批次。
    pub fn finish(self) -> ResourceRegistry {
        self.registry
    }
}

impl Deref for ResourceRegistryStaging {
    type Target = ResourceRegistry;

    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}
