use super::*;

#[test]
fn mui_feedback_utility_classes_match_alert_and_snackbar_selectors() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_FEEDBACK_UTILITY_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let alert = &root.children[0];
    let snackbar = &root.children[1];
    let default_snackbar = &root.children[2];
    let snackbar_content = &root.children[3];
    let alert_title = &root.children[4];
    let skeleton = &root.children[5];

    assert_eq!(str_attr(alert, "validation_level"), Some("warning"));
    assert!(
        str_attr(alert, "component_variant").is_some_and(|value| {
            ["filled", "warning", "colorWarning", "hasIcon", "hasAction"]
                .iter()
                .all(|token| value.split_whitespace().any(|part| part == *token))
        }),
        "Alert root should carry retained severity and slot metadata"
    );
    assert_classes(
        alert,
        &["MuiAlert-root", "MuiAlert-filled", "MuiAlert-colorWarning"],
    );
    assert!(
        !alert
            .classes
            .iter()
            .any(|class_name| class_name == "MuiAlert-filledWarning"),
        "MUI v9 Alert utility classes no longer emit variant+severity combo classes"
    );
    assert!(
        !alert
            .classes
            .iter()
            .any(|class_name| class_name == "MuiAlert-colorPrimary"
                || class_name == "MuiAlert-sizeMedium"),
        "Alert should not inherit generic MUI color/size classes that local Alert.js does not emit"
    );
    let alert_icon = &alert.children[0];
    assert_eq!(str_attr(alert_icon, "text_tone"), Some("warning"));
    assert_classes(alert_icon, &["MuiAlert-icon", "alert-icon-extra"]);
    assert!(
        str_attr(alert_icon, "component_variant").is_some_and(|value| value
            .split_whitespace()
            .all(|part| part != "")
            && ["muiAlertSlot", "alertSlotIcon"]
                .iter()
                .all(|token| value.split_whitespace().any(|part| part == *token))),
        "Alert icon slot should carry retained hide metadata"
    );
    let alert_action = &alert.children[1];
    assert_eq!(str_attr(alert_action, "text_tone"), Some("warning"));
    assert_classes(alert_action, &["MuiAlert-action", "alert-action-extra"]);
    assert!(
        str_attr(alert_action, "component_variant").is_some_and(|value| [
            "muiAlertSlot",
            "alertSlotAction"
        ]
        .iter()
        .all(|token| value.split_whitespace().any(|part| part == *token))),
        "Alert action slot should carry retained hide metadata"
    );

    assert_eq!(str_attr(snackbar, "surface_variant"), Some("snackbar"));
    assert_classes(
        snackbar,
        &[
            "MuiSnackbar-root",
            "MuiSnackbar-anchorOriginTopRight",
            "Mui-open",
        ],
    );
    assert!(
        !snackbar
            .classes
            .iter()
            .any(|class_name| class_name == "MuiSnackbar-colorPrimary"
                || class_name == "MuiSnackbar-sizeMedium"),
        "Snackbar should only emit root and anchor utility classes"
    );
    assert_classes(
        default_snackbar,
        &[
            "MuiSnackbar-root",
            "MuiSnackbar-anchorOriginBottomLeft",
            "Mui-open",
        ],
    );
    assert_classes(snackbar_content, &["MuiSnackbarContent-root"]);
    assert!(
        !snackbar_content
            .classes
            .iter()
            .any(|class_name| class_name == "MuiSnackbarContent-colorPrimary"
                || class_name == "MuiSnackbarContent-sizeMedium"),
        "SnackbarContent should not inherit generic MUI color/size classes"
    );
    let snackbar_action = &snackbar_content.children[0];
    assert_eq!(str_attr(snackbar_action, "text_tone"), Some("warning"));
    assert_classes(
        snackbar_action,
        &["MuiSnackbarContent-action", "snackbar-action-extra"],
    );
    assert_classes(alert_title, &["MuiAlertTitle-root"]);
    assert!(
        !alert_title
            .classes
            .iter()
            .any(|class_name| class_name == "MuiAlertTitle-colorPrimary"
                || class_name == "MuiAlertTitle-sizeMedium"),
        "AlertTitle should only emit local MUI root utility classes"
    );

    assert_eq!(str_attr(skeleton, "validation_level"), Some("info"));
    assert_classes(
        skeleton,
        &[
            "MuiSkeleton-root",
            "MuiSkeleton-rounded",
            "MuiSkeleton-wave",
            "MuiSkeleton-withChildren",
            "MuiSkeleton-fitContent",
            "MuiSkeleton-heightAuto",
        ],
    );
    assert_no_classes(
        skeleton,
        &["MuiSkeleton-colorPrimary", "MuiSkeleton-sizeMedium"],
    );
    assert!(
        str_attr(skeleton, "component_variant").is_some_and(|value| {
            [
                "rounded",
                "wave",
                "withChildren",
                "fitContent",
                "heightAuto",
            ]
            .iter()
            .all(|token| value.split_whitespace().any(|part| part == *token))
        }),
        "Skeleton root should carry retained painter metadata"
    );
    let skeleton_child = &skeleton.children[0];
    assert!(
        str_attr(skeleton_child, "component_variant").is_some_and(|value| value
            .split_whitespace()
            .any(|part| part == "muiSkeletonChild")),
        "Skeleton child should carry retained painter hide metadata"
    );
    assert_classes(skeleton_child, &["MuiLabel-root"]);
}
