use uuid::Uuid;

use super::HubSessionToken;

impl HubSessionToken {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
