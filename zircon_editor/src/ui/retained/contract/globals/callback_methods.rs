macro_rules! callback_methods {
    ($callbacks:ident, $on_name:ident, $invoke_name:ident, $field:ident, ($($arg:ident : $ty:ty),* $(,)?)) => {
        pub(crate) fn $on_name(&self, callback: impl Fn($($ty),*) + 'static) {
            self.state.borrow_mut().$callbacks.$field = Some(Rc::new(callback));
        }

        pub(crate) fn $invoke_name(&self, $($arg: $ty),*) {
            let callback = self.state.borrow().$callbacks.$field.clone();
            if let Some(callback) = callback {
                callback($($arg),*);
            }
        }
    };
}
