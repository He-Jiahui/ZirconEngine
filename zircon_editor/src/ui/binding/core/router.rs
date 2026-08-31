use std::collections::HashMap;

use zircon_runtime_interface::ui::binding::UiEventPath;

use super::Handler;

#[cfg(test)]
#[path = "router/hash_index_tests.rs"]
mod hash_index_tests;

pub struct EditorUiRouter<T> {
    pub(crate) exact_routes: HashMap<UiEventPath, Vec<Handler<T>>>,
}

impl<T> Default for EditorUiRouter<T> {
    fn default() -> Self {
        Self {
            exact_routes: HashMap::new(),
        }
    }
}
