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
    let (options, structured_options) =
        super::command_palette::projected_command_palette_option_rows(component_role, attributes)
            .or_else(|| {
                super::notification_center::projected_notification_center_option_rows(
                    component_role,
                    attributes,
                )
            })
            .unwrap_or_else(|| {
                let options = options::projected_options(attributes);
                let structured_options =
                    structured::projected_structured_options(attributes, &options);
                (options, structured_options)
            });
    let tree_state = tree::projected_tree_state(attributes);
    let options_text = options.join(", ");

    ProjectedSelectionOptions {
        selection_state: selection::projected_selection_state(attributes),
        search_query: search::projected_search_query(attributes),
        selected: selection::projected_selected(attributes),
        tree_depth: tree_state.depth,
        tree_indent_px: tree_state.indent_px,
        options_text,
        structured_options,
        options,
    }
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn specialized_options_are_projected_once_per_node() {
        let source = include_str!("mod.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");

        assert!(implementation.contains("projected_command_palette_option_rows"));
        assert!(implementation.contains("projected_notification_center_option_rows"));
        assert!(!implementation.contains("projected_command_palette_options("));
        assert!(!implementation.contains("projected_notification_center_options("));
    }
}
