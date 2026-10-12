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
