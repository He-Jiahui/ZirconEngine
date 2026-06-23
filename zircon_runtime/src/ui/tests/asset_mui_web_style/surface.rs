use super::*;

#[test]
fn mui_surface_utility_classes_match_paper_card_and_app_bar_selectors() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_SURFACE_UTILITY_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let paper = &root.children[0];
    let outlined_paper = &root.children[1];
    let app_bar = &root.children[2];
    let toolbar = &root.children[3];
    let card = &root.children[4];
    let card_header = &root.children[5];
    let card_actions = &root.children[6];
    let card_media = &root.children[7];
    let card_action_area = &root.children[8];

    assert_eq!(str_attr(paper, "surface_variant"), Some("popup"));
    assert_classes(
        paper,
        &[
            "MuiPaper-root",
            "MuiPaper-elevation",
            "MuiPaper-rounded",
            "MuiPaper-elevation3",
        ],
    );
    assert_no_classes(paper, &["MuiPaper-colorPrimary", "MuiPaper-sizeMedium"]);

    assert_classes(outlined_paper, &["MuiPaper-root", "MuiPaper-outlined"]);
    assert_no_classes(outlined_paper, &["MuiPaper-rounded", "MuiPaper-elevation1"]);

    assert_eq!(str_attr(app_bar, "surface_variant"), Some("primary"));
    assert_eq!(str_attr(app_bar, "text_tone"), Some("inverse"));
    assert_classes(
        app_bar,
        &[
            "MuiAppBar-root",
            "MuiAppBar-colorPrimary",
            "MuiAppBar-positionFixed",
            "mui-fixed",
        ],
    );
    assert_no_classes(app_bar, &["MuiAppBar-sizeMedium"]);

    assert_eq!(str_attr(toolbar, "text_align"), Some("center"));
    assert_classes(
        toolbar,
        &[
            "MuiToolbar-root",
            "MuiToolbar-gutters",
            "MuiToolbar-regular",
        ],
    );
    assert_no_classes(
        toolbar,
        &["MuiToolbar-colorPrimary", "MuiToolbar-sizeMedium"],
    );

    assert_classes(card, &["MuiCard-root"]);
    assert_no_classes(
        card,
        &[
            "MuiCard-outlined",
            "MuiCard-colorPrimary",
            "MuiCard-sizeMedium",
        ],
    );

    assert_classes(card_header, &["MuiCardHeader-root"]);
    assert_no_classes(
        card_header,
        &["MuiCardHeader-colorPrimary", "MuiCardHeader-sizeMedium"],
    );
    let card_title = &card_header.children[0];
    assert_eq!(str_attr(card_title, "text"), Some("Slot Title"));
    assert_eq!(str_attr(card_title, "text_tone"), Some("info"));
    assert_classes(
        card_title,
        &["MuiLabel-root", "MuiCardHeader-title", "card-title-extra"],
    );

    assert_eq!(float_attr(card_actions, "border_width"), Some(2.0));
    assert_classes(
        card_actions,
        &["MuiCardActions-root", "MuiCardActions-spacing"],
    );
    assert_no_classes(
        card_actions,
        &["MuiCardActions-colorPrimary", "MuiCardActions-sizeMedium"],
    );

    assert_eq!(str_attr(card_media, "overflow"), Some("clip"));
    assert_classes(
        card_media,
        &[
            "MuiCardMedia-root",
            "MuiCardMedia-media",
            "MuiCardMedia-img",
        ],
    );
    assert_no_classes(
        card_media,
        &["MuiCardMedia-colorPrimary", "MuiCardMedia-sizeMedium"],
    );

    assert_classes(
        card_action_area,
        &[
            "MuiCardActionArea-root",
            "MuiCardActionArea-focusVisible",
            "keyboard-focus",
        ],
    );
    assert_no_classes(
        card_action_area,
        &[
            "MuiCardActionArea-colorPrimary",
            "MuiCardActionArea-sizeMedium",
        ],
    );
    let focus_highlight = &card_action_area.children[0];
    assert_eq!(
        bool_attr(focus_highlight, "state_layer_enabled"),
        Some(true)
    );
    assert_classes(
        focus_highlight,
        &["MuiCardActionArea-focusHighlight", "focus-highlight-extra"],
    );
}
