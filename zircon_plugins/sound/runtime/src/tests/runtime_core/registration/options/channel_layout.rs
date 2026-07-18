use super::super::super::super::*;

#[test]
fn sound_channel_layout_option_uses_framework_named_layout_vocabulary() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());
    let option = report
        .package_manifest
        .options
        .iter()
        .find(|option| option.key == "sound.channel_layout")
        .expect("sound channel layout option");

    assert_eq!(option.value_type, "enum");
    assert_eq!(option.default_value, "stereo");
    assert_eq!(option.enum_values, ["mono", "stereo"]);
    for option_value in &option.enum_values {
        assert!(
            AudioChannelLayout::from_name(option_value).is_some(),
            "unknown sound channel layout option value {option_value}"
        );
    }
}
