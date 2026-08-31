use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettingsLocalizationDomain {
    BuiltIn,
    Plugin(Arc<str>),
}
