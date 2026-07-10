/// Normalize an optional BCP 47 text-language tag for cache and fallback identity.
///
/// The text stack accepts underscore-separated application locale values at its
/// boundary, but stores a lowercase hyphen-separated identity. Empty values are
/// represented by `None` rather than becoming a distinct cache entry.
pub fn normalize_ui_text_language_tag(language: Option<&str>) -> Option<String> {
    language
        .map(str::trim)
        .filter(|language| !language.is_empty())
        .map(|language| language.replace('_', "-").to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::normalize_ui_text_language_tag;

    #[test]
    fn text_language_normalizes_case_separator_and_empty_values() {
        assert_eq!(
            normalize_ui_text_language_tag(Some(" ZH_Hans_CN ")).as_deref(),
            Some("zh-hans-cn")
        );
        assert_eq!(normalize_ui_text_language_tag(Some("   ")), None);
        assert_eq!(normalize_ui_text_language_tag(None), None);
    }
}
