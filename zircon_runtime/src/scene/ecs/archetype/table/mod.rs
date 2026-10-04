mod column;
mod error;
mod preflighted_row;
mod table;
mod taken_row;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(crate) use error::ArchetypeTableError;
pub(crate) use preflighted_row::ArchetypePreflightedRow;
pub(crate) use table::ArchetypeTable;
pub(crate) use taken_row::ArchetypeTakenRow;
