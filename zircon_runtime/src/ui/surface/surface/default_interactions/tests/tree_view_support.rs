use std::collections::HashSet;

use super::{collect_disabled_option_ids, collect_tree_node_ids};

#[test]
fn metadata_tree_ids_borrow_first_occurrence_and_preserve_order() {
    let value = toml::Value::Array(vec![
        toml::Value::String("root".to_string()),
        toml::Value::String("root".to_string()),
        toml::Value::String("child".to_string()),
    ]);
    let first_root = match &value {
        toml::Value::Array(values) => match &values[0] {
            toml::Value::String(value) => value.as_ptr(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    let mut ids = Vec::new();
    let mut seen = HashSet::new();

    collect_tree_node_ids(&value, &mut ids, &mut seen);

    assert_eq!(ids, ["root", "child"]);
    assert_eq!(ids[0].as_ptr(), first_root);
}

#[test]
fn metadata_disabled_index_preserves_table_identity_aliases() {
    let value = toml::Value::Array(vec![
        toml::Value::String("root".to_string()),
        toml::Value::Table(
            [(
                "nodeId".to_string(),
                toml::Value::String("child".to_string()),
            )]
            .into_iter()
            .collect(),
        ),
    ]);
    let mut disabled = HashSet::new();

    collect_disabled_option_ids(&value, &mut disabled);

    assert_eq!(disabled.len(), 2);
    assert!(disabled.contains("root"));
    assert!(disabled.contains("child"));
}
