use std::num::NonZeroU8;

/// Stable semantic exit classification before a platform-specific numeric code is selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductExitClass {
    Success,
    StartupFailure,
    RuntimeFailure,
    ShutdownFailure,
    ForcedTermination,
}

/// Portable process exit code selected from a semantic host result or an explicit command code.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProductProcessExitCode {
    #[default]
    Success,
    Failure(NonZeroU8),
}

impl ProductProcessExitCode {
    pub const fn failure() -> Self {
        Self::Failure(NonZeroU8::MIN)
    }

    pub const fn from_code(code: u8) -> Self {
        match NonZeroU8::new(code) {
            Some(code) => Self::Failure(code),
            None => Self::Success,
        }
    }

    pub const fn from_class(class: ProductExitClass) -> Self {
        match class {
            ProductExitClass::Success => Self::Success,
            ProductExitClass::StartupFailure
            | ProductExitClass::RuntimeFailure
            | ProductExitClass::ShutdownFailure
            | ProductExitClass::ForcedTermination => Self::failure(),
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Failure(code) => code.get(),
        }
    }

    pub const fn is_failure(self) -> bool {
        matches!(self, Self::Failure(_))
    }
}

impl From<ProductProcessExitCode> for std::process::ExitCode {
    fn from(value: ProductProcessExitCode) -> Self {
        Self::from(value.code())
    }
}

/// First terminal cause accepted by a product host generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductTerminalReason {
    Completed,
    WindowClosed,
    CommandCompleted,
    ParentRequested,
    PlatformTerminated,
    StartupFailed,
    RuntimeFailed,
    ShutdownFailed,
    ForcedTermination,
}

impl ProductTerminalReason {
    pub(crate) const fn exit_class(self) -> ProductExitClass {
        match self {
            Self::Completed
            | Self::WindowClosed
            | Self::CommandCompleted
            | Self::ParentRequested
            | Self::PlatformTerminated => ProductExitClass::Success,
            Self::StartupFailed => ProductExitClass::StartupFailure,
            Self::RuntimeFailed => ProductExitClass::RuntimeFailure,
            Self::ShutdownFailed => ProductExitClass::ShutdownFailure,
            Self::ForcedTermination => ProductExitClass::ForcedTermination,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::WindowClosed => "window_closed",
            Self::CommandCompleted => "command_completed",
            Self::ParentRequested => "parent_requested",
            Self::PlatformTerminated => "platform_terminated",
            Self::StartupFailed => "startup_failed",
            Self::RuntimeFailed => "runtime_failed",
            Self::ShutdownFailed => "shutdown_failed",
            Self::ForcedTermination => "forced_termination",
        }
    }
}
