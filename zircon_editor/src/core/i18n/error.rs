use thiserror::Error;

#[derive(Debug, Error)]
pub enum EditorI18nError {
    #[error("editor locale `{0}` is not a valid language tag")]
    InvalidLocale(String),
    #[error("editor translation key `{0}` is not valid")]
    InvalidTranslationKey(String),
    #[error("editor translation `{0}` must not be empty")]
    EmptyTranslation(String),
    #[error("editor translation bundle is invalid: {0}")]
    InvalidBundle(String),
    #[error("editor translation bundle repeats locale `{0}`")]
    DuplicateLocale(String),
    #[error("editor translation bundles must provide the English fallback")]
    MissingEnglishFallback,
    #[error("editor translation bundle for locale `{0}` is unavailable")]
    UnavailableLocale(String),
}
