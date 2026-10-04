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
#[path = "tests/model.rs"]
mod tests;
