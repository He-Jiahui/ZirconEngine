use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use super::{ResourceId, ResourceMarker, UntypedResourceHandle};

/// 仅携带资源身份的类型化句柄；复制句柄不持有载荷，也不保证资源已就绪。
/// 载荷读取须通过资源管理器，失败重载后仍可能取得上次成功载入的载荷。
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 以静态类型标记传递资源身份，供上层 API 选择相应的资源读取入口。
///
/// # 所有权与生命周期
///
/// 句柄只保存资源 ID 和类型标记。复制或克隆句柄不持有载荷租约，也不会延长资源记录或载荷的生存期。
///
/// # 构造与验证边界
///
/// 构造不查询注册表；序列化只保存 ID，类型标记不进入数据。句柄类型和成功反序列化都不能证明该 ID 存在、记录种类匹配或载荷可读。读取时仍由管理器核对记录种类与具体载荷。
///
/// # 参考
///
/// [`Self::new`]、[`crate::resource::ResourceMarker`]、[`crate::resource::UntypedResourceHandle`]。
pub struct ResourceHandle<TMarker> {
    id: ResourceId,
    #[serde(skip)]
    marker: PhantomData<TMarker>,
}

impl<TMarker> Copy for ResourceHandle<TMarker> {}

impl<TMarker> Clone for ResourceHandle<TMarker> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<TMarker> ResourceHandle<TMarker> {
    pub fn new(id: ResourceId) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }

    pub fn id(self) -> ResourceId {
        self.id
    }
}

impl<TMarker: ResourceMarker> From<ResourceHandle<TMarker>> for UntypedResourceHandle {
    fn from(value: ResourceHandle<TMarker>) -> Self {
        Self::new(value.id, TMarker::KIND)
    }
}
