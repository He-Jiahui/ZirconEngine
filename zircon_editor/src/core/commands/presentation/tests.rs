use std::collections::BTreeMap;

use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::{EditorI18nService, EditorLocale, EditorLocalizationBundle};

use super::{
    EditorCommandMenuPath, EditorCommandMenuSegment, EditorCommandMenuSegmentId,
    EditorCommandPresentation,
};

#[test]
fn typed_menu_path_keeps_stable_ids_separate_from_localized_labels() {
    let operation =
        EditorOperationPath::parse("weather.cloud_layer.refresh").expect("valid operation id");
    let path = EditorCommandMenuPath::new(
        EditorCommandMenuSegment::parse("tools", "menu.tools.label").unwrap(),
        [EditorCommandMenuSegment::parse("weather", "menu.tools.weather.label").unwrap()],
        EditorCommandMenuSegment::parse(
            operation.as_str(),
            "command.weather.cloud_layer.refresh.label",
        )
        .unwrap(),
    );

    assert_eq!(
        path.stable_path(),
        "tools/weather/weather.cloud_layer.refresh"
    );
    assert_eq!(
        EditorCommandMenuSegmentId::parse("Weather").unwrap_err(),
        "editor command menu segment id `Weather` must use lowercase dot-separated identifier segments"
    );
}

#[test]
fn plugin_command_presentation_resolves_only_through_its_bound_bundle() {
    let bundle = EditorLocalizationBundle::from_locale_maps(
        "weather.editor",
        BTreeMap::from([
            (
                "en".to_string(),
                BTreeMap::from([
                    (
                        "command.weather.refresh.label".to_string(),
                        "Refresh Weather".to_string(),
                    ),
                    (
                        "command.weather.refresh.description".to_string(),
                        "Refresh the weather simulation".to_string(),
                    ),
                ]),
            ),
            (
                "zh-CN".to_string(),
                BTreeMap::from([
                    (
                        "command.weather.refresh.label".to_string(),
                        "刷新天气".to_string(),
                    ),
                    (
                        "command.weather.refresh.description".to_string(),
                        "刷新天气模拟".to_string(),
                    ),
                ]),
            ),
        ]),
    )
    .unwrap();
    let mut presentation = EditorCommandPresentation::localized(
        "weather.editor",
        "command.weather.refresh.label",
        "command.weather.refresh.description",
    )
    .unwrap();
    let i18n = EditorI18nService::default();
    let chinese = EditorLocale::parse("zh-CN").unwrap();

    assert_eq!(
        presentation.resolve_label(&i18n, &chinese).as_ref(),
        "command.weather.refresh.label"
    );
    presentation.bind_bundle(&bundle).unwrap();
    assert_eq!(
        presentation.resolve_label(&i18n, &chinese).as_ref(),
        "刷新天气"
    );
}
