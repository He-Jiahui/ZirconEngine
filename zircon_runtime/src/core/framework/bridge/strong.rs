use std::ops::Deref;
use std::sync::Arc;

/// 持有 provider 的强引用，适合生命周期由插件依赖关系约束的调用方。
///
/// 从冻结表获取后不会随禁用自动撤销；插件目录在禁用或停用前检查强依赖。
#[derive(Debug)]
pub struct StrongBridge<T: ?Sized> {
    target: Arc<T>,
}

impl<T: ?Sized> StrongBridge<T> {
    pub(crate) fn new(target: Arc<T>) -> Self {
        Self { target }
    }
}

impl<T: ?Sized> Clone for StrongBridge<T> {
    fn clone(&self) -> Self {
        Self {
            target: self.target.clone(),
        }
    }
}

impl<T: ?Sized> Deref for StrongBridge<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}
