use std::collections::BTreeMap;

use toml::Value;

use super::projected_value_text;

#[test]
fn search_field_projects_its_query_as_the_rendered_value() {
    let attributes = BTreeMap::from([("query".to_string(), Value::String("strafe".into()))]);

    assert_eq!(
        projected_value_text("search-field", &attributes, &Default::default()),
        "strafe"
    );
}

#[test]
fn collection_value_summaries_do_not_depend_on_nested_contents() {
    let nested_array = Value::Array(vec![
        Value::Array(vec![Value::Integer(1), Value::Integer(2)]),
        Value::Table(Default::default()),
    ]);
    let nested_table = Value::Table(
        [("nested".to_string(), nested_array.clone())]
            .into_iter()
            .collect(),
    );

    for (value, expected) in [(nested_array, "2 items"), (nested_table, "1 entries")] {
        let attributes = BTreeMap::from([("value".to_string(), value)]);
        assert_eq!(
            projected_value_text("list", &attributes, &Default::default()),
            expected
        );
    }
}
