use std::collections::BTreeMap;

use zircon_ui::UiEventPath;

use super::Handler;

pub struct EditorUiRouter<T> {
    pub(crate) exact_routes: BTreeMap<UiEventPath, Vec<Handler<T>>>,
}

impl<T> Default for EditorUiRouter<T> {
    fn default() -> Self {
        Self {
            exact_routes: BTreeMap::new(),
        }
    }
}
