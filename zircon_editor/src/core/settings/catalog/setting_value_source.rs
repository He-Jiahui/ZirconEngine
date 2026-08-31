use super::super::SettingsScope;

/// Identifies which precedence layer supplied one resolved setting value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingValueSource {
    Default,
    Scope(SettingsScope),
}
