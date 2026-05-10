use std::ops::{Deref, DerefMut};

use crate::scene::ecs::{ChangeTick, ChangeTickWindow, ComponentTicks};

#[derive(Clone, Copy, Debug)]
pub struct Ref<'world, T> {
    value: &'world T,
    ticks: ComponentTicks,
    window: ChangeTickWindow,
}

#[derive(Debug)]
pub struct Mut<'world, T> {
    value: &'world mut T,
    ticks: ComponentTicks,
    window: ChangeTickWindow,
}

impl<'world, T> Ref<'world, T> {
    pub(crate) fn new(value: &'world T, ticks: ComponentTicks, window: ChangeTickWindow) -> Self {
        Self {
            value,
            ticks,
            window,
        }
    }

    pub fn into_inner(self) -> &'world T {
        self.value
    }

    pub fn is_added(&self) -> bool {
        self.ticks.is_added(self.window)
    }

    pub fn is_changed(&self) -> bool {
        self.ticks.is_changed(self.window)
    }

    pub fn last_changed(&self) -> ChangeTick {
        self.ticks.changed()
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'world, T> Mut<'world, T> {
    pub(crate) fn new(
        value: &'world mut T,
        ticks: ComponentTicks,
        window: ChangeTickWindow,
    ) -> Self {
        Self {
            value,
            ticks,
            window,
        }
    }

    pub fn into_inner(self) -> &'world mut T {
        self.value
    }

    pub fn is_added(&self) -> bool {
        self.ticks.is_added(self.window)
    }

    pub fn is_changed(&self) -> bool {
        self.ticks.is_changed(self.window)
    }

    pub fn last_changed(&self) -> ChangeTick {
        self.ticks.changed()
    }
}

impl<T> Deref for Mut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> DerefMut for Mut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}
