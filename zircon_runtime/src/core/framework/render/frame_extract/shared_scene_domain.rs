use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crate::core::math::Mat4;

use super::super::SpriteExtract;

#[derive(Clone, Debug, PartialEq)]
struct SharedSceneDomainValue<T> {
    value: T,
    sprite_world_matrices: Option<Arc<Vec<Option<Mat4>>>>,
}

/// A scene-extract domain shared across submissions until that domain is
/// explicitly derived for one submission.
#[repr(transparent)]
#[derive(Debug, PartialEq)]
pub struct RenderSharedSceneDomain<T>(Arc<SharedSceneDomainValue<T>>);

impl<T> RenderSharedSceneDomain<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(SharedSceneDomainValue {
            value,
            sprite_world_matrices: None,
        }))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl RenderSharedSceneDomain<SpriteExtract> {
    // The World producer attaches rows in the finalized snapshot/phase index order.
    pub(crate) fn set_prepared_sprite_world_matrices(&mut self, matrices: Vec<Option<Mat4>>) {
        assert_eq!(matrices.len(), self.sprites.len());
        if !matrices.is_empty() {
            Arc::make_mut(&mut self.0).sprite_world_matrices = Some(Arc::new(matrices));
        }
    }

    pub(crate) fn prepared_sprite_world_matrix(&self, index: usize) -> Option<Mat4> {
        let matrices = self.0.sprite_world_matrices.as_ref()?;
        matrices.get(index).copied().flatten()
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
        &self.0.value
    }
}

impl<T> Deref for RenderSharedSceneDomain<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}

impl<T: Clone> DerefMut for RenderSharedSceneDomain<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if Arc::strong_count(&self.0) > 1 {
            crate::profile_counter!("render", "shared_scene_domain_cow_clone", 1);
        }
        let domain = Arc::make_mut(&mut self.0);
        // A raw domain mutation can replace or reorder rows even with identical TRS values.
        domain.sprite_world_matrices = None;
        &mut domain.value
    }
}
