//! Compact owned storage that preserves the contained error's source identity.
use std::{fmt, ops::Deref};

/// Owns an error behind one allocation while retaining typed access to that error.
///
/// This deliberately does not implement std::error::Error: thiserror source
/// lookup follows Deref to the contained error rather than adding a wrapper node.
pub struct TypedErrorSource<E>(Box<E>);

impl<E> TypedErrorSource<E> {
    /// Recovers the original owned error.
    pub fn into_inner(self) -> E {
        *self.0
    }
}

impl<E> From<E> for TypedErrorSource<E> {
    fn from(source: E) -> Self {
        Self(Box::new(source))
    }
}

impl<E> Deref for TypedErrorSource<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<E: fmt::Display> fmt::Display for TypedErrorSource<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, formatter)
    }
}

impl<E: fmt::Debug> fmt::Debug for TypedErrorSource<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, formatter)
    }
}
