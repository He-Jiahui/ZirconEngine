use std::collections::HashMap;
use std::hash::Hash;

pub(super) struct CandidateRows<T, K> {
    rows: Vec<Option<T>>,
    indices_by_identity: HashMap<K, Vec<usize>>,
    source_rows_indexed: usize,
}

impl<T, K> CandidateRows<T, K>
where
    K: Eq + Hash,
{
    pub(super) fn from_source(source: &[T], mut identity: impl FnMut(&T) -> K) -> Self
    where
        T: Clone,
    {
        let mut indices_by_identity = HashMap::<K, Vec<usize>>::new();
        let mut rows = Vec::with_capacity(source.len());
        for row in source {
            let index = rows.len();
            indices_by_identity
                .entry(identity(row))
                .or_default()
                .push(index);
            rows.push(Some(row.clone()));
        }
        Self {
            rows,
            indices_by_identity,
            source_rows_indexed: source.len(),
        }
    }

    pub(super) fn append(&mut self, identity: K, row: T) {
        let index = self.rows.len();
        self.rows.push(Some(row));
        self.indices_by_identity
            .entry(identity)
            .or_default()
            .push(index);
    }

    pub(super) fn replace(&mut self, identity: &K, row: T) -> bool {
        let Self {
            rows,
            indices_by_identity,
            ..
        } = self;
        let Some(indices) = indices_by_identity.get(identity) else {
            return false;
        };
        let Some(first_live) = indices.iter().copied().find(|index| rows[*index].is_some()) else {
            return false;
        };
        rows[first_live] = Some(row);
        for index in indices.iter().copied().filter(|index| *index != first_live) {
            rows[index] = None;
        }
        true
    }

    pub(super) fn remove(&mut self, identity: &K) -> bool {
        let Self {
            rows,
            indices_by_identity,
            ..
        } = self;
        let Some(indices) = indices_by_identity.get(identity) else {
            return false;
        };
        let mut removed = false;
        for index in indices {
            removed |= rows[*index].take().is_some();
        }
        removed
    }

    pub(super) fn source_rows_indexed(&self) -> usize {
        self.source_rows_indexed
    }

    pub(super) fn into_rows(self) -> Vec<T> {
        self.rows.into_iter().flatten().collect()
    }
}
