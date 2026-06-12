use crate::settings::HubLanguage;

pub use super::build::BuildMessageId;
pub use super::delivery::DeliveryMessageId;
pub use super::engine::EngineMessageId;
pub use super::learn::LearnMessageId;
pub use super::process::ProcessMessageId;
pub use super::project::ProjectMessageId;
pub use super::settings::SettingsMessageId;
pub use super::shell::ShellMessageId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HubMessageId {
    Shell(ShellMessageId),
    Project(ProjectMessageId),
    Engine(EngineMessageId),
    Build(BuildMessageId),
    Delivery(DeliveryMessageId),
    Process(ProcessMessageId),
    Settings(SettingsMessageId),
    Learn(LearnMessageId),
}

impl HubMessageId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shell(id) => id.as_str(),
            Self::Project(id) => id.as_str(),
            Self::Engine(id) => id.as_str(),
            Self::Build(id) => id.as_str(),
            Self::Delivery(id) => id.as_str(),
            Self::Process(id) => id.as_str(),
            Self::Settings(id) => id.as_str(),
            Self::Learn(id) => id.as_str(),
        }
    }

    pub fn from_str_id(id: &str) -> Option<Self> {
        Self::all()
            .into_iter()
            .find(|candidate| candidate.as_str() == id)
    }

    pub fn param_count(self) -> usize {
        match self {
            Self::Shell(id) => id.param_count(),
            Self::Project(id) => id.param_count(),
            Self::Engine(id) => id.param_count(),
            Self::Build(id) => id.param_count(),
            Self::Delivery(id) => id.param_count(),
            Self::Process(id) => id.param_count(),
            Self::Settings(id) => id.param_count(),
            Self::Learn(id) => id.param_count(),
        }
    }

    pub fn template(self, language: HubLanguage) -> &'static str {
        match self {
            Self::Shell(id) => id.template(language),
            Self::Project(id) => id.template(language),
            Self::Engine(id) => id.template(language),
            Self::Build(id) => id.template(language),
            Self::Delivery(id) => id.template(language),
            Self::Process(id) => id.template(language),
            Self::Settings(id) => id.template(language),
            Self::Learn(id) => id.template(language),
        }
    }

    pub fn all() -> Vec<Self> {
        let mut ids = Vec::new();
        ids.extend(ShellMessageId::ALL.iter().copied().map(Self::Shell));
        ids.extend(ProjectMessageId::ALL.iter().copied().map(Self::Project));
        ids.extend(EngineMessageId::ALL.iter().copied().map(Self::Engine));
        ids.extend(BuildMessageId::ALL.iter().copied().map(Self::Build));
        ids.extend(DeliveryMessageId::ALL.iter().copied().map(Self::Delivery));
        ids.extend(ProcessMessageId::ALL.iter().copied().map(Self::Process));
        ids.extend(SettingsMessageId::ALL.iter().copied().map(Self::Settings));
        ids.extend(LearnMessageId::ALL.iter().copied().map(Self::Learn));
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::HubMessageId;
    use crate::settings::HubLanguage;

    #[test]
    fn every_message_id_has_bilingual_templates_with_matching_placeholders() {
        for id in HubMessageId::all() {
            for language in [HubLanguage::English, HubLanguage::Chinese] {
                let template = id.template(language);
                assert!(!template.trim().is_empty(), "{id:?} missing {language:?}");
                for index in 0..id.param_count() {
                    assert!(
                        template.contains(&format!("{{{index}}}")),
                        "{id:?} {language:?} template is missing placeholder {{{index}}}: {template}"
                    );
                }
                assert!(
                    !template.contains(&format!("{{{}}}", id.param_count())),
                    "{id:?} {language:?} template has an out-of-range placeholder: {template}"
                );
            }
        }
    }

    #[test]
    fn message_id_round_trips_through_stable_string_ids() {
        for id in HubMessageId::all() {
            assert_eq!(HubMessageId::from_str_id(id.as_str()), Some(id));
        }
    }
}
