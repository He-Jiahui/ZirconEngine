#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WorldDomain {
    #[default]
    Edit,
    Play,
}
