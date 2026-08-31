use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderFrameCommandRef, UiSurfaceFrame,
};

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::HostRenderSourceTable;

use super::super::command::{ChromeCommand, ChromeCommandKind};
use super::geometry::clamp_surface_size;
use super::image_resources::{
    compact_image_resources_with_residency, ChromeImageResource, ChromeImageResources,
};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeCommandStream {
    surface_size: (u32, u32),
    damage: Option<FrameRect>,
    full_rebuild: bool,
    pub(super) commands: Vec<ChromeCommand>,
    image_resources: ChromeImageResources,
    pub(super) image_resources_compacted: bool,
    render_sources: HostRenderSourceTable,
}

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn from_extracted_commands(
        surface_size: (u32, u32),
        damage: Option<FrameRect>,
        commands: Vec<ChromeCommand>,
        render_sources: HostRenderSourceTable,
    ) -> Self {
        Self {
            surface_size: clamp_surface_size(surface_size),
            full_rebuild: damage.is_none(),
            damage,
            commands,
            image_resources: ChromeImageResources::default(),
            image_resources_compacted: false,
            render_sources,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn full_rebuild(
        surface_size: (u32, u32),
    ) -> Self {
        Self {
            surface_size: clamp_surface_size(surface_size),
            damage: None,
            full_rebuild: true,
            commands: Vec::new(),
            image_resources: ChromeImageResources::default(),
            image_resources_compacted: false,
            render_sources: HostRenderSourceTable::default(),
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
            image_resources: ChromeImageResources::default(),
            image_resources_compacted: false,
            render_sources: HostRenderSourceTable::default(),
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

    pub(in crate::ui::retained_host::host_contract) fn resolve_command_source(
        &self,
        command_index: usize,
    ) -> Option<(&Arc<UiSurfaceFrame>, UiRenderFrameCommandRef, u16)> {
        let source = self.commands.get(command_index)?.source?;
        let frame = self.render_sources.resolve(source.surface_key)?;
        Some((frame, source.command_ref, source.fragment_index))
    }

    pub(in crate::ui::retained_host::host_contract) fn resolve_runtime_command_source(
        &self,
        command_index: usize,
    ) -> Option<(&Arc<UiSurfaceFrame>, &UiRenderCommand, u16)> {
        let source = self.commands.get(command_index)?.source?;
        let frame = self.render_sources.resolve(source.surface_key)?;
        let command = frame.render_extract.command_by_ref(source.command_ref)?;
        Some((frame, command, source.fragment_index))
    }

    pub(in crate::ui::retained_host::host_contract) fn image_resource(
        &self,
        resource_key: &str,
        generation: u64,
    ) -> Option<&ChromeImageResource> {
        self.image_resources.get(resource_key, generation)
    }

    pub(in crate::ui::retained_host::host_contract) fn image_resources(
        &self,
    ) -> &ChromeImageResources {
        &self.image_resources
    }

    /// Retains only sources the runtime UI registry still needs to stage.
    /// Commands retain their resource handle and generation either way.
    pub(in crate::ui::retained_host::host_contract) fn retain_unresident_image_resources(
        &mut self,
        mut is_resident: impl FnMut(&str, u64) -> bool,
    ) {
        self.image_resources
            .retain(|resource_key, generation, _| !is_resident(resource_key, generation));
    }

    pub(in crate::ui::retained_host::host_contract) fn compact_image_resources(&mut self) {
        self.compact_image_resources_with_residency(|_, _| false);
    }

    pub(in crate::ui::retained_host::host_contract) fn compact_image_resources_with_residency(
        &mut self,
        mut is_resident: impl FnMut(&str, u64) -> bool,
    ) {
        if self.image_resources_compacted {
            return;
        }
        let has_uncompacted_resource = self.commands.iter().any(|command| {
            let ChromeCommandKind::Image { payload } = &command.kind else {
                return false;
            };
            payload.rgba.is_some()
                || (payload.atlas_uv.is_some()
                    && self
                        .image_resources
                        .get(payload.resource_key.as_str(), payload.resource_generation)
                        .is_none())
        });
        if !has_uncompacted_resource {
            self.image_resources_compacted = true;
            return;
        }
        self.image_resources
            .extend(compact_image_resources_with_residency(
                &mut self.commands,
                &mut is_resident,
            ));
        self.image_resources_compacted = true;
    }

    pub(in crate::ui::retained_host::host_contract) fn into_parts(
        self,
    ) -> (
        Vec<ChromeCommand>,
        ChromeImageResources,
        HostRenderSourceTable,
    ) {
        (self.commands, self.image_resources, self.render_sources)
    }

    #[cfg(test)]
    pub(in crate::ui::retained_host::host_contract) fn push_command_for_test(
        &mut self,
        command: ChromeCommand,
    ) {
        self.image_resources_compacted = false;
        self.commands.push(command);
    }
}

#[cfg(test)]
mod tests {
    use super::super::image_resources::ChromeImageResource;
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
            source: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
                    resource_key: "image://stable".to_string(),
                    resource_generation: 9,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: Some(vec![9; 16].into()),
                    atlas_uv: None,
                },
            },
        });

        stream.compact_image_resources();
        let pixels_ptr = stream
            .image_resource("image://stable", 9)
            .expect("first compaction stores the resource")
            .rgba
            .as_ptr();

        stream.compact_image_resources();

        assert_eq!(
            stream
                .image_resource("image://stable", 9)
                .expect("second compaction preserves the resource")
                .rgba
                .as_ptr(),
            pixels_ptr
        );
    }

    #[test]
    fn resident_images_keep_their_commands_but_drop_staged_source_bytes() {
        let mut stream = ChromeCommandStream::full_rebuild((64, 64));
        stream.image_resources.insert(
            "atlas://editor/icons".to_string(),
            ChromeImageResource {
                generation: 7,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: vec![7; 16].into(),
            },
        );
        stream.image_resources.insert(
            "atlas://editor/changed".to_string(),
            ChromeImageResource {
                generation: 8,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: vec![8; 16].into(),
            },
        );

        stream.retain_unresident_image_resources(|resource_key, generation| {
            resource_key == "atlas://editor/icons" && generation == 7
        });

        assert!(stream
            .image_resources()
            .get("atlas://editor/icons", 7)
            .is_none());
        assert_eq!(
            stream
                .image_resources()
                .get("atlas://editor/changed", 8)
                .map(|resource| resource.rgba.as_ref()),
            Some(&[8; 16][..])
        );
    }

    #[test]
    fn command_append_reopens_image_resource_compaction() {
        let image = |generation| ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: generation as i32,
            frame: FrameRect {
                width: 2.0,
                height: 2.0,
                ..FrameRect::default()
            },
            clip: None,
            source: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
                    resource_key: "image://compaction-state".to_string(),
                    resource_generation: generation,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: Some(vec![generation as u8; 16].into()),
                    atlas_uv: None,
                },
            },
        };
        let mut stream = ChromeCommandStream::full_rebuild((64, 64));
        stream.push_command_for_test(image(1));

        assert!(!stream.image_resources_compacted);
        stream.compact_image_resources();
        assert!(stream.image_resources_compacted);
        assert!(stream
            .image_resource("image://compaction-state", 1)
            .is_some());

        stream.push_command_for_test(image(2));
        assert!(!stream.image_resources_compacted);
        stream.compact_image_resources();
        assert!(stream.image_resources_compacted);
        assert!(stream
            .image_resource("image://compaction-state", 2)
            .is_some());
    }
}
