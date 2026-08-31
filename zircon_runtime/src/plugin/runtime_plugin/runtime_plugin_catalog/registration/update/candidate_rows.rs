use std::collections::HashMap;
use std::hash::Hash;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct CandidateRows<T, K> {
    rows: Vec<Option<T>>,
    indices_by_identity: HashMap<K, Vec<usize>>,
    source_rows_indexed: usize,
}

impl<T, K> CandidateRows<T, K>
where
    K: Eq + Hash,
{
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn from_source(
        source: &[T],
        mut identity: impl FnMut(&T) -> K,
    ) -> Self
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

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn append(
        &mut self,
        identity: K,
        row: T,
    ) {
        let index = self.rows.len();
        self.rows.push(Some(row));
        self.indices_by_identity
            .entry(identity)
            .or_default()
            .push(index);
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn replace(
        &mut self,
        identity: &K,
        row: T,
    ) -> bool {
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

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn remove(
        &mut self,
        identity: &K,
    ) -> bool {
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

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn source_rows_indexed(
        &self,
    ) -> usize {
        self.source_rows_indexed
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn into_rows(self) -> Vec<T> {
        self.rows.into_iter().flatten().collect()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::CandidateRows;

    struct CountedRow {
        id: usize,
        clones: Arc<AtomicUsize>,
    }

    impl Clone for CountedRow {
        fn clone(&self) -> Self {
            self.clones.fetch_add(1, Ordering::Relaxed);
            Self {
                id: self.id,
                clones: Arc::clone(&self.clones),
            }
        }
    }

    #[test]
    fn source_rows_are_cloned_and_indexed_exactly_once() {
        let clones = Arc::new(AtomicUsize::new(0));
        let source = (0..1_024)
            .map(|id| CountedRow {
                id,
                clones: Arc::clone(&clones),
            })
            .collect::<Vec<_>>();

        let candidate = CandidateRows::from_source(&source, |row| row.id);

        assert_eq!(candidate.source_rows_indexed(), source.len());
        assert_eq!(clones.load(Ordering::Relaxed), source.len());
        assert_eq!(candidate.into_rows().len(), source.len());
    }
}
