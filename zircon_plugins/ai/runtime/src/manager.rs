use std::sync::{Arc, Mutex};

mod behavior_tree;
mod blackboard;
mod execution;
mod parameters;
mod perception;
mod service;
mod snapshot;
mod state;
mod tick;
mod validation;

use state::AiRuntimeState;

#[derive(Clone, Debug, Default)]
pub struct DefaultAiManager {
    state: Arc<Mutex<AiRuntimeState>>,
}
