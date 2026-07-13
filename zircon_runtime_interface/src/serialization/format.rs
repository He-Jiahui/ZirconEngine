/// Physical encoding used by a versioned payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Text,
    Binary,
}
