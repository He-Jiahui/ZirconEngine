use std::{borrow::Cow, cmp::Ordering};

use icu_locale_core::{
    Locale,
    subtags::{Language, Region, Script},
};

const DEFAULT_TEXT_LOCALE: &str = "en-US";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextLanguageScriptSubtag([u8; 4]);

impl TextLanguageScriptSubtag {
    pub(crate) fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }

    fn from_icu_script(script: icu_locale_core::subtags::Script) -> Self {
        let bytes: [u8; 4] = script
            .as_str()
            .as_bytes()
            .try_into()
            .expect("ICU4X script subtags are exactly four bytes");
        Self(bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TextLanguageFallbackKey {
    language: Language,
    script: Option<Script>,
    region: Option<Region>,
}

impl TextLanguageFallbackKey {
    pub(crate) fn from_language(language: Option<&str>) -> Option<Self> {
        canonical_text_language(language?)
            .ok()
            .map(|language| language.fallback_key())
    }

    pub(crate) const fn language(self) -> Language {
        self.language
    }

    pub(crate) const fn script(self) -> Option<Script> {
        self.script
    }

    pub(crate) const fn region(self) -> Option<Region> {
        self.region
    }

    pub(crate) fn explicit_script(self) -> Option<TextLanguageScriptSubtag> {
        self.script.map(TextLanguageScriptSubtag::from_icu_script)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextCultureSelector(TextLanguageFallbackKey);

impl TextCultureSelector {
    pub(crate) fn compile(authored: &str) -> Option<Self> {
        let language = canonical_text_language(authored).ok()?;
        (!language.has_variants_or_extensions()).then(|| Self(language.fallback_key()))
    }

    pub(crate) fn matches(self, language: TextLanguageFallbackKey) -> bool {
        let selector = self.0;
        selector.language == language.language
            && selector
                .script
                .is_none_or(|script| Some(script) == language.script)
            && selector
                .region
                .is_none_or(|region| Some(region) == language.region)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalTextLanguage<'a> {
    tag: Cow<'a, str>,
    fallback_key: TextLanguageFallbackKey,
    has_variants_or_extensions: bool,
}

impl<'a> CanonicalTextLanguage<'a> {
    pub(crate) fn explicit_script(&self) -> Option<TextLanguageScriptSubtag> {
        self.fallback_key.explicit_script()
    }

    pub(crate) const fn fallback_key(&self) -> TextLanguageFallbackKey {
        self.fallback_key
    }

    const fn has_variants_or_extensions(&self) -> bool {
        self.has_variants_or_extensions
    }

    pub(crate) fn into_tag(self) -> Cow<'a, str> {
        self.tag
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum TextLanguageTagError {
    #[error("text language tag is empty")]
    Empty,
    #[error("text language tag is not valid BCP 47 syntax")]
    InvalidSyntax,
}

pub(crate) fn canonical_text_language(
    language: &str,
) -> Result<CanonicalTextLanguage<'_>, TextLanguageTagError> {
    let trimmed = language.trim();
    if trimmed.is_empty() {
        return Err(TextLanguageTagError::Empty);
    }
    let hyphenated = if trimmed.contains('_') {
        Cow::Owned(trimmed.replace('_', "-"))
    } else {
        Cow::Borrowed(trimmed)
    };
    let locale = Locale::try_from_str(hyphenated.as_ref())
        .map_err(|_| TextLanguageTagError::InvalidSyntax)?;
    let fallback_key = TextLanguageFallbackKey {
        language: locale.id.language,
        script: locale.id.script,
        region: locale.id.region,
    };
    let has_variants_or_extensions =
        !locale.id.variants.is_empty() || !locale.extensions.is_empty();
    let tag = match hyphenated {
        Cow::Borrowed(input)
            if input.len() == language.len()
                && locale.strict_cmp(language.as_bytes()) == Ordering::Equal =>
        {
            Cow::Borrowed(language)
        }
        Cow::Owned(input) if locale.strict_cmp(input.as_bytes()) == Ordering::Equal => {
            Cow::Owned(input)
        }
        _ => Cow::Owned(locale.to_string()),
    };
    Ok(CanonicalTextLanguage {
        tag,
        fallback_key,
        has_variants_or_extensions,
    })
}

pub(crate) fn canonical_text_language_tag(
    language: &str,
) -> Result<Cow<'_, str>, TextLanguageTagError> {
    canonical_text_language(language).map(CanonicalTextLanguage::into_tag)
}

pub(crate) fn text_language_script_subtag(
    language: Option<&str>,
) -> Option<TextLanguageScriptSubtag> {
    canonical_text_language(language?).ok()?.explicit_script()
}

pub(crate) fn normalize_text_language_tag(language: Option<&str>) -> Option<String> {
    language
        .and_then(|language| canonical_text_language_tag(language).ok())
        .map(Cow::into_owned)
}

pub(crate) fn text_language_cache_identity(language: Option<&str>) -> Option<String> {
    language.map(|language| {
        canonical_text_language_tag(language)
            .map(Cow::into_owned)
            .unwrap_or_else(|_| language.to_owned())
    })
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
    use std::borrow::Cow;

    use super::{
        TextCultureSelector, TextLanguageFallbackKey, canonical_text_language, default_text_locale,
        normalize_text_language_tag, system_text_locale, text_language_cache_identity,
        text_language_script_subtag,
    };

    #[test]
    fn default_text_locale_is_normalized() {
        assert_eq!(default_text_locale(), "en-US");
    }

    #[test]
    fn text_language_uses_bcp47_casing_and_rejects_invalid_syntax() {
        assert_eq!(
            normalize_text_language_tag(Some(" ZH_Hans_CN ")).as_deref(),
            Some("zh-Hans-CN")
        );
        assert_eq!(
            normalize_text_language_tag(Some("sr_latn_rs")).as_deref(),
            Some("sr-Latn-RS")
        );
        assert_eq!(normalize_text_language_tag(Some("en--US")), None);
        assert_eq!(normalize_text_language_tag(Some("not a tag")), None);
        assert_eq!(normalize_text_language_tag(Some("   ")), None);
        assert_eq!(normalize_text_language_tag(None), None);
    }

    #[test]
    fn cache_identity_canonicalizes_valid_input_and_preserves_invalid_input() {
        assert_eq!(
            text_language_cache_identity(Some("ZH_hans_cn")).as_deref(),
            Some("zh-Hans-CN")
        );
        assert_eq!(
            text_language_cache_identity(Some("en--US")).as_deref(),
            Some("en--US")
        );
        assert_eq!(text_language_cache_identity(None), None);
    }

    #[test]
    fn canonical_language_borrows_an_already_canonical_tag() {
        let language = canonical_text_language("en-US").expect("canonical language parses");

        assert!(matches!(language.into_tag(), Cow::Borrowed("en-US")));
    }

    #[test]
    fn explicit_script_projection_uses_the_validated_language_owner() {
        let script = text_language_script_subtag(Some(" JA_hira_jp "))
            .expect("canonical language keeps an explicit script");
        assert_eq!(script.as_str(), Some("Hira"));
        assert_eq!(text_language_script_subtag(Some("ja-x-Kana")), None);
        assert_eq!(text_language_script_subtag(Some("ja-u-ca-japanese")), None);
        assert_eq!(text_language_script_subtag(Some("not a tag")), None);
        assert_eq!(text_language_script_subtag(None), None);
    }

    #[test]
    fn culture_selectors_follow_language_script_region_parent_combinations() {
        let language = TextLanguageFallbackKey::from_language(Some("zh-Hans-CN"))
            .expect("request language has a fallback identity");

        assert!(
            TextCultureSelector::compile("zh-Hans-CN")
                .expect("exact selector")
                .matches(language)
        );
        assert!(
            TextCultureSelector::compile("zh-CN")
                .expect("region parent selector")
                .matches(language)
        );
        assert!(
            TextCultureSelector::compile("zh-Hans")
                .expect("script parent selector")
                .matches(language)
        );
        assert!(
            TextCultureSelector::compile("zh")
                .expect("language parent selector")
                .matches(language)
        );
        assert!(
            !TextCultureSelector::compile("ja")
                .expect("other-language selector")
                .matches(language)
        );
        assert_eq!(TextCultureSelector::compile("zh-u-ca-chinese"), None);
    }

    #[test]
    fn system_text_locale_is_nonempty_and_normalized() {
        let locale = system_text_locale();

        assert!(!locale.is_empty());
        assert!(!locale.contains('_'));
        assert_eq!(
            normalize_text_language_tag(Some(&locale)).as_deref(),
            Some(locale.as_str())
        );
    }
}
