#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApplicationActivationState {
    #[default]
    Unknown,
    Active,
    Inactive,
}
