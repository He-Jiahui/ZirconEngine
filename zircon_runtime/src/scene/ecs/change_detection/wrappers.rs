use std::ops::{Deref, DerefMut};

use crate::scene::ecs::{ChangeTick, ChangeTickWindow, ComponentMutationRecorder, ComponentTicks};

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
    changed_tick: Option<&'world mut ComponentTicks>,
    this_run: ChangeTick,
    window: ChangeTickWindow,
    mutation_recorder: Option<ComponentMutationRecorder<'world>>,
    mutation_recorded: bool,
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
        changed_tick: &'world mut ComponentTicks,
        this_run: ChangeTick,
        window: ChangeTickWindow,
    ) -> Self {
        Self {
            value,
            ticks: *changed_tick,
            changed_tick: Some(changed_tick),
            this_run,
            window,
            mutation_recorder: None,
            mutation_recorded: false,
        }
    }

    pub(crate) fn new_tracked(
        value: &'world mut T,
        changed_tick: &'world mut ComponentTicks,
        this_run: ChangeTick,
        window: ChangeTickWindow,
        mutation_recorder: ComponentMutationRecorder<'world>,
    ) -> Self {
        Self {
            value,
            ticks: *changed_tick,
            changed_tick: Some(changed_tick),
            this_run,
            window,
            mutation_recorder: Some(mutation_recorder),
            mutation_recorded: false,
        }
    }

    pub(crate) fn new_eager(
        value: &'world mut T,
        ticks: ComponentTicks,
        window: ChangeTickWindow,
    ) -> Self {
        Self {
            value,
            ticks,
            changed_tick: None,
            this_run: window.this_run(),
            window,
            mutation_recorder: None,
            mutation_recorded: false,
        }
    }

    pub fn into_inner(mut self) -> &'world mut T {
        self.set_changed();
        self.value
    }

    pub fn as_mut(&mut self) -> &mut T {
        self.set_changed();
        self.value
    }

    pub fn set_changed(&mut self) {
        let Some(ticks) = self.changed_tick.as_deref_mut() else {
            return;
        };
        ticks.set_changed(self.this_run);
        self.ticks = *ticks;
        if !self.mutation_recorded {
            if let Some(recorder) = self.mutation_recorder {
                recorder.record();
            }
            self.mutation_recorded = true;
        }
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
        self.as_mut()
    }
}
