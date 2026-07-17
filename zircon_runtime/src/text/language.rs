const DEFAULT_TEXT_LOCALE: &str = "en-us";

pub(crate) fn normalize_text_language_tag(language: Option<&str>) -> Option<String> {
    language
        .map(str::trim)
        .filter(|language| !language.is_empty())
        .map(|language| language.replace('_', "-").to_ascii_lowercase())
}

pub(crate) fn default_text_locale() -> String {
    DEFAULT_TEXT_LOCALE.to_string()
}

pub(crate) fn system_text_locale() -> String {
    let locale = sys_locale::get_locale();
    normalize_text_language_tag(locale.as_deref()).unwrap_or_else(default_text_locale)
}

#[cfg(test)]
mod tests {
    use super::{default_text_locale, system_text_locale};

    #[test]
    fn default_text_locale_is_normalized() {
        assert_eq!(default_text_locale(), "en-us");
    }

    #[test]
    fn system_text_locale_is_nonempty_and_normalized() {
        let locale = system_text_locale();

        assert!(!locale.is_empty());
        assert_eq!(locale, locale.to_ascii_lowercase());
        assert!(!locale.contains('_'));
    }
}
