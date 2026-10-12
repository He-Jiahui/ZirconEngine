use super::*;

#[test]
fn activity_rail_expansion_reserves_raw_and_tab_output_upper_bound() {
    let tabs = model_rc(vec![
        TabData {
            id: "Hierarchy".into(),
            slot: "drawer".into(),
            title: "Hierarchy".into(),
            icon_key: "hierarchy".into(),
            active: true,
            closeable: false,
        },
        TabData {
            id: "Assets".into(),
            slot: "drawer".into(),
            title: "Assets".into(),
            icon_key: "assets".into(),
            active: false,
            closeable: false,
        },
    ]);
    let raw_nodes = (0..3)
        .map(|index| ViewTemplateNodeData {
            control_id: format!("StaticNode{index}").into(),
            ..ViewTemplateNodeData::default()
        })
        .collect::<Vec<_>>();
    let expected_capacity = raw_nodes.len().saturating_add(
        tabs.row_count()
            .saturating_mul(ACTIVITY_RAIL_EXPANDED_NODES_PER_TAB),
    );

    let nodes = expand_activity_rail_button_nodes(raw_nodes, &tabs, &"".into());

    assert_eq!(nodes.len(), 3);
    assert!(
        nodes.capacity() >= expected_capacity,
        "projected activity-rail nodes should reserve the raw/template upper bound"
    );
}
