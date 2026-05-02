use std::collections::BTreeMap;

use zircon_runtime_interface::ui::binding::{UiEventBinding, UiEventPath};

type Handler<T> = Box<dyn Fn(&UiEventBinding) -> T + Send + Sync + 'static>;

pub struct UiEventRouter<T> {
    exact_routes: BTreeMap<UiEventPath, Vec<Handler<T>>>,
}

impl<T> Default for UiEventRouter<T> {
    fn default() -> Self {
        Self {
            exact_routes: BTreeMap::new(),
        }
    }
}

impl<T> UiEventRouter<T> {
    pub fn register_exact<F>(&mut self, path: UiEventPath, handler: F)
    where
        F: Fn(&UiEventBinding) -> T + Send + Sync + 'static,
    {
        self.exact_routes
            .entry(path)
            .or_default()
            .push(Box::new(handler));
    }

    pub fn dispatch(&self, binding: &UiEventBinding) -> Vec<T> {
        self.exact_routes
            .get(&binding.path)
            .into_iter()
            .flat_map(|handlers| handlers.iter())
            .map(|handler| handler(binding))
            .collect()
    }
}
