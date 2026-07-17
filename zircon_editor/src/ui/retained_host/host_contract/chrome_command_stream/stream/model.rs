use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::command::ChromeCommand;
use super::geometry::clamp_surface_size;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeCommandStream {
    surface_size: (u32, u32),
    damage: Option<FrameRect>,
    full_rebuild: bool,
    pub(super) commands: Vec<ChromeCommand>,
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

    pub(in crate::ui::retained_host::host_contract) fn into_commands(self) -> Vec<ChromeCommand> {
        self.commands
    }

    #[cfg(test)]
    pub(in crate::ui::retained_host::host_contract) fn push_command_for_test(
        &mut self,
        command: ChromeCommand,
    ) {
        self.commands.push(command);
    }
}
