use std::ops::Deref;
use std::sync::Arc;

/// 持有 provider 的强引用，适合生命周期由插件依赖关系约束的调用方。
///
/// 从冻结表获取后不会随禁用自动撤销；插件目录在禁用或停用前检查强依赖。
///
/// # 所有权与生命周期
/// clone 共享取得时的同一 provider；表内后续替换 provider 不改变已取得句柄的目标。
/// 每个句柄持有一份 `Arc` 所有权，解引用返回的共享引用受该句柄当前借用期约束。
///
/// # 参考
/// [`Clone::clone`]、[`Deref::deref`]；获取入口为插件冻结表的 `resolve_strong`。
#[derive(Debug)]
pub struct StrongBridge<T: ?Sized> {
    target: Arc<T>,
}

impl<T: ?Sized> StrongBridge<T> {
    /// 当前插件表在解析并校验类型后调用此构造，将共享 provider 移入强句柄。
    ///
    /// # 参数与返回
    /// `target` 的 `Arc` 所有权移入返回值；此构造不调用 provider 的业务方法。
    pub(crate) fn new(target: Arc<T>) -> Self {
        Self { target }
    }
}

impl<T: ?Sized> Clone for StrongBridge<T> {
    /// 克隆 provider 的共享所有权；两个句柄继续指向同一实例。
    fn clone(&self) -> Self {
        Self {
            target: self.target.clone(),
        }
    }
}

impl<T: ?Sized> Deref for StrongBridge<T> {
    type Target = T;

    /// 返回所持 provider 的共享借用，引用寿命受本次 `self` 借用约束。
    fn deref(&self) -> &Self::Target {
        &self.target
    }
}
