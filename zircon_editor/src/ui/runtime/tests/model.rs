use super::*;

#[test]
fn surface_metadata_index_preserves_preorder_last_write_wins() {
    let projection = RetainedUiProjection {
        document_id: "test.index".to_string(),
        bindings: Vec::new(),
        root: RetainedUiNodeProjection {
            component: "Root".to_string(),
            control_id: None,
            source_path: None,
            source_node_id: None,
            instance_path: None,
            attributes: BTreeMap::new(),
            style_tokens: BTreeMap::new(),
            binding_ids: Vec::new(),
            children: vec![
                RetainedUiNodeProjection {
                    component: "Button".to_string(),
                    control_id: Some("SharedControl".to_string()),
                    source_path: None,
                    source_node_id: None,
                    instance_path: None,
                    attributes: BTreeMap::from([
                        ("preserved".to_string(), Value::Boolean(true)),
                        ("winner".to_string(), Value::String("first".to_string())),
                    ]),
                    style_tokens: BTreeMap::from([("accent".to_string(), "first".to_string())]),
                    binding_ids: Vec::new(),
                    children: Vec::new(),
                },
                RetainedUiNodeProjection {
                    component: "Button".to_string(),
                    control_id: Some("SharedControl".to_string()),
                    source_path: None,
                    source_node_id: None,
                    instance_path: None,
                    attributes: BTreeMap::from([(
                        "winner".to_string(),
                        Value::String("second".to_string()),
                    )]),
                    style_tokens: BTreeMap::from([("accent".to_string(), "second".to_string())]),
                    binding_ids: Vec::new(),
                    children: Vec::new(),
                },
            ],
        },
    };

    let index = projection.surface_metadata_index();
    let (attributes, style_tokens) = index.metadata_for("SharedControl").unwrap();
    assert_eq!(attributes.get("preserved"), Some(&Value::Boolean(true)));
    assert_eq!(
        attributes.get("winner"),
        Some(&Value::String("second".to_string()))
    );
    assert_eq!(style_tokens.get("accent"), Some(&"second".to_string()));
}
