use std::any::{type_name, Any};
use std::fmt;
use std::sync::Arc;

/// Type-erased equality key for one bounded I/O serialization domain.
///
/// The lane retains the caller's equality contract instead of flattening domain identities into
/// strings. Keys of different concrete types are always distinct.
#[derive(Clone)]
pub struct BoundedKeyedIoKey {
    inner: Arc<dyn ErasedBoundedKeyedIoKey>,
}

trait ErasedBoundedKeyedIoKey: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn equals(&self, other: &dyn ErasedBoundedKeyedIoKey) -> bool;
    fn type_name(&self) -> &'static str;
}

// 擦除只隐藏键的具体类型；相等比较仍先做 Any 下转型，因此不同类型域不会意外合并。
impl<T> ErasedBoundedKeyedIoKey for T
where
    T: Eq + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals(&self, other: &dyn ErasedBoundedKeyedIoKey) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|other| self == other)
    }

    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}

impl BoundedKeyedIoKey {
    pub fn from_value<T>(value: T) -> Self
    where
        T: Eq + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(value),
        }
    }
}

impl PartialEq for BoundedKeyedIoKey {
    fn eq(&self, other: &Self) -> bool {
        self.inner.equals(other.inner.as_ref())
    }
}

impl Eq for BoundedKeyedIoKey {}

impl fmt::Debug for BoundedKeyedIoKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundedKeyedIoKey")
            .field("type", &self.inner.type_name())
            .finish_non_exhaustive()
    }
}

impl From<&str> for BoundedKeyedIoKey {
    fn from(value: &str) -> Self {
        Self::from_value(Arc::<str>::from(value))
    }
}

impl From<String> for BoundedKeyedIoKey {
    fn from(value: String) -> Self {
        Self::from_value(Arc::<str>::from(value))
    }
}

impl From<Arc<str>> for BoundedKeyedIoKey {
    fn from(value: Arc<str>) -> Self {
        Self::from_value(value)
    }
}

#[cfg(test)]
#[path = "tests/key.rs"]
mod tests;
