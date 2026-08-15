use std::fmt::{Display, Formatter};

use super::HubSessionToken;

impl Display for HubSessionToken {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
