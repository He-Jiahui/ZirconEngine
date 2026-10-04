use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

/// 资源服务保存的类型擦除载荷；具体资产类型满足调试、线程安全与 `'static` 约束后，可跨加载线程共享。
/// 借用用于类型检查，拥有的 `Arc` 转换用于取回具体类型；两者都不重新构造载荷。
pub trait ResourceData: Any + Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl<T> ResourceData for T
where
    T: Any + Debug + Send + Sync,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}
