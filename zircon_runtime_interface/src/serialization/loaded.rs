/// Successfully loaded payload plus the version it was migrated from, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loaded<T> {
    pub value: T,
    pub migrated_from: Option<u32>,
}
