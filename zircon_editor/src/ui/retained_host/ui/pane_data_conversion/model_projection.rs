use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::ModelRc;

pub(super) fn map_model_rc<T, U, F>(model: &ModelRc<T>, mut map: F) -> ModelRc<U>
where
    T: Clone + 'static,
    U: Clone + 'static,
    F: FnMut(&T) -> U,
{
    model_rc(model.iter().map(&mut map).collect())
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    struct CloneProbe(Arc<AtomicUsize>);

    impl Clone for CloneProbe {
        fn clone(&self) -> Self {
            self.0.fetch_add(1, Ordering::Relaxed);
            Self(Arc::clone(&self.0))
        }
    }

    #[test]
    fn pane_model_mapping_borrows_source_rows() {
        let clone_count = Arc::new(AtomicUsize::new(0));
        let source = model_rc(vec![CloneProbe(Arc::clone(&clone_count))]);

        let mapped = map_model_rc(&source, |_| 7_u8);

        assert_eq!(mapped.row_data(0), Some(7));
        assert_eq!(clone_count.load(Ordering::Relaxed), 0);
    }
}
