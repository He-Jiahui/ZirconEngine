use std::collections::HashMap;

use super::{UiSurfaceCommand, UiSurfaceCommandKind};

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceImageResource {
    /// Producer revision for this resource payload, independent from draw order or damage.
    pub generation: u64,
    pub width: u32,
    pub height: u32,
    pub upload_bytes: u64,
    pub rgba: Vec<u8>,
}

pub(super) fn compact_image_resources(
    mut commands: Vec<UiSurfaceCommand>,
) -> (
    Vec<UiSurfaceCommand>,
    HashMap<String, UiSurfaceImageResource>,
) {
    let mut resources = HashMap::new();
    for command in &mut commands {
        let UiSurfaceCommandKind::Image { payload } = &mut command.kind else {
            continue;
        };
        let Some(rgba) = payload.rgba.take() else {
            continue;
        };
        let replace = resources
            .get(&payload.resource_key)
            .map_or(true, |resource: &UiSurfaceImageResource| {
                resource.generation < payload.resource_generation
            });
        if !replace {
            continue;
        }
        resources.insert(
            payload.resource_key.clone(),
            UiSurfaceImageResource {
                generation: payload.resource_generation,
                width: payload.width,
                height: payload.height,
                upload_bytes: payload.upload_bytes,
                rgba,
            },
        );
    }
    (commands, resources)
}

#[cfg(test)]
mod tests {
    use super::compact_image_resources;
    use crate::rhi::{
        UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceImagePayload, UiSurfaceRect,
    };

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
        assert_eq!(resources["atlas://editor/icons"].generation, 23);
        assert_eq!(resources["atlas://editor/icons"].rgba, vec![0; 16]);
        assert!(commands.iter().all(|command| matches!(
            &command.kind,
            UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
        )));
    }

    #[test]
    fn newer_image_generation_replaces_the_canonical_resource_payload() {
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

        let resource = resources
            .get("atlas://editor/icons")
            .expect("newest atlas generation is canonical");
        assert_eq!(resource.generation, 5);
        assert_eq!(resource.rgba, vec![5; 16]);
        assert!(commands.iter().all(|command| matches!(
            &command.kind,
            UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
        )));
    }
}
