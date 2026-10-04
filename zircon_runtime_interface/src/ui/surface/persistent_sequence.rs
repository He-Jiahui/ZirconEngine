use std::{
    fmt,
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice,
    sync::Arc,
};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

pub const UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE: usize = 64;
const UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT: usize = 32;
const UI_PERSISTENT_SEQUENCE_MAX_DIRECTORY_DEPTH: usize = 16;

#[cfg(test)]
#[path = "persistent_sequence/tests/construction_performance_tests.rs"]
mod construction_performance_tests;
#[cfg(test)]
#[path = "persistent_sequence/tests/deserialization_performance_tests.rs"]
mod deserialization_performance_tests;
#[cfg(test)]
#[path = "persistent_sequence/tests/directory_performance_tests.rs"]
mod directory_performance_tests;
#[cfg(test)]
#[path = "persistent_sequence/tests/extend_performance_tests.rs"]
mod extend_performance_tests;
#[cfg(test)]
#[path = "persistent_sequence/tests/tail_segment_performance_tests.rs"]
mod tail_segment_performance_tests;

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

    fn from_owned_iter<I: Iterator<Item = T>>(mut items: I) -> Self {
        let (lower_bound, _) = items.size_hint();
        let mut nodes =
            Vec::with_capacity(lower_bound.div_ceil(UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE));
        let mut len = 0usize;

        while let Some(first) = items.next() {
            let mut segment = Vec::with_capacity(Self::owned_segment_capacity(&items));
            segment.push(first);
            segment.extend(items.by_ref().take(UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE - 1));
            len += segment.len();
            nodes.push(Arc::new(UiPersistentSequenceNode::Segment(segment.into())));
        }

        Self::from_segment_nodes(nodes, len)
    }

    fn owned_segment_capacity<I: Iterator<Item = T>>(items: &I) -> usize {
        let (lower_bound, upper_bound) = items.size_hint();
        if upper_bound == Some(lower_bound) {
            1 + lower_bound.min(UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE - 1)
        } else {
            UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE
        }
    }

    fn from_segment_nodes(mut nodes: Vec<Arc<UiPersistentSequenceNode<T>>>, len: usize) -> Self {
        if nodes.is_empty() {
            return Self::default();
        }

        let segment_count = nodes.len();
        let mut directory_depth = 0_u8;
        let mut directory_node_count = 0_usize;
        loop {
            nodes = Self::promote_directory_level(nodes, &mut directory_node_count);
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

    fn promote_directory_level(
        nodes: Vec<Arc<UiPersistentSequenceNode<T>>>,
        directory_node_count: &mut usize,
    ) -> Vec<Arc<UiPersistentSequenceNode<T>>> {
        let parent_count = nodes
            .len()
            .div_ceil(UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT);
        let mut children = nodes.into_iter();
        let mut parents = Vec::with_capacity(parent_count);

        while let Some(first) = children.next() {
            let child_capacity = 1 + children
                .len()
                .min(UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT - 1);
            let mut directory = Vec::with_capacity(child_capacity);
            directory.push(first);
            directory.extend(
                children
                    .by_ref()
                    .take(UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT - 1),
            );
            *directory_node_count += 1;
            parents.push(Arc::new(UiPersistentSequenceNode::Directory(
                directory.into(),
            )));
        }

        parents
    }
}

impl<T: Clone> UiPersistentSequence<T> {
    pub fn from_slice(items: &[T]) -> Self {
        Self::from_owned_iter(items.iter().cloned())
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
        Self::from_owned_iter(items.into_iter())
    }
}

impl<T> FromIterator<T> for UiPersistentSequence<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_owned_iter(iter.into_iter())
    }
}

impl<T: Clone> Extend<T> for UiPersistentSequence<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let next = Self::from_owned_iter(self.iter().cloned().chain(iter));
        *self = next;
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

struct UiPersistentSequenceVisitor<T>(PhantomData<fn() -> T>);

impl<'de, T> Visitor<'de> for UiPersistentSequenceVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = UiPersistentSequence<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a flat persistent UI sequence")
    }

    // 保持扁平序列的 serde 形状，同时边读取边组装固定大小叶段，避免先暂存整条 Vec<T>。
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let item_count = sequence.size_hint().unwrap_or(0);
        let mut nodes =
            Vec::with_capacity(item_count.div_ceil(UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE));
        let mut len = 0usize;

        loop {
            let Some(first) = sequence.next_element::<T>()? else {
                break;
            };
            let mut segment = Vec::with_capacity(UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE);
            segment.push(first);
            let mut reached_end = false;
            while segment.len() < UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE {
                let Some(item) = sequence.next_element::<T>()? else {
                    reached_end = true;
                    break;
                };
                segment.push(item);
            }
            len += segment.len();
            nodes.push(Arc::new(UiPersistentSequenceNode::Segment(segment.into())));
            if reached_end {
                break;
            }
        }

        Ok(UiPersistentSequence::from_segment_nodes(nodes, len))
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
        deserializer.deserialize_seq(UiPersistentSequenceVisitor::<T>(PhantomData))
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
#[path = "tests/persistent_sequence.rs"]
mod tests;
