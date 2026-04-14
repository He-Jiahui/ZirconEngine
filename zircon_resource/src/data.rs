use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

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
