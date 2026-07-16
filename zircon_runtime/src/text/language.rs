pub(crate) fn normalize_text_language_tag(language: Option<&str>) -> Option<String> {
    language
        .map(str::trim)
        .filter(|language| !language.is_empty())
        .map(|language| language.replace('_', "-").to_ascii_lowercase())
}
