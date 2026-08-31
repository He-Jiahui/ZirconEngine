use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;

use super::super::super::pane_option_projection::structured_options_for_node;
pub(super) fn projected_structured_options(
    attributes: &BTreeMap<String, toml::Value>,
    options: &[String],
) -> Vec<host_contract::TemplatePaneOptionData> {
    if options.is_empty() {
        return Vec::new();
    }

    structured_options_for_node(options, attributes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_option_source_ignores_orphan_option_state() {
        let attributes = BTreeMap::from([
            ("query".to_string(), toml::Value::String("build".into())),
            (
                "hovered_option_id".to_string(),
                toml::Value::String("build.project".into()),
            ),
            (
                "selected_options".to_string(),
                toml::Value::Array(vec![toml::Value::String("build.project".into())]),
            ),
        ]);

        assert!(projected_structured_options(&attributes, &[]).is_empty());
    }
}
