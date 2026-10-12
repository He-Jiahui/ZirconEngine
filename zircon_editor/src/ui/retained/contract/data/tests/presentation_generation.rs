use super::*;

#[test]
fn paint_scope_projects_generation_state_without_materializing_structure() {
    let structure = Arc::new(HostWindowPresentationData::default());
    let menu_state = Arc::new(HostMenuStateData {
        open_menu_index: 7,
        ..HostMenuStateData::default()
    });
    let generation = HostPresentationGeneration::new(
        Arc::clone(&structure),
        menu_state,
        Arc::new(HostPageOverflowMenuStateData::default()),
        Arc::new(HostDockOverflowMenuStateData::default()),
        Arc::new(HostPaneInteractionStateData::default()),
        Arc::new(HostTextInputFocusData::default()),
        HostViewportImageSet::default(),
        Arc::new(HostWorkbenchHitIndex::from_presentation(&structure)),
        crate::ui::retained_host::host_contract::paint_theme::capture_host_paint_theme_snapshot(),
        Arc::new("diagnostics".to_owned()),
        1,
        2,
        3,
        4,
        5,
    );

    assert_eq!(paint_menu_state(&structure).open_menu_index, -1);
    {
        let _scope = generation.enter_paint_scope();
        assert_eq!(paint_menu_state(&structure).open_menu_index, 7);
        assert_eq!(paint_debug_refresh_rate(&structure).as_str(), "diagnostics");
        assert_eq!(structure.menu_state.open_menu_index, -1);
    }
    assert_eq!(paint_menu_state(&structure).open_menu_index, -1);
}

#[test]
fn paint_scope_translates_host_damage_into_workbench_local_coordinates() {
    let nodes = ModelRc::from(std::rc::Rc::new(
        crate::ui::retained_host::primitives::VecModel::from(vec![TemplatePaneNodeData {
            frame: crate::ui::retained_host::host_contract::data::TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            },
            ..TemplatePaneNodeData::default()
        }]),
    ));
    let mut structure_data = HostWindowPresentationData::default();
    structure_data
        .host_scene_data
        .document_dock
        .pane
        .template_v2
        .nodes = nodes.clone();
    let structure = Arc::new(structure_data);
    let generation = HostPresentationGeneration::new(
        Arc::clone(&structure),
        Arc::new(HostMenuStateData::default()),
        Arc::new(HostPageOverflowMenuStateData::default()),
        Arc::new(HostDockOverflowMenuStateData::default()),
        Arc::new(HostPaneInteractionStateData::default()),
        Arc::new(HostTextInputFocusData::default()),
        HostViewportImageSet::default(),
        Arc::new(HostWorkbenchHitIndex::from_presentation(&structure)),
        crate::ui::retained_host::host_contract::paint_theme::capture_host_paint_theme_snapshot(),
        Arc::new(SharedString::default()),
        1,
        1,
        1,
        1,
        1,
    );

    let _scope = generation.enter_paint_scope();
    let mut rows = Vec::new();
    let used_index = visit_paint_workbench_rows(
        &nodes,
        &FrameRect {
            x: 100.0,
            y: 200.0,
            width: 300.0,
            height: 300.0,
        },
        &FrameRect {
            x: 110.0,
            y: 220.0,
            width: 30.0,
            height: 40.0,
        },
        &mut |row| rows.push(row),
    );

    assert!(used_index);
    assert_eq!(rows, vec![0]);
}
