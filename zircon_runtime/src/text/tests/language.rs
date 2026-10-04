use std::borrow::Cow;

use super::{
    canonical_text_language, default_text_locale, normalize_text_language_tag, system_text_locale,
    text_language_cache_identity, text_language_script_subtag, TextCultureSelector,
    TextLanguageFallbackKey,
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

    assert!(TextCultureSelector::compile("zh-Hans-CN")
        .expect("exact selector")
        .matches(language));
    assert!(TextCultureSelector::compile("zh-CN")
        .expect("region parent selector")
        .matches(language));
    assert!(TextCultureSelector::compile("zh-Hans")
        .expect("script parent selector")
        .matches(language));
    assert!(TextCultureSelector::compile("zh")
        .expect("language parent selector")
        .matches(language));
    assert!(!TextCultureSelector::compile("ja")
        .expect("other-language selector")
        .matches(language));
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
