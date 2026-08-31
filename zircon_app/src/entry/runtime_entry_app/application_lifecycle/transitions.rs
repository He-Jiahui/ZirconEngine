use super::{
    action::SurfaceReleaseAction, machine::ApplicationLifecycleMachine,
    state::ApplicationLifecycleState,
};

impl ApplicationLifecycleMachine {
    pub(super) fn resume(&mut self) -> bool {
        match self.state {
            ApplicationLifecycleState::Cold | ApplicationLifecycleState::Suspended => {
                self.state = ApplicationLifecycleState::AwaitingSurface;
                true
            }
            ApplicationLifecycleState::AwaitingSurface
            | ApplicationLifecycleState::SurfaceActive
            | ApplicationLifecycleState::Exiting => false,
        }
    }

    pub(super) const fn surface_creation_requested(&self) -> bool {
        matches!(
            self.state,
            ApplicationLifecycleState::Cold | ApplicationLifecycleState::AwaitingSurface
        )
    }

    pub(super) fn confirm_surface_created(&mut self) {
        if self.surface_creation_requested() {
            self.state = ApplicationLifecycleState::SurfaceActive;
        }
    }

    pub(super) fn destroy_surfaces(&mut self) -> SurfaceReleaseAction {
        if self.state == ApplicationLifecycleState::SurfaceActive {
            self.state = ApplicationLifecycleState::AwaitingSurface;
            SurfaceReleaseAction::Release
        } else {
            SurfaceReleaseAction::Noop
        }
    }

    pub(super) fn suspend(&mut self) -> Option<SurfaceReleaseAction> {
        match self.state {
            ApplicationLifecycleState::Suspended | ApplicationLifecycleState::Exiting => None,
            ApplicationLifecycleState::SurfaceActive => {
                self.state = ApplicationLifecycleState::Suspended;
                Some(SurfaceReleaseAction::Release)
            }
            ApplicationLifecycleState::Cold | ApplicationLifecycleState::AwaitingSurface => {
                self.state = ApplicationLifecycleState::Suspended;
                Some(SurfaceReleaseAction::Noop)
            }
        }
    }

    pub(super) fn exit(&mut self) -> Option<SurfaceReleaseAction> {
        match self.state {
            ApplicationLifecycleState::Exiting => None,
            ApplicationLifecycleState::SurfaceActive => {
                self.state = ApplicationLifecycleState::Exiting;
                Some(SurfaceReleaseAction::Release)
            }
            ApplicationLifecycleState::Cold
            | ApplicationLifecycleState::AwaitingSurface
            | ApplicationLifecycleState::Suspended => {
                self.state = ApplicationLifecycleState::Exiting;
                Some(SurfaceReleaseAction::Noop)
            }
        }
    }

    pub(super) const fn allows_frame_pump(&self) -> bool {
        self.state == ApplicationLifecycleState::SurfaceActive
    }
}
