use super::super::support::*;
use crate::ui::workbench::autolayout::{minimum_document_width_fraction, WorkbenchChromeMetrics};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn componentized_workbench_layout_collapses_right_drawer_shell_at_narrow_width() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge =
        match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(640.0, 420.0)) {
            Ok(bridge) => bridge,
            Err(error) => panic!("workbench bridge should build: {error:?}"),
        };
    if let Err(error) =
        bridge.recompute_layout_with_workbench_model(UiSize::new(640.0, 420.0), &model, &metrics)
    {
        panic!("narrow workbench layout should recompute: {error:?}");
    }

    let narrow_frames = bridge.layout_frames();
    assert_eq!(narrow_frames.right_drawer_shell_frame, None);
    assert_eq!(narrow_frames.right_drawer_content_frame, None);

    if let Err(error) =
        bridge.recompute_layout_with_workbench_model(UiSize::new(900.0, 620.0), &model, &metrics)
    {
        panic!("regular workbench layout should recompute: {error:?}");
    }

    let regular_frames = bridge.layout_frames();
    let Some(right_shell_frame) = regular_frames.right_drawer_shell_frame else {
        panic!("right drawer shell should render at regular width");
    };
    let Some(right_content_frame) = regular_frames.right_drawer_content_frame else {
        panic!("right drawer content should render at regular width");
    };
    assert!(right_shell_frame.width > 0.0);
    assert!(right_content_frame.width > 0.0);
}

#[test]
fn mounted_workbench_batches_state_projection_into_one_layout_pass() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");
    let before = bridge.layout_pass_count();

    bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .expect("workbench layout should recompute");

    assert_eq!(bridge.layout_pass_count() - before, 1);
}

#[test]
fn componentized_regular_workbench_reserves_half_width_for_the_document() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let shell_size = UiSize::new(900.0, 620.0);
    let mut bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };
    if let Err(error) = bridge.recompute_layout_with_workbench_model(shell_size, &model, &metrics) {
        panic!("regular workbench layout should recompute: {error:?}");
    }

    let frames = bridge.layout_frames();
    let Some(document) = frames.document_region_frame else {
        panic!("regular workbench should expose its document region");
    };
    assert!(
        frames.left_region_frame.is_some() && frames.right_region_frame.is_some(),
        "regular width should retain both side regions"
    );
    assert!(
        document.width >= shell_size.width * minimum_document_width_fraction(),
        "regular workbench should reserve half of the shell for the document: {document:?}"
    );
}

#[test]
fn componentized_workbench_layout_collapses_right_drawer_shell_by_logical_width_under_scale() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge =
        match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 840.0)) {
            Ok(bridge) => bridge,
            Err(error) => panic!("workbench bridge should build: {error:?}"),
        };
    if let Err(error) = bridge.recompute_layout_with_workbench_model_at_scale(
        UiSize::new(1280.0, 840.0),
        2.0,
        &model,
        &metrics,
    ) {
        panic!("scaled narrow workbench layout should recompute: {error:?}");
    }

    let scaled_narrow_frames = bridge.layout_frames();
    assert_eq!(scaled_narrow_frames.right_drawer_shell_frame, None);
    assert_eq!(scaled_narrow_frames.right_drawer_content_frame, None);

    if let Err(error) = bridge.recompute_layout_with_workbench_model_at_scale(
        UiSize::new(1800.0, 1240.0),
        2.0,
        &model,
        &metrics,
    ) {
        panic!("scaled regular workbench layout should recompute: {error:?}");
    }

    let scaled_regular_frames = bridge.layout_frames();
    assert!(scaled_regular_frames.right_drawer_shell_frame.is_some());
    assert!(scaled_regular_frames.right_drawer_content_frame.is_some());
}
