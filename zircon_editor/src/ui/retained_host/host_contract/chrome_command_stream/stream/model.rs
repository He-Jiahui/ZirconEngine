use std::collections::HashMap;

use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::command::{ChromeCommand, ChromeCommandKind};
use super::geometry::clamp_surface_size;
use super::image_resources::{compact_image_resources_with_residency, ChromeImageResource};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeCommandStream {
    surface_size: (u32, u32),
    damage: Option<FrameRect>,
    full_rebuild: bool,
    pub(super) commands: Vec<ChromeCommand>,
    image_resources: HashMap<String, ChromeImageResource>,
}

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn full_rebuild(
        surface_size: (u32, u32),
    ) -> Self {
        Self {
            surface_size: clamp_surface_size(surface_size),
            damage: None,
            full_rebuild: true,
            commands: Vec::new(),
            image_resources: HashMap::new(),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn patch(
        surface_size: (u32, u32),
        damage: FrameRect,
    ) -> Self {
        Self {
            surface_size: clamp_surface_size(surface_size),
            damage: Some(damage),
            full_rebuild: false,
            commands: Vec::new(),
            image_resources: HashMap::new(),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn is_full_rebuild(&self) -> bool {
        self.full_rebuild
    }

    pub(in crate::ui::retained_host::host_contract) fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    pub(in crate::ui::retained_host::host_contract) fn damage(&self) -> Option<&FrameRect> {
        self.damage.as_ref()
    }

    pub(in crate::ui::retained_host::host_contract) fn commands(&self) -> &[ChromeCommand] {
        &self.commands
    }

    pub(in crate::ui::retained_host::host_contract) fn image_resource(
        &self,
        resource_key: &str,
    ) -> Option<&ChromeImageResource> {
        self.image_resources.get(resource_key)
    }

    pub(in crate::ui::retained_host::host_contract) fn image_resources(
        &self,
    ) -> &HashMap<String, ChromeImageResource> {
        &self.image_resources
    }

    /// Retains only sources the runtime UI registry still needs to stage.
    /// Commands retain their resource handle and generation either way.
    pub(in crate::ui::retained_host::host_contract) fn retain_unresident_image_resources(
        &mut self,
        mut is_resident: impl FnMut(&str, u64) -> bool,
    ) {
        self.image_resources.retain(|resource_key, resource| {
            !is_resident(resource_key.as_str(), resource.generation)
        });
    }

    pub(in crate::ui::retained_host::host_contract) fn compact_image_resources(&mut self) {
        self.compact_image_resources_with_residency(|_, _| false);
    }

    pub(in crate::ui::retained_host::host_contract) fn compact_image_resources_with_residency(
        &mut self,
        mut is_resident: impl FnMut(&str, u64) -> bool,
    ) {
        let has_uncompacted_resource = self.commands.iter().any(|command| {
            let ChromeCommandKind::Image { payload } = &command.kind else {
                return false;
            };
            payload.rgba.is_some()
                || (payload.atlas_uv.is_some()
                    && self
                        .image_resources
                        .get(&payload.resource_key)
                        .map_or(true, |resource| {
                            resource.generation < payload.resource_generation
                        }))
        });
        if !has_uncompacted_resource {
            return;
        }
        for (resource_key, resource) in
            compact_image_resources_with_residency(&mut self.commands, &mut is_resident)
        {
            let replace = self
                .image_resources
                .get(&resource_key)
                .map_or(true, |existing| existing.generation < resource.generation);
            if replace {
                self.image_resources.insert(resource_key, resource);
            }
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn into_parts(
        self,
    ) -> (Vec<ChromeCommand>, HashMap<String, ChromeImageResource>) {
        (self.commands, self.image_resources)
    }

    #[cfg(test)]
    pub(in crate::ui::retained_host::host_contract) fn push_command_for_test(
        &mut self,
        command: ChromeCommand,
    ) {
        self.commands.push(command);
    }
}

#[cfg(test)]
mod tests {
    use super::ChromeCommandStream;
    use crate::ui::retained_host::host_contract::chrome_command_stream::{
        ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload,
    };
    use crate::ui::retained_host::host_contract::data::FrameRect;

    #[test]
    fn repeated_compaction_keeps_the_canonical_image_allocation() {
        let mut stream = ChromeCommandStream::full_rebuild((2, 2));
        stream.push_command_for_test(ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: 0,
            frame: FrameRect::default(),
            clip: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
                    resource_key: "image://stable".to_string(),
                    resource_generation: 9,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: Some(vec![9; 16]),
                    atlas_uv: None,
                },
            },
        });

        stream.compact_image_resources();
        let pixels_ptr = stream
            .image_resource("image://stable")
            .expect("first compaction stores the resource")
            .rgba
            .as_ptr();

        stream.compact_image_resources();

        assert_eq!(
            stream
                .image_resource("image://stable")
                .expect("second compaction preserves the resource")
                .rgba
                .as_ptr(),
            pixels_ptr
        );
    }

    #[test]
    fn resident_images_keep_their_commands_but_drop_staged_source_bytes() {
        let mut stream = ChromeCommandStream::new((64, 64), None, Vec::new());
        stream.image_resources.insert(
            "atlas://editor/icons".to_string(),
            ChromeImageResource {
                generation: 7,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: vec![7; 16],
            },
        );
        stream.image_resources.insert(
            "atlas://editor/changed".to_string(),
            ChromeImageResource {
                generation: 8,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: vec![8; 16],
            },
        );

        stream.retain_unresident_image_resources(|resource_key, generation| {
            resource_key == "atlas://editor/icons" && generation == 7
        });

        assert!(!stream
            .image_resources()
            .contains_key("atlas://editor/icons"));
        assert_eq!(
            stream
                .image_resources()
                .get("atlas://editor/changed")
                .map(|resource| resource.rgba.as_slice()),
            Some(&[8; 16][..])
        );
    }
}
