use std::collections::HashMap;

use super::super::command::{ChromeCommand, ChromeCommandKind};
use crate::ui::retained_host::host_contract::paint_template_nodes::copy_editor_sprite_atlas_rgba;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeImageResource {
    pub(in crate::ui::retained_host::host_contract) generation: u64,
    pub(in crate::ui::retained_host::host_contract) width: u32,
    pub(in crate::ui::retained_host::host_contract) height: u32,
    pub(in crate::ui::retained_host::host_contract) upload_bytes: u64,
    pub(in crate::ui::retained_host::host_contract) rgba: Vec<u8>,
}

pub(super) fn compact_image_resources(
    commands: &mut [ChromeCommand],
) -> HashMap<String, ChromeImageResource> {
    compact_image_resources_with_residency(commands, |_, _| false)
}

pub(super) fn compact_image_resources_with_residency(
    commands: &mut [ChromeCommand],
    mut is_resident: impl FnMut(&str, u64) -> bool,
) -> HashMap<String, ChromeImageResource> {
    let mut resources = HashMap::new();
    for command in commands {
        let ChromeCommandKind::Image { payload } = &mut command.kind else {
            continue;
        };
        if is_resident(payload.resource_key.as_str(), payload.resource_generation) {
            payload.rgba = None;
            continue;
        }
        let replace = resources
            .get(&payload.resource_key)
            .map_or(true, |resource: &ChromeImageResource| {
                resource.generation < payload.resource_generation
            });
        let rgba = payload.rgba.take().or_else(|| {
            (replace && payload.atlas_uv.is_some()).then(|| {
                copy_editor_sprite_atlas_rgba(
                    payload.resource_key.as_str(),
                    payload.resource_generation,
                )
            })?
        });
        let Some(rgba) = rgba else {
            continue;
        };
        if !replace {
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
    use super::{compact_image_resources, compact_image_resources_with_residency};
    use crate::ui::retained_host::host_contract::chrome_command_stream::{
        ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload,
    };
    use crate::ui::retained_host::host_contract::data::FrameRect;

    #[test]
    fn shared_atlas_commands_move_pixels_into_one_stream_resource() {
        let command = |generation, rgba| ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: generation as i32,
            frame: FrameRect::default(),
            clip: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
                    resource_key: "atlas://editor/icons".to_string(),
                    resource_generation: generation,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: Some(rgba),
                    resource_source_available: true,
                    atlas_uv: None,
                },
            },
        };

        let mut commands = vec![command(4, vec![4; 16]), command(5, vec![5; 16])];
        let resources = compact_image_resources(&mut commands);

        let resource = resources
            .get("atlas://editor/icons")
            .expect("newest atlas generation is canonical");
        assert_eq!(resource.generation, 5);
        assert_eq!(resource.rgba, vec![5; 16]);
        assert!(commands.iter().all(|command| matches!(
            &command.kind,
            ChromeCommandKind::Image { payload } if payload.rgba.is_none()
        )));
    }

    #[test]
    fn resident_atlas_handle_skips_the_source_resolver() {
        let mut commands = vec![ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: 0,
            frame: FrameRect::default(),
            clip: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
                    resource_key: "atlas://editor/icons".to_string(),
                    resource_generation: 7,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: Some(vec![7; 16]),
                    atlas_uv: None,
                },
            },
        }];

        let resources = compact_image_resources_with_residency(&mut commands, |key, generation| {
            key == "atlas://editor/icons" && generation == 7
        });

        assert!(resources.is_empty());
    }
}
