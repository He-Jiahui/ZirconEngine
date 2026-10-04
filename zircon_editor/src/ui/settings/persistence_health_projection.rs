use crate::core::i18n::EditorI18nService;
use crate::core::settings::{
    SettingsPersistenceDocumentHealth, SettingsPersistenceHealthSnapshot,
    SettingsPersistenceHealthStatus, SettingsPersistenceTerminal, SettingsScope,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SettingsPersistenceHealthProjection {
    generation: u64,
    retry_scope: Option<SettingsScope>,
    status_text: String,
}

impl SettingsPersistenceHealthProjection {
    pub(crate) fn capture(
        snapshot: SettingsPersistenceHealthSnapshot,
        i18n: &EditorI18nService,
    ) -> Self {
        let documents = [snapshot.project(), snapshot.user()];
        let actionable = documents
            .iter()
            .copied()
            .find(|document| document.status().is_retryable());
        let display_only = || {
            documents.iter().copied().find(|document| {
                matches!(
                    document.status(),
                    SettingsPersistenceHealthStatus::Terminal(
                        SettingsPersistenceTerminal::BlockedInvalid
                            | SettingsPersistenceTerminal::MissingWriteDisposition
                    )
                )
            })
        };
        let Some(document) = actionable.or_else(display_only) else {
            return Self {
                generation: snapshot.generation(),
                retry_scope: None,
                status_text: String::new(),
            };
        };
        let label_key = match document.scope() {
            SettingsScope::Project => "settings.persistence.project.label",
            SettingsScope::User => "settings.persistence.user.label",
            SettingsScope::Session => unreachable!("session settings are not persistent"),
        };
        let message_key = persistence_message_key(document);
        Self {
            generation: snapshot.generation(),
            retry_scope: document.status().is_retryable().then_some(document.scope()),
            status_text: format!(
                "{}: {}",
                i18n.translate(label_key),
                i18n.translate(message_key)
            ),
        }
    }

    pub(crate) const fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) const fn retry_scope_name(&self) -> &'static str {
        match self.retry_scope {
            Some(SettingsScope::Project) => "project",
            Some(SettingsScope::User) => "user",
            Some(SettingsScope::Session) | None => "",
        }
    }

    pub(crate) fn status_text(&self) -> &str {
        &self.status_text
    }
}

fn persistence_message_key(document: SettingsPersistenceDocumentHealth) -> &'static str {
    match document.status() {
        SettingsPersistenceHealthStatus::PendingAdmission(_) => {
            "settings.persistence.pending.status"
        }
        SettingsPersistenceHealthStatus::Terminal(
            SettingsPersistenceTerminal::Failed(_)
            | SettingsPersistenceTerminal::BlockedInvalid
            | SettingsPersistenceTerminal::MissingWriteDisposition,
        ) => "settings.persistence.failed.status",
        _ => unreachable!("projection accepts only retryable persistence states"),
    }
}
