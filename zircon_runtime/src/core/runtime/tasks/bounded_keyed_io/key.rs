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
mod tests {
    use super::BoundedKeyedIoKey;

    #[derive(Clone, PartialEq, Eq)]
    struct PhysicalPathIdentity(u64);

    #[test]
    fn typed_keys_preserve_domain_equality_across_clones() {
        let first = BoundedKeyedIoKey::from_value(PhysicalPathIdentity(7));
        let same = BoundedKeyedIoKey::from_value(PhysicalPathIdentity(7));
        let different = BoundedKeyedIoKey::from_value(PhysicalPathIdentity(8));

        assert_eq!(first, first.clone());
        assert_eq!(first, same);
        assert_ne!(first, different);
    }

    #[test]
    fn different_key_domains_never_collide() {
        let typed = BoundedKeyedIoKey::from_value(PhysicalPathIdentity(7));
        let numeric = BoundedKeyedIoKey::from_value(7_u64);

        assert_ne!(typed, numeric);
    }

    #[test]
    fn existing_string_conversions_share_one_key_domain() {
        let borrowed = BoundedKeyedIoKey::from("archive");
        let owned = BoundedKeyedIoKey::from(String::from("archive"));
        let shared = BoundedKeyedIoKey::from(Arc::<str>::from("archive"));

        assert_eq!(borrowed, owned);
        assert_eq!(owned, shared);
    }
}
