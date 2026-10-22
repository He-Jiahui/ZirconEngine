use std::sync::{Arc, OnceLock};

use crate::core::i18n::EditorI18nService;
use zircon_runtime_interface::ui::template::UiLocalizedTextRef;

/// Bootstrap uses the canonical catalog's default locale before settings exist.
/// No runtime, asset staging, GPU, or project is needed to resolve these references.
pub(crate) fn startup_text(key: &str) -> Arc<str> {
    static BOOTSTRAP_I18N: OnceLock<EditorI18nService> = OnceLock::new();
    let i18n = BOOTSTRAP_I18N.get_or_init(EditorI18nService::default);
    let reference = UiLocalizedTextRef {
        key: key.to_owned(),
        table: Some("editor".to_owned()),
        fallback: None,
    };
    i18n.resolve_localized_text_for_locale(&i18n.active_locale(), &reference)
        .unwrap_or_else(|_| Arc::from(key))
}
