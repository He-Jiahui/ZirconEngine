use serde::{Deserialize, Serialize};

use super::{ResourceHandle, ResourceId, ResourceKind, ResourceMarker};

/// 在事件、清单等异构边界携带资源身份和种类；转换为类型化句柄只校验种类。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UntypedResourceHandle {
    id: ResourceId,
    kind: ResourceKind,
}

impl UntypedResourceHandle {
    pub fn new(id: ResourceId, kind: ResourceKind) -> Self {
        Self { id, kind }
    }

    pub fn id(self) -> ResourceId {
        self.id
    }

    pub fn kind(self) -> ResourceKind {
        self.kind
    }

    /// 种类匹配不代表资源存在或载荷可读，后续仍应向资源管理器查询。
    pub fn typed<TMarker: ResourceMarker>(self) -> Option<ResourceHandle<TMarker>> {
        (self.kind == TMarker::KIND).then(|| ResourceHandle::new(self.id))
    }
}
