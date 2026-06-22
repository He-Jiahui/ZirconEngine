use super::super::super::command::ChromeCommand;
use super::super::model::ChromeCommandStream;

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn extend_commands(
        &mut self,
        commands: impl IntoIterator<Item = ChromeCommand>,
    ) {
        self.commands.extend(commands);
    }
}
