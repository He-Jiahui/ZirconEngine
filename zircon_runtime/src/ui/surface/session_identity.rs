use std::sync::Arc;

/// 区分拥有相同树 ID 的不同运行时 surface 实例，隔离文档历史和未完成模型回写。
/// 普通克隆获得新身份；值相等不代表可继续使用原实例的宿主会话。
#[derive(Debug, Default)]
pub(super) struct UiSurfaceSessionIdentity(Arc<()>);

impl UiSurfaceSessionIdentity {
    pub(super) fn handle(&self) -> UiSurfaceSessionIdentityHandle {
        UiSurfaceSessionIdentityHandle(Arc::clone(&self.0))
    }
}

impl Clone for UiSurfaceSessionIdentity {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for UiSurfaceSessionIdentity {
    fn eq(&self, _other: &Self) -> bool {
        // Runtime ownership identity is intentionally absent from semantic surface equality.
        true
    }
}

/// 宿主保留此句柄以识别会话切换；句柄克隆保持身份，surface 克隆则重新发放身份。
#[derive(Clone, Debug)]
pub(crate) struct UiSurfaceSessionIdentityHandle(Arc<()>);

impl PartialEq for UiSurfaceSessionIdentityHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for UiSurfaceSessionIdentityHandle {}
