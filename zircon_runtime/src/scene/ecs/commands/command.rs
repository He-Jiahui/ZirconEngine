use crate::scene::World;

pub trait Command: Send + 'static {
    fn apply(self, world: &mut World);
}

pub struct FnCommand<F> {
    command: F,
}

impl<F> FnCommand<F> {
    pub fn new(command: F) -> Self {
        Self { command }
    }
}

impl<F> Command for FnCommand<F>
where
    F: FnOnce(&mut World) + Send + 'static,
{
    fn apply(self, world: &mut World) {
        (self.command)(world);
    }
}

impl<F> Command for F
where
    F: FnOnce(&mut World) + Send + 'static,
{
    fn apply(self, world: &mut World) {
        self(world);
    }
}

pub(crate) trait ErasedCommand: Send {
    fn apply_boxed(self: Box<Self>, world: &mut World);
}

impl<C> ErasedCommand for C
where
    C: Command,
{
    fn apply_boxed(self: Box<Self>, world: &mut World) {
        (*self).apply(world);
    }
}
