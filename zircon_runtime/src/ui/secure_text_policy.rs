use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

const SECURE_BOOLEAN_KEYS: [&str; 3] = ["secure", "secure_input", "secureInput"];
const PLAIN_INPUT_KINDS: [&str; 6] = ["text", "email", "search", "number", "tel", "url"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum UiSecureTextPolicy {
    #[default]
    PlainText,
    Password,
}

impl UiSecureTextPolicy {
    pub(crate) const fn is_secure(self) -> bool {
        matches!(self, Self::Password)
    }
}

pub(crate) fn secure_text_policy(metadata: &UiTemplateNodeMetadata) -> UiSecureTextPolicy {
    for key in SECURE_BOOLEAN_KEYS {
        let Some(value) = metadata.attributes.get(key) else {
            continue;
        };
        match value.as_bool() {
            Some(true) => return UiSecureTextPolicy::Password,
            Some(false) => {}
            None => return UiSecureTextPolicy::Password,
        }
    }

    if let Some(value) = metadata.attributes.get("input_kind") {
        let Some(input_kind) = value.as_str() else {
            return UiSecureTextPolicy::Password;
        };
        let input_kind = input_kind.trim();
        if input_kind.eq_ignore_ascii_case("password") {
            return UiSecureTextPolicy::Password;
        }
        if !PLAIN_INPUT_KINDS
            .iter()
            .any(|known| input_kind.eq_ignore_ascii_case(known))
        {
            return UiSecureTextPolicy::Password;
        }
    }

    if metadata
        .attributes
        .get("type")
        .and_then(toml::Value::as_str)
        .is_some_and(|input_type| input_type.trim().eq_ignore_ascii_case("password"))
    {
        return UiSecureTextPolicy::Password;
    }
    UiSecureTextPolicy::PlainText
}

#[cfg(test)]
#[path = "tests/secure_text_policy.rs"]
mod tests;
