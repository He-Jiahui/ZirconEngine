use std::collections::HashSet;
use std::sync::Arc;

use crate::core::framework::render::{
    RenderMeshSnapshot, RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL,
};
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::MeshHitProxyTokenSource;
use crate::graphics::ViewportRenderFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::runtime::render_framework) struct ViewportHitProxyIdentity {
    pub(in crate::graphics::runtime::render_framework) entity: EntityId,
    pub(in crate::graphics::runtime::render_framework) instance: u64,
    pub(in crate::graphics::runtime::render_framework) subobject: u64,
}

#[derive(Clone, Debug, Default)]
pub(in crate::graphics::runtime::render_framework) struct ViewportHitProxyTable {
    identities: Arc<[ViewportHitProxyIdentity]>,
}

impl ViewportHitProxyTable {
    pub(in crate::graphics::runtime::render_framework) fn from_rendered_frame(
        frame: &ViewportRenderFrame,
        visible_stable_instance_keys: &[u64],
    ) -> Self {
        let visible = visible_stable_instance_keys
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        let mut identities = frame
            .meshes()
            .iter()
            .filter(|mesh| {
                mesh.node_id != 0
                    && mesh.stable_instance_key != 0
                    && visible.contains(&mesh.stable_instance_key)
            })
            .map(identity_from_mesh)
            .collect::<Vec<_>>();
        identities.sort_unstable_by_key(|identity| (identity.instance, identity.entity));
        identities.dedup_by_key(|identity| identity.instance);
        assert!(
            identities.len() < u32::MAX as usize,
            "viewport hit-proxy table exhausted the nonzero u32 token space"
        );
        Self {
            identities: identities.into(),
        }
    }

    pub(in crate::graphics::runtime::render_framework) fn token_for_instance(
        &self,
        stable_instance_key: u64,
    ) -> Option<u32> {
        self.identities
            .binary_search_by_key(&stable_instance_key, |identity| identity.instance)
            .ok()
            .and_then(|index| u32::try_from(index + 1).ok())
    }

    pub(in crate::graphics::runtime::render_framework) fn resolve(
        &self,
        token: u32,
    ) -> Option<ViewportHitProxyIdentity> {
        let index = usize::try_from(token.checked_sub(1)?).ok()?;
        self.identities.get(index).copied()
    }
}

impl MeshHitProxyTokenSource for ViewportHitProxyTable {
    fn token_for_instance(&self, stable_instance_key: u64) -> Option<u32> {
        ViewportHitProxyTable::token_for_instance(self, stable_instance_key)
    }
}

fn identity_from_mesh(mesh: &RenderMeshSnapshot) -> ViewportHitProxyIdentity {
    ViewportHitProxyIdentity {
        entity: mesh.node_id,
        instance: mesh.stable_instance_key,
        subobject: mesh.stable_instance_key
            & u64::from(RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL),
    }
}
