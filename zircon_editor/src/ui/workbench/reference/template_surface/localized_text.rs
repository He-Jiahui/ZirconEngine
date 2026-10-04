use super::{EditorWorkbenchTemplateSurface, EditorWorkbenchTemplateSurfaceError};
use crate::core::i18n::{EditorI18nService, EditorLocale};
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct WorkbenchLocalizedTextContext {
    service: Arc<EditorI18nService>,
    last_locale: Option<EditorLocale>,
}

impl std::fmt::Debug for WorkbenchLocalizedTextContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkbenchLocalizedTextContext")
            .field("last_locale", &self.last_locale)
            .finish_non_exhaustive()
    }
}

impl EditorWorkbenchTemplateSurface {
    /// Binds the actual Editor context's locale/catalog authority to this projection.
    pub(crate) fn install_localization_context(&mut self, service: Arc<EditorI18nService>) {
        self.localization_context = Some(WorkbenchLocalizedTextContext {
            service,
            last_locale: None,
        });
    }

    pub(super) fn synchronize_localized_text(
        &mut self,
    ) -> Result<(), EditorWorkbenchTemplateSurfaceError> {
        let Some(context) = self.localization_context.as_mut() else {
            return Ok(());
        };
        let locale = context.service.active_locale();
        if context.last_locale.as_ref() == Some(&locale) {
            return Ok(());
        }
        self.surface.synchronize_localized_text(|reference| {
            context
                .service
                .resolve_localized_text_for_locale(&locale, reference)
                .map(|text| text.as_ref().to_owned())
        })?;
        context.last_locale = Some(locale);
        Ok(())
    }
}

#[cfg(test)]
#[path = "localized_text/tests/cases.rs"]
mod tests;
