use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::ui::retained_host) struct EditorHostWindowFailure {
    component: &'static str,
    requested: String,
    cause: String,
    recovery: &'static str,
}

impl EditorHostWindowFailure {
    pub(super) fn new(
        component: &'static str,
        requested: impl Display,
        cause: impl Display,
        recovery: &'static str,
    ) -> Self {
        Self {
            component,
            requested: requested.to_string(),
            cause: cause.to_string(),
            recovery,
        }
    }
}

impl Display for EditorHostWindowFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "editor startup diagnostic: component={} requested={} cause={} recovery={}",
            self.component, self.requested, self.cause, self.recovery
        )
    }
}

impl Error for EditorHostWindowFailure {}

#[cfg(test)]
#[path = "tests/failure.rs"]
mod tests;
