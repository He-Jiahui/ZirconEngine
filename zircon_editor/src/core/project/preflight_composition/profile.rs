/// Selects which project-derived capabilities may enter the post-admission composition plan.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProjectPreflightCompositionProfile {
    #[default]
    Normal,
    Safe,
    Recovery,
}
