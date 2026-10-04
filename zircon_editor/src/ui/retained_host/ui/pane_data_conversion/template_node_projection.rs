//! project_nodes 将原生节点映射为宿主 ModelRc 时保留来源元数据，供正常原生路径与模板失败回退沿用；project_node_vec 返回 Vec 节点，不携带 ModelRc 元数据。
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
#[path = "tests/template_node_projection.rs"]
mod tests;
