use super::*;

#[test]
fn published_wizard_view_model_is_borrowed_without_a_payload_clone() {
    let plan = ExportWizardPipelinePlan::unavailable(
        "desktop_windows",
        EXPORT_WIZARD_DEFAULT_OUT,
        "fixture unavailable",
    );
    let data = BuildExportPaneViewData {
        wizard_view_model: Some(ExportWizardPanelViewModel::from_plan(
            EXPORT_WIZARD_PANEL_JOB_ID,
            &plan,
        )),
        ..BuildExportPaneViewData::default()
    };

    let selected = build_export_wizard_panel_view_model(&data);

    assert!(matches!(&selected, Cow::Borrowed(_)));
    assert!(std::ptr::eq(
        selected.as_ref(),
        data.wizard_view_model
            .as_ref()
            .expect("published wizard view model")
    ));
}

#[test]
fn missing_wizard_view_model_constructs_an_owned_fallback() {
    let data = BuildExportPaneViewData::default();
    let selected = build_export_wizard_panel_view_model(&data);

    assert!(matches!(selected, Cow::Owned(_)));
}
