use std::ops::Deref;
use std::sync::Arc;

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
