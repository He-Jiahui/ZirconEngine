use super::{EditorUiBinding, EditorUiRouter};

impl<T> EditorUiRouter<T> {
    pub fn register_exact<F>(&mut self, path: zircon_ui::UiEventPath, handler: F)
    where
        F: Fn(&EditorUiBinding) -> T + Send + Sync + 'static,
    {
        self.exact_routes
            .entry(path)
            .or_default()
            .push(Box::new(handler));
    }

    pub fn dispatch(&self, binding: &EditorUiBinding) -> Vec<T> {
        self.exact_routes
            .get(binding.path())
            .map(|handlers| handlers.iter().map(|handler| handler(binding)).collect())
            .unwrap_or_default()
    }
}
