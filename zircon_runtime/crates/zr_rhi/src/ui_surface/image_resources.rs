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
        for (resource_key, resource) in resources.into_entries() {
            self.insert(resource_key, resource);
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
mod tests {
    use super::compact_image_resources;
    use crate::{
        UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceImagePayload, UiSurfaceImageResource,
        UiSurfaceImageResourceTable, UiSurfaceRect,
    };

    #[test]
    fn clearing_image_resource_table_removes_all_resource_generations() {
        let mut resources = UiSurfaceImageResourceTable::default();
        for generation in [4, 5] {
            resources.insert(
                "atlas://editor/icons".to_owned(),
                UiSurfaceImageResource {
                    generation,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: vec![generation as u8; 16].into(),
                },
            );
        }

        resources.clear();

        assert!(resources.is_empty());
        assert_eq!(resources.len(), 0);
    }

    #[test]
    fn shared_image_commands_move_rgba_into_one_resource_entry() {
        let image = |z_index| UiSurfaceCommand {
            z_index,
            frame: UiSurfaceRect::new(0.0, 0.0, 8.0, 8.0),
            clip: None,
            kind: UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: "atlas://editor/icons".to_string(),
                    resource_generation: 23,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: Some(vec![z_index as u8; 16]),
                    atlas_uv: None,
                },
            },
        };

        let (commands, resources) = compact_image_resources(vec![image(0), image(1)]);

        assert_eq!(resources.len(), 1);
        let resource = resources
            .get("atlas://editor/icons", 23)
            .expect("shared atlas generation is canonical");
        assert_eq!(resource.generation, 23);
        assert_eq!(resource.rgba.as_ref(), &[0; 16]);
        assert!(commands.iter().all(|command| matches!(
            &command.kind,
            UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
        )));
    }

    #[test]
    fn distinct_image_generations_remain_separate_canonical_resource_payloads() {
        let command = |generation, rgba| UiSurfaceCommand {
            z_index: generation as i32,
            frame: UiSurfaceRect::new(0.0, 0.0, 8.0, 8.0),
            clip: None,
            kind: UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: "atlas://editor/icons".to_string(),
                    resource_generation: generation,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: Some(rgba),
                    atlas_uv: None,
                },
            },
        };

        let (commands, resources) =
            compact_image_resources(vec![command(4, vec![4; 16]), command(5, vec![5; 16])]);

        assert_eq!(resources.len(), 2);
        assert_eq!(
            resources
                .get("atlas://editor/icons", 4)
                .expect("older generation remains addressable")
                .rgba
                .as_ref(),
            &[4; 16]
        );
        assert_eq!(
            resources
                .get("atlas://editor/icons", 5)
                .expect("newer generation remains addressable")
                .rgba
                .as_ref(),
            &[5; 16]
        );
        assert!(commands.iter().all(|command| matches!(
            &command.kind,
            UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
        )));
    }
}
