use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuerySingleError {
    NoEntities,
    MultipleEntities,
}

impl fmt::Display for QuerySingleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoEntities => formatter.write_str("no entities fit the query"),
            Self::MultipleEntities => formatter.write_str("multiple entities fit the query"),
        }
    }
}

impl std::error::Error for QuerySingleError {}

pub(crate) fn single_from_iter<T>(
    mut iter: impl Iterator<Item = T>,
) -> Result<T, QuerySingleError> {
    let Some(first) = iter.next() else {
        return Err(QuerySingleError::NoEntities);
    };
    if iter.next().is_some() {
        return Err(QuerySingleError::MultipleEntities);
    }
    Ok(first)
}
