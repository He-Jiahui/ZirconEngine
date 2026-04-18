use std::sync::{Arc, Mutex};

use super::viewport_state::ViewportState;

#[derive(Clone)]
pub(crate) struct SlintViewportController {
    pub(super) shared: Arc<Mutex<ViewportState>>,
}
