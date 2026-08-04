mod column;
mod error;
mod table;
mod taken_row;

#[cfg(test)]
mod tests;

pub(crate) use error::ArchetypeTableError;
pub(crate) use table::ArchetypeTable;
pub(crate) use taken_row::ArchetypeTakenRow;
