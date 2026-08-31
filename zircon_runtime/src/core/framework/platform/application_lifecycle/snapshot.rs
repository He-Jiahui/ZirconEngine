use super::{
    ApplicationActivationState, ApplicationLifecycleGeneration, ApplicationLifecycleOperation,
    ApplicationLifecycleState, ApplicationLifecycleTerminalResult, ApplicationSurfaceAvailability,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplicationLifecycleSnapshot {
    state: ApplicationLifecycleState,
    activation: ApplicationActivationState,
    surface_availability: ApplicationSurfaceAvailability,
    active_operation: Option<ApplicationLifecycleOperation>,
    terminal: Option<ApplicationLifecycleTerminalResult>,
    generation: ApplicationLifecycleGeneration,
}

impl ApplicationLifecycleSnapshot {
    pub(crate) const fn new(
        state: ApplicationLifecycleState,
        activation: ApplicationActivationState,
        surface_availability: ApplicationSurfaceAvailability,
        active_operation: Option<ApplicationLifecycleOperation>,
        terminal: Option<ApplicationLifecycleTerminalResult>,
        generation: ApplicationLifecycleGeneration,
    ) -> Self {
        Self {
            state,
            activation,
            surface_availability,
            active_operation,
            terminal,
            generation,
        }
    }

    pub const fn state(self) -> ApplicationLifecycleState {
        self.state
    }

    pub const fn activation(self) -> ApplicationActivationState {
        self.activation
    }

    pub const fn surface_availability(self) -> ApplicationSurfaceAvailability {
        self.surface_availability
    }

    pub const fn active_operation(self) -> Option<ApplicationLifecycleOperation> {
        self.active_operation
    }

    pub const fn terminal(self) -> Option<ApplicationLifecycleTerminalResult> {
        self.terminal
    }

    pub const fn generation(self) -> ApplicationLifecycleGeneration {
        self.generation
    }

    pub const fn allows_runtime_updates(self) -> bool {
        matches!(self.state, ApplicationLifecycleState::Running)
    }
}
