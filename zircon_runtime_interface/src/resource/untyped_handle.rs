use serde::{Deserialize, Serialize};

use super::{ResourceHandle, ResourceId, ResourceKind, ResourceMarker};

/// 在事件、清单等异构边界携带资源身份和种类；转换为类型化句柄只校验种类。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 在异构资源边界携带资源 ID 与种类标签，供调用端选择静态类型入口。
///
/// # 所有权与验证边界
///
/// 句柄可复制且不持有载荷。构造和反序列化仅保存携带的身份与标签；不查询注册表，不证明二者配对正确。
///
/// # 使用时机
///
/// 转换成类型化句柄只检查种类标签，资源存在性、就绪情况和实际载荷类型仍由管理器读取路径确认。
///
/// # 参考
///
/// [`Self::typed`]、[`crate::resource::ResourceHandle`]、[`crate::resource::ResourceKind`]。
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
    /// 仅在携带种类与静态 marker 相符时，提供类型化的身份句柄。
    ///
    /// # 类型参数
    ///
    /// `TMarker` 提供所需的种类标签；它不代表该 ID 已加载的具体载荷类型。
    ///
    /// # 返回与使用时机
    ///
    /// 标签不匹配返回 None；匹配时返回保存同一 ID 的句柄，不查询注册表或取得载荷。转换后仍须走管理器读取入口检查存在、种类和具体载荷。
    ///
    /// # 参考
    ///
    /// [`crate::resource::ResourceMarker`]、[`crate::resource::ResourceHandle::new`]。
    pub fn typed<TMarker: ResourceMarker>(self) -> Option<ResourceHandle<TMarker>> {
        (self.kind == TMarker::KIND).then(|| ResourceHandle::new(self.id))
    }
}
