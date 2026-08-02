use std::sync::Arc;

use super::ObserverId;

pub(super) fn append_observer_to_bucket<T>(bucket: Option<&Arc<[T]>>, observer: T) -> Arc<[T]>
where
    T: Clone,
{
    let existing_len = bucket.map_or(0, |entries| entries.len());
    let mut next = Vec::with_capacity(existing_len + 1);
    if let Some(entries) = bucket {
        next.extend(entries.iter().cloned());
    }
    next.push(observer);
    Arc::from(next)
}

pub(super) fn remove_observer_from_bucket<T>(
    bucket: &[T],
    id: ObserverId,
    observer_id: impl Fn(&T) -> ObserverId,
) -> Option<Arc<[T]>>
where
    T: Clone,
{
    let mut index = 0_usize;
    while index < bucket.len() {
        if observer_id(&bucket[index]) == id {
            let mut next = Vec::with_capacity(bucket.len().saturating_sub(1));
            next.extend(bucket[..index].iter().cloned());
            next.extend(bucket[index + 1..].iter().cloned());
            return Some(Arc::from(next));
        }
        index += 1;
    }
    None
}
