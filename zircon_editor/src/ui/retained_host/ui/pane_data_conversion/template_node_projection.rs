use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;

pub(super) fn project_nodes<T, F>(
    nodes: &ModelRc<T>,
    map: F,
) -> ModelRc<host_contract::TemplatePaneNodeData>
where
    T: Clone + 'static,
    F: FnMut(T) -> host_contract::TemplatePaneNodeData,
{
    model_rc(project_node_vec(nodes, map))
}

pub(super) fn project_node_vec<T, F>(
    nodes: &ModelRc<T>,
    mut map: F,
) -> Vec<host_contract::TemplatePaneNodeData>
where
    T: Clone + 'static,
    F: FnMut(T) -> host_contract::TemplatePaneNodeData,
{
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .map(&mut map)
        .collect()
}
