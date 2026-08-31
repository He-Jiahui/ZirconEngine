use std::{
    ops::{Index, IndexMut},
    slice,
    sync::Arc,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE: usize = 64;
const UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT: usize = 32;
const UI_PERSISTENT_SEQUENCE_MAX_DIRECTORY_DEPTH: usize = 16;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiPersistentSequenceCowStats {
    pub cloned_item_count: usize,
    pub cloned_segment_count: usize,
    pub cloned_directory_node_count: usize,
}

impl UiPersistentSequenceCowStats {
    pub fn accumulate(&mut self, other: Self) {
        self.cloned_item_count = self
            .cloned_item_count
            .saturating_add(other.cloned_item_count);
        self.cloned_segment_count = self
            .cloned_segment_count
            .saturating_add(other.cloned_segment_count);
        self.cloned_directory_node_count = self
            .cloned_directory_node_count
            .saturating_add(other.cloned_directory_node_count);
    }
}

/// A persistent, index-addressable sequence for retained UI frame domains.
///
/// Clones share the complete directory. Mutating one item uses copy-on-write for only the
/// containing leaf segment and its directory path, so a consumer can retain an older frame
/// without forcing the producer to clone the full sequence.
#[derive(Clone, Debug)]
pub struct UiPersistentSequence<T> {
    root: Option<Arc<UiPersistentSequenceNode<T>>>,
    len: usize,
    segment_count: usize,
    directory_depth: u8,
    directory_node_count: usize,
}

impl<T> Default for UiPersistentSequence<T> {
    fn default() -> Self {
        Self {
            root: None,
            len: 0,
            segment_count: 0,
            directory_depth: 0,
            directory_node_count: 0,
        }
    }
}

impl<T> UiPersistentSequence<T> {
    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn segment_count(&self) -> usize {
        self.segment_count
    }

    pub const fn directory_depth(&self) -> u8 {
        self.directory_depth
    }

    pub const fn directory_node_count(&self) -> usize {
        self.directory_node_count
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        let segment_index = index / UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE;
        let segment_offset = index % UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE;
        let segment =
            segment_for_index(self.root.as_deref()?, self.directory_depth, segment_index)?;
        segment.get(segment_offset)
    }

    pub fn first(&self) -> Option<&T> {
        self.get(0)
    }

    pub fn iter(&self) -> UiPersistentSequenceIter<'_, T> {
        UiPersistentSequenceIter::new(self.root.as_deref(), self.len)
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

impl<T: Clone> UiPersistentSequence<T> {
    pub fn from_slice(items: &[T]) -> Self {
        Self::from(items.to_vec())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.get_mut_with_stats(index).map(|(item, _stats)| item)
    }

    pub fn get_mut_with_stats(
        &mut self,
        index: usize,
    ) -> Option<(&mut T, UiPersistentSequenceCowStats)> {
        if index >= self.len {
            return None;
        }
        let segment_index = index / UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE;
        let segment_offset = index % UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE;
        let mut stats = UiPersistentSequenceCowStats::default();
        let item = item_for_index_mut(
            self.root.as_mut()?,
            self.directory_depth,
            segment_index,
            segment_offset,
            &mut stats,
        )?;
        Some((item, stats))
    }

    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    #[cfg(test)]
    fn shared_segment_count(&self, other: &Self) -> usize {
        let mut left = Vec::new();
        let mut right = Vec::new();
        collect_segments(self.root.as_deref(), &mut left);
        collect_segments(other.root.as_deref(), &mut right);
        left.iter()
            .zip(right)
            .filter(|(left, right)| Arc::ptr_eq(left, right))
            .count()
    }
}

impl<T> From<Vec<T>> for UiPersistentSequence<T> {
    fn from(items: Vec<T>) -> Self {
        if items.is_empty() {
            return Self::default();
        }

        let len = items.len();
        let mut items = items.into_iter();
        let mut nodes = Vec::with_capacity(len.div_ceil(UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE));
        loop {
            let segment = items
                .by_ref()
                .take(UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE)
                .collect::<Vec<_>>();
            if segment.is_empty() {
                break;
            }
            nodes.push(Arc::new(UiPersistentSequenceNode::Segment(segment.into())));
        }
        let segment_count = nodes.len();
        let mut directory_depth = 0_u8;
        let mut directory_node_count = 0_usize;
        loop {
            nodes = nodes
                .chunks(UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT)
                .map(|children| {
                    directory_node_count += 1;
                    Arc::new(UiPersistentSequenceNode::Directory(
                        children.to_vec().into(),
                    ))
                })
                .collect();
            directory_depth = directory_depth.saturating_add(1);
            assert!(
                usize::from(directory_depth) <= UI_PERSISTENT_SEQUENCE_MAX_DIRECTORY_DEPTH,
                "persistent UI sequence directory depth exceeds the platform bound"
            );
            if nodes.len() == 1 {
                break;
            }
        }

        Self {
            root: nodes.pop(),
            len,
            segment_count,
            directory_depth,
            directory_node_count,
        }
    }
}

impl<T> FromIterator<T> for UiPersistentSequence<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl<T: Clone> Extend<T> for UiPersistentSequence<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut items = self.to_vec();
        items.extend(iter);
        *self = items.into();
    }
}

impl<T: PartialEq> PartialEq for UiPersistentSequence<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}

impl<T: PartialEq> PartialEq<Vec<T>> for UiPersistentSequence<T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.len == other.len() && self.iter().eq(other.iter())
    }
}

impl<T> Index<usize> for UiPersistentSequence<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .expect("persistent UI sequence index must be in bounds")
    }
}

impl<T: Clone> IndexMut<usize> for UiPersistentSequence<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
            .expect("persistent UI sequence index must be in bounds")
    }
}

impl<T: Serialize> Serialize for UiPersistentSequence<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.iter())
    }
}

impl<'de, T> Deserialize<'de> for UiPersistentSequence<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<T>::deserialize(deserializer).map(Into::into)
    }
}

impl<'a, T> IntoIterator for &'a UiPersistentSequence<T> {
    type Item = &'a T;
    type IntoIter = UiPersistentSequenceIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone, Debug)]
enum UiPersistentSequenceNode<T> {
    Segment(Arc<[T]>),
    Directory(Arc<[Arc<UiPersistentSequenceNode<T>>]>),
}

pub struct UiPersistentSequenceIter<'a, T> {
    directory_stack: [Option<UiPersistentSequenceDirectoryCursor<'a, T>>;
        UI_PERSISTENT_SEQUENCE_MAX_DIRECTORY_DEPTH],
    directory_stack_len: usize,
    segment: Option<slice::Iter<'a, T>>,
    remaining: usize,
}

struct UiPersistentSequenceDirectoryCursor<'a, T> {
    children: &'a [Arc<UiPersistentSequenceNode<T>>],
    next_child_index: usize,
}

impl<T> Copy for UiPersistentSequenceDirectoryCursor<'_, T> {}

impl<T> Clone for UiPersistentSequenceDirectoryCursor<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> UiPersistentSequenceIter<'a, T> {
    fn new(root: Option<&'a UiPersistentSequenceNode<T>>, len: usize) -> Self {
        let mut iter = Self {
            directory_stack: [None; UI_PERSISTENT_SEQUENCE_MAX_DIRECTORY_DEPTH],
            directory_stack_len: 0,
            segment: None,
            remaining: len,
        };
        if let Some(root) = root {
            iter.descend(root);
        }
        iter
    }

    fn descend(&mut self, mut node: &'a UiPersistentSequenceNode<T>) {
        loop {
            match node {
                UiPersistentSequenceNode::Segment(items) => {
                    self.segment = Some(items.iter());
                    return;
                }
                UiPersistentSequenceNode::Directory(children) => {
                    let Some(first_child) = children.first() else {
                        self.segment = None;
                        return;
                    };
                    assert!(
                        self.directory_stack_len < UI_PERSISTENT_SEQUENCE_MAX_DIRECTORY_DEPTH,
                        "persistent UI sequence directory depth exceeds the platform bound"
                    );
                    self.directory_stack[self.directory_stack_len] =
                        Some(UiPersistentSequenceDirectoryCursor {
                            children,
                            next_child_index: 1,
                        });
                    self.directory_stack_len += 1;
                    node = first_child.as_ref();
                }
            }
        }
    }

    fn advance_segment(&mut self) -> bool {
        while self.directory_stack_len > 0 {
            let cursor = self.directory_stack[self.directory_stack_len - 1]
                .as_mut()
                .expect("a retained directory level must own a cursor");
            if let Some(child) = cursor.children.get(cursor.next_child_index) {
                cursor.next_child_index += 1;
                self.descend(child.as_ref());
                return self.segment.is_some();
            }
            self.directory_stack_len -= 1;
            self.directory_stack[self.directory_stack_len] = None;
        }
        false
    }
}

impl<'a, T> Iterator for UiPersistentSequenceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item) = self.segment.as_mut().and_then(Iterator::next) {
                self.remaining -= 1;
                return Some(item);
            }
            self.segment = None;
            if !self.advance_segment() {
                return None;
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for UiPersistentSequenceIter<'_, T> {}

fn segment_for_index<T>(
    mut node: &UiPersistentSequenceNode<T>,
    mut directory_depth: u8,
    mut segment_index: usize,
) -> Option<&[T]> {
    while directory_depth > 0 {
        let UiPersistentSequenceNode::Directory(children) = node else {
            return None;
        };
        let child_capacity = directory_child_segment_capacity(directory_depth);
        let child_index = segment_index / child_capacity;
        segment_index %= child_capacity;
        node = children.get(child_index)?.as_ref();
        directory_depth -= 1;
    }
    match node {
        UiPersistentSequenceNode::Segment(items) => Some(items),
        UiPersistentSequenceNode::Directory(_) => None,
    }
}

fn item_for_index_mut<'a, T: Clone>(
    node: &'a mut Arc<UiPersistentSequenceNode<T>>,
    directory_depth: u8,
    segment_index: usize,
    segment_offset: usize,
    stats: &mut UiPersistentSequenceCowStats,
) -> Option<&'a mut T> {
    let node_was_shared = Arc::strong_count(node) > 1;
    let node = Arc::make_mut(node);
    if directory_depth == 0 {
        let UiPersistentSequenceNode::Segment(items) = node else {
            return None;
        };
        if node_was_shared || Arc::strong_count(items) > 1 {
            stats.cloned_item_count = stats.cloned_item_count.saturating_add(items.len());
            stats.cloned_segment_count = stats.cloned_segment_count.saturating_add(1);
        }
        return Arc::make_mut(items).get_mut(segment_offset);
    }

    let UiPersistentSequenceNode::Directory(children) = node else {
        return None;
    };
    if node_was_shared || Arc::strong_count(children) > 1 {
        stats.cloned_directory_node_count = stats.cloned_directory_node_count.saturating_add(1);
    }
    let child_capacity = directory_child_segment_capacity(directory_depth);
    let child_index = segment_index / child_capacity;
    let child_segment_index = segment_index % child_capacity;
    item_for_index_mut(
        Arc::make_mut(children).get_mut(child_index)?,
        directory_depth - 1,
        child_segment_index,
        segment_offset,
        stats,
    )
}

fn directory_child_segment_capacity(directory_depth: u8) -> usize {
    UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT.pow(u32::from(directory_depth.saturating_sub(1)))
}

#[cfg(test)]
fn collect_segments<'a, T>(
    node: Option<&'a UiPersistentSequenceNode<T>>,
    segments: &mut Vec<&'a Arc<[T]>>,
) {
    let Some(node) = node else {
        return;
    };
    match node {
        UiPersistentSequenceNode::Segment(items) => segments.push(items),
        UiPersistentSequenceNode::Directory(children) => {
            for child in children.iter() {
                collect_segments(Some(child.as_ref()), segments);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NonClone(u32);

    #[test]
    fn owned_vector_construction_does_not_require_item_clones() {
        let sequence: UiPersistentSequence<_> = vec![NonClone(1), NonClone(2)].into();

        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence[1].0, 2);
    }

    #[test]
    fn one_item_mutation_clones_only_its_leaf_path() {
        let retained: UiPersistentSequence<_> = (0_u32..130).collect();
        let mut next = retained.clone();

        let (_, stats) = next
            .get_mut_with_stats(65)
            .map(|(item, stats)| {
                *item = 9_999;
                (item, stats)
            })
            .expect("mutable item");

        assert_eq!(retained[65], 65);
        assert_eq!(next[65], 9_999);
        assert_eq!(retained.shared_segment_count(&next), 2);
        assert_eq!(next.iter().copied().collect::<Vec<_>>().len(), 130);
        assert_eq!(stats.cloned_item_count, 64);
        assert_eq!(stats.cloned_segment_count, 1);
        assert_eq!(stats.cloned_directory_node_count, 1);
    }

    #[test]
    fn serde_keeps_the_flat_wire_contract() {
        let sequence: UiPersistentSequence<_> = (0_u32..70).collect();
        let encoded = serde_json::to_string(&sequence).expect("serialize persistent sequence");
        let decoded: UiPersistentSequence<u32> =
            serde_json::from_str(&encoded).expect("deserialize persistent sequence");

        assert_eq!(decoded, sequence);
        assert_eq!(
            encoded,
            serde_json::to_string(&(0_u32..70).collect::<Vec<_>>()).unwrap()
        );
    }

    #[test]
    fn unique_owner_mutation_uses_the_allocation_free_path() {
        let mut sequence: UiPersistentSequence<_> = (0_u32..130).collect();

        let (item, stats) = sequence
            .get_mut_with_stats(65)
            .expect("mutable unique-owner item");
        *item = 7_777;

        assert_eq!(sequence[65], 7_777);
        assert_eq!(stats, UiPersistentSequenceCowStats::default());
    }
}
