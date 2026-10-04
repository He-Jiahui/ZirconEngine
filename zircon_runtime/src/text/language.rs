use std::{borrow::Cow, cmp::Ordering};

use icu_locale_core::{
    subtags::{Language, Region, Script},
    Locale,
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
#[path = "tests/language.rs"]
mod tests;
