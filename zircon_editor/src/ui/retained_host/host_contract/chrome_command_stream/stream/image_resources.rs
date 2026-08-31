use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use super::super::command::{ChromeCommand, ChromeCommandKind};
use crate::ui::retained_host::host_contract::paint_template_nodes::copy_editor_sprite_atlas_rgba;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeImageResource {
    pub(in crate::ui::retained_host::host_contract) generation: u64,
    pub(in crate::ui::retained_host::host_contract) width: u32,
    pub(in crate::ui::retained_host::host_contract) height: u32,
    pub(in crate::ui::retained_host::host_contract) upload_bytes: u64,
    pub(in crate::ui::retained_host::host_contract) rgba: Arc<[u8]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeImageResources {
    by_resource_key: HashMap<String, BTreeMap<u64, ChromeImageResource>>,
}

impl ChromeImageResources {
    pub(in crate::ui::retained_host::host_contract) fn insert(
        &mut self,
        resource_key: String,
        resource: ChromeImageResource,
    ) {
        self.by_resource_key
            .entry(resource_key)
            .or_default()
            .insert(resource.generation, resource);
    }

    pub(in crate::ui::retained_host::host_contract) fn get(
        &self,
        resource_key: &str,
        generation: u64,
    ) -> Option<&ChromeImageResource> {
        self.by_resource_key
            .get(resource_key)
            .and_then(|generations| generations.get(&generation))
    }

    pub(in crate::ui::retained_host::host_contract) fn is_empty(&self) -> bool {
        self.by_resource_key.is_empty()
    }

    #[cfg(test)]
    pub(in crate::ui::retained_host::host_contract) fn len(&self) -> usize {
        self.by_resource_key.values().map(BTreeMap::len).sum()
    }

    pub(in crate::ui::retained_host::host_contract) fn retain(
        &mut self,
        mut keep: impl FnMut(&str, u64, &ChromeImageResource) -> bool,
    ) {
        self.by_resource_key.retain(|resource_key, generations| {
            generations.retain(|generation, resource| keep(resource_key, *generation, resource));
            !generations.is_empty()
        });
    }

    pub(in crate::ui::retained_host::host_contract) fn iter(
        &self,
    ) -> impl Iterator<Item = (&str, u64, &ChromeImageResource)> {
        self.by_resource_key
            .iter()
            .flat_map(|(resource_key, generations)| {
                generations.iter().map(move |(generation, resource)| {
                    (resource_key.as_str(), *generation, resource)
                })
            })
    }

    pub(in crate::ui::retained_host::host_contract) fn into_entries(
        self,
    ) -> impl Iterator<Item = (String, ChromeImageResource)> {
        self.by_resource_key
            .into_iter()
            .flat_map(|(resource_key, generations)| {
                generations
                    .into_values()
                    .map(move |resource| (resource_key.clone(), resource))
            })
    }

    pub(in crate::ui::retained_host::host_contract) fn extend(&mut self, resources: Self) {
        for (resource_key, mut generations) in resources.by_resource_key {
            self.by_resource_key
                .entry(resource_key)
                .or_default()
                .append(&mut generations);
        }
    }
}

pub(super) fn compact_image_resources(commands: &mut [ChromeCommand]) -> ChromeImageResources {
    compact_image_resources_with_residency(commands, |_, _| false)
}

pub(super) fn compact_image_resources_with_residency(
    commands: &mut [ChromeCommand],
    mut is_resident: impl FnMut(&str, u64) -> bool,
) -> ChromeImageResources {
    let mut resources = ChromeImageResources::default();
    let mut resident_results = HashMap::<String, BTreeMap<u64, bool>>::new();
    for command in commands {
        let ChromeCommandKind::Image { payload } = &mut command.kind else {
            continue;
        };
        let resident = resident_results
            .get(payload.resource_key.as_str())
            .and_then(|generations| generations.get(&payload.resource_generation))
            .copied()
            .unwrap_or_else(|| {
                let resident =
                    is_resident(payload.resource_key.as_str(), payload.resource_generation);
                resident_results
                    .entry(payload.resource_key.clone())
                    .or_default()
                    .insert(payload.resource_generation, resident);
                resident
            });
        if resident {
            payload.rgba = None;
            continue;
        }
        let needs_resource = resources
            .get(payload.resource_key.as_str(), payload.resource_generation)
            .is_none();
        let rgba = payload.rgba.take().or_else(|| {
            (needs_resource && payload.atlas_uv.is_some()).then(|| {
                copy_editor_sprite_atlas_rgba(
                    payload.resource_key.as_str(),
                    payload.resource_generation,
                )
                .map(Arc::from)
            })?
        });
        let Some(rgba) = rgba else {
            continue;
        };
        if !needs_resource {
            continue;
        }
        resources.insert(
            payload.resource_key.clone(),
            ChromeImageResource {
                generation: payload.resource_generation,
                width: payload.width,
                height: payload.height,
                upload_bytes: payload.upload_bytes,
                rgba,
            },
        );
    }
    resources
}

#[cfg(test)]
mod tests {
    use super::{
        compact_image_resources, compact_image_resources_with_residency, ChromeImageResource,
        ChromeImageResources,
    };
    use crate::ui::retained_host::host_contract::chrome_command_stream::{
        ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload,
    };
    use crate::ui::retained_host::host_contract::data::FrameRect;

    #[test]
    fn same_generation_atlas_commands_move_pixels_into_one_stream_resource() {
        let command = |generation, rgba| ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: generation as i32,
            frame: FrameRect::default(),
            clip: None,
            source: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
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

        let mut commands = vec![
            command(5, vec![5; 16].into()),
            command(5, vec![5; 16].into()),
        ];
        let resources = compact_image_resources(&mut commands);

        let resource = resources
            .get("atlas://editor/icons", 5)
            .expect("shared atlas generation is canonical");
        assert_eq!(resource.generation, 5);
        assert_eq!(resource.rgba.as_ref(), &[5; 16]);
        assert!(commands.iter().all(|command| matches!(
            &command.kind,
            ChromeCommandKind::Image { payload } if payload.rgba.is_none()
        )));
    }

    #[test]
    fn distinct_atlas_generations_remain_separate_stream_resources() {
        let command = |generation, rgba| ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: generation as i32,
            frame: FrameRect::default(),
            clip: None,
            source: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
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

        let mut commands = vec![
            command(4, vec![4; 16].into()),
            command(5, vec![5; 16].into()),
        ];
        let resources = compact_image_resources(&mut commands);

        assert_eq!(resources.len(), 2);
        assert_eq!(
            resources
                .get("atlas://editor/icons", 4)
                .expect("older generation must remain available for its command")
                .rgba
                .as_ref(),
            &[4; 16]
        );
        assert_eq!(
            resources
                .get("atlas://editor/icons", 5)
                .expect("newer generation must remain available for its command")
                .rgba
                .as_ref(),
            &[5; 16]
        );
    }

    #[test]
    fn resource_group_merge_moves_all_generations_and_preserves_newer_batch_authority() {
        let resource = |generation, value| ChromeImageResource {
            generation,
            width: 1,
            height: 1,
            upload_bytes: 4,
            rgba: vec![value; 4].into(),
        };
        let mut retained = ChromeImageResources::default();
        retained.insert("image://shared".to_string(), resource(1, 1));
        retained.insert("image://shared".to_string(), resource(2, 2));
        let mut incoming = ChromeImageResources::default();
        incoming.insert("image://shared".to_string(), resource(2, 22));
        incoming.insert("image://shared".to_string(), resource(3, 3));

        retained.extend(incoming);

        assert_eq!(retained.len(), 3);
        assert_eq!(
            retained
                .get("image://shared", 1)
                .expect("retained generation")
                .rgba
                .as_ref(),
            &[1; 4]
        );
        assert_eq!(
            retained
                .get("image://shared", 2)
                .expect("incoming generation replaces the old generation")
                .rgba
                .as_ref(),
            &[22; 4]
        );
        assert_eq!(
            retained
                .get("image://shared", 3)
                .expect("new generation")
                .rgba
                .as_ref(),
            &[3; 4]
        );
    }

    #[test]
    fn resident_atlas_handle_skips_the_source_resolver() {
        let mut commands = vec![ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: 0,
            frame: FrameRect::default(),
            clip: None,
            source: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
                    resource_key: "atlas://editor/icons".to_string(),
                    resource_generation: 7,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: Some(vec![7; 16].into()),
                    atlas_uv: None,
                },
            },
        }];

        let resources = compact_image_resources_with_residency(&mut commands, |key, generation| {
            key == "atlas://editor/icons" && generation == 7
        });

        assert!(resources.is_empty());
    }

    #[test]
    fn repeated_commands_probe_one_resource_generation_once() {
        let command = || ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: 0,
            frame: FrameRect::default(),
            clip: None,
            source: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
                    resource_key: "atlas://editor/icons".to_string(),
                    resource_generation: 7,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: Some(vec![7; 16].into()),
                    atlas_uv: None,
                },
            },
        };
        let mut commands = vec![command(), command(), command()];
        let mut probe_count = 0;

        let resources = compact_image_resources_with_residency(&mut commands, |_, _| {
            probe_count += 1;
            true
        });

        assert_eq!(probe_count, 1);
        assert!(resources.is_empty());
    }
}
