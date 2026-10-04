use std::collections::BTreeMap;

use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

use super::{secure_text_policy, UiSecureTextPolicy};

#[test]
fn password_input_kind_wins_over_explicit_false_alias() {
    let metadata = metadata([
        ("secure", toml::Value::Boolean(false)),
        ("input_kind", toml::Value::String("password".to_string())),
    ]);

    assert_eq!(secure_text_policy(&metadata), UiSecureTextPolicy::Password);
}

#[test]
fn known_plain_input_kind_remains_plain() {
    let metadata = metadata([("input_kind", toml::Value::String("email".to_string()))]);

    assert_eq!(secure_text_policy(&metadata), UiSecureTextPolicy::PlainText);
}

#[test]
fn malformed_secure_alias_and_unknown_input_kind_fail_closed() {
    let malformed = metadata([("secure_input", toml::Value::String("sometimes".to_string()))]);
    let unknown = metadata([(
        "input_kind",
        toml::Value::String("private-token".to_string()),
    )]);

    assert_eq!(secure_text_policy(&malformed), UiSecureTextPolicy::Password);
    assert_eq!(secure_text_policy(&unknown), UiSecureTextPolicy::Password);
}

fn metadata<const N: usize>(values: [(&str, toml::Value); N]) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        attributes: values
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<BTreeMap<_, _>>(),
        ..UiTemplateNodeMetadata::default()
    }
}
