use std::collections::BTreeMap;

mod model;
mod options;
mod search;
mod selection;
mod structured;
mod tree;

pub(super) use self::model::ProjectedSelectionOptions;

pub(super) fn projected_selection_options(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedSelectionOptions {
    let options = options::projected_options(component_role, attributes);
    let tree_state = tree::projected_tree_state(attributes);

    ProjectedSelectionOptions {
        selection_state: selection::projected_selection_state(attributes),
        search_query: search::projected_search_query(attributes),
        selected: selection::projected_selected(attributes),
        tree_depth: tree_state.depth,
        tree_indent_px: tree_state.indent_px,
        options_text: options.join(", "),
        structured_options: structured::projected_structured_options(
            component_role,
            attributes,
            &options,
        ),
        options,
    }
}
