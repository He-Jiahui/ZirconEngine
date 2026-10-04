use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use super::{UiSurfaceCommand, UiSurfaceCommandKind};

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceImageResource {
    /// Producer revision for this resource payload, independent from draw order or damage.
    pub generation: u64,
    pub width: u32,
    pub height: u32,
    pub upload_bytes: u64,
    /// Canonical producer payload in straight-alpha RGBA8 byte order.
    pub rgba: Arc<[u8]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiSurfaceImageResourceTable {
    by_resource_key: HashMap<String, BTreeMap<u64, UiSurfaceImageResource>>,
}

impl UiSurfaceImageResourceTable {
    pub fn insert(&mut self, resource_key: String, resource: UiSurfaceImageResource) {
        self.by_resource_key
            .entry(resource_key)
            .or_default()
            .insert(resource.generation, resource);
    }

    pub fn get(&self, resource_key: &str, generation: u64) -> Option<&UiSurfaceImageResource> {
        self.by_resource_key
            .get(resource_key)
            .and_then(|generations| generations.get(&generation))
    }

    pub fn remove(
        &mut self,
        resource_key: &str,
        generation: u64,
    ) -> Option<UiSurfaceImageResource> {
        let (resource, remove_key) = {
            let generations = self.by_resource_key.get_mut(resource_key)?;
            let resource = generations.remove(&generation);
            (resource, generations.is_empty())
        };
        if remove_key {
            self.by_resource_key.remove(resource_key);
        }
        resource
    }

    pub fn is_empty(&self) -> bool {
        self.by_resource_key.is_empty()
    }

    pub fn clear(&mut self) {
        self.by_resource_key.clear();
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.by_resource_key.values().map(BTreeMap::len).sum()
    }

    pub fn into_entries(self) -> impl Iterator<Item = (String, UiSurfaceImageResource)> {
        self.by_resource_key
            .into_iter()
            .flat_map(|(resource_key, generations)| {
                generations
                    .into_values()
                    .map(move |resource| (resource_key.clone(), resource))
            })
    }

    pub fn extend(&mut self, resources: Self) {
        for (resource_key, mut generations) in resources.by_resource_key {
            self.by_resource_key
                .entry(resource_key)
                .or_default()
                .append(&mut generations);
        }
    }
}

pub(super) fn compact_image_resources(
    mut commands: Vec<UiSurfaceCommand>,
) -> (Vec<UiSurfaceCommand>, UiSurfaceImageResourceTable) {
    let mut resources = UiSurfaceImageResourceTable::default();
    for command in &mut commands {
        let UiSurfaceCommandKind::Image { payload } = &mut command.kind else {
            continue;
        };
        let Some(rgba) = payload.rgba.take() else {
            continue;
        };
        let needs_resource = resources
            .get(payload.resource_key.as_str(), payload.resource_generation)
            .is_none();
        if !needs_resource {
            continue;
        }
        resources.insert(
            payload.resource_key.clone(),
            UiSurfaceImageResource {
                generation: payload.resource_generation,
                width: payload.width,
                height: payload.height,
                upload_bytes: payload.upload_bytes,
                rgba: rgba.into(),
            },
        );
    }
    (commands, resources)
}

#[cfg(test)]
#[path = "tests/image_resources.rs"]
mod tests;
