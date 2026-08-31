use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// A scene-extract domain shared across submissions until that domain is
/// explicitly derived for one submission.
#[repr(transparent)]
#[derive(Debug, PartialEq)]
pub struct RenderSharedSceneDomain<T>(Arc<T>);

impl<T> RenderSharedSceneDomain<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Clone for RenderSharedSceneDomain<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> From<T> for RenderSharedSceneDomain<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> AsRef<T> for RenderSharedSceneDomain<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> Deref for RenderSharedSceneDomain<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T: Clone> DerefMut for RenderSharedSceneDomain<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}
