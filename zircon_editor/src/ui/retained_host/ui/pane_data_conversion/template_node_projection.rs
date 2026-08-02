use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;

#[cfg(test)]
use crate::ui::layouts::common::model_rc;

pub(super) fn project_nodes<T, F>(
    nodes: &ModelRc<T>,
    map: F,
) -> ModelRc<host_contract::TemplatePaneNodeData>
where
    T: Clone + 'static,
    F: FnMut(&T) -> host_contract::TemplatePaneNodeData,
{
    nodes.map_preserving_metadata(map)
}

pub(super) fn project_node_vec<T, F>(
    nodes: &ModelRc<T>,
    mut map: F,
) -> Vec<host_contract::TemplatePaneNodeData>
where
    T: Clone + 'static,
    F: FnMut(&T) -> host_contract::TemplatePaneNodeData,
{
    nodes.iter().map(&mut map).collect()
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    struct CloneProbe(Arc<AtomicUsize>);

    #[derive(Debug, PartialEq, Eq)]
    struct FixtureMetadata {
        generation: u64,
    }

    impl Clone for CloneProbe {
        fn clone(&self) -> Self {
            self.0.fetch_add(1, Ordering::Relaxed);
            Self(Arc::clone(&self.0))
        }
    }

    #[test]
    fn pane_template_node_projection_borrows_source_rows() {
        let clone_count = Arc::new(AtomicUsize::new(0));
        let source = model_rc(vec![CloneProbe(Arc::clone(&clone_count))]);

        let projected = project_nodes(&source, |_| host_contract::TemplatePaneNodeData::default());

        assert_eq!(projected.row_count(), 1);
        assert_eq!(clone_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn pane_template_node_projection_preserves_generation_metadata() {
        let source = ModelRc::with_metadata(
            vec![CloneProbe(Arc::new(AtomicUsize::new(0)))],
            FixtureMetadata { generation: 11 },
        );
        let source_metadata = source
            .metadata_rc::<FixtureMetadata>()
            .expect("source metadata");

        let projected = project_nodes(&source, |_| host_contract::TemplatePaneNodeData::default());
        let projected_metadata = projected
            .metadata_rc::<FixtureMetadata>()
            .expect("projected metadata");

        assert!(std::rc::Rc::ptr_eq(&source_metadata, &projected_metadata));
    }
}
