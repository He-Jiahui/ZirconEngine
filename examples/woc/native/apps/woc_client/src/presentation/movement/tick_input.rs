use woc_protocol::{Command, MovementFrame};

/// One client-side fixed-step transaction. Commands and the independent held
/// movement stream cross the VM boundary together but retain their own
/// sequencing rules.
#[derive(Clone, Copy, Debug)]
pub struct ClientTickInput<'a> {
    commands: &'a [Command],
    movement: MovementFrame,
}

impl<'a> ClientTickInput<'a> {
    pub fn new(commands: &'a [Command], movement: MovementFrame) -> Self {
        Self { commands, movement }
    }

    pub fn commands(self) -> &'a [Command] {
        self.commands
    }

    pub fn movement(self) -> MovementFrame {
        self.movement
    }
}
