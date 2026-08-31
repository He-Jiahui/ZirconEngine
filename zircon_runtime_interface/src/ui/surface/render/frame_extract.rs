use std::{
    collections::{BTreeMap, HashMap},
    ops::{Index, Range},
    slice,
    sync::Arc,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::UiLayoutMetrics,
};

use super::{UiPaintElement, UiRenderCommand, UiRenderExtract};

pub const UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE: usize = 64;
const UI_RENDER_FRAME_DIRECTORY_FANOUT: usize = 32;
const UI_RENDER_FRAME_MAX_DIRECTORY_DEPTH: usize = 16;

/// Immutable render data published with a surface frame.
///
/// The mutable surface keeps its flat extract for efficient command generation. Published
/// generations use a persistent directory so a fixed-cardinality local patch copies only the
/// touched command segments and their directory paths.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderFrameExtract {
    pub tree_id: UiTreeId,
    pub list: UiRenderFrameList,
    pub raster_scale: f32,
}

impl Default for UiRenderFrameExtract {
    fn default() -> Self {
        Self {
            tree_id: UiTreeId::default(),
            list: UiRenderFrameList::default(),
            raster_scale: 1.0,
        }
    }
}

impl UiRenderFrameExtract {
    pub fn from_extract(extract: &UiRenderExtract) -> Self {
        Self {
            tree_id: extract.tree_id.clone(),
            list: UiRenderFrameList {
                commands: UiRenderFrameCommands::from_slice(&extract.list.commands),
            },
            raster_scale: extract.raster_scale,
        }
    }

    pub fn patch_ranges_from_extract(
        &self,
        extract: &UiRenderExtract,
        ranges: &[Range<usize>],
    ) -> Option<(Self, UiRenderFramePatchStats)> {
        if self.tree_id != extract.tree_id
            || self.list.commands.len() != extract.list.commands.len()
        {
            return None;
        }
        let (commands, stats) = self
            .list
            .commands
            .patch_ranges(&extract.list.commands, ranges)?;
        Some((
            Self {
                tree_id: extract.tree_id.clone(),
                list: UiRenderFrameList { commands },
                raster_scale: extract.raster_scale,
            },
            stats,
        ))
    }

    pub fn to_extract(&self) -> UiRenderExtract {
        UiRenderExtract {
            tree_id: self.tree_id.clone(),
            list: super::UiRenderList {
                commands: self.list.commands.iter().cloned().collect(),
            },
            raster_scale: self.raster_scale,
        }
    }

    pub fn normalized_raster_scale(&self) -> f32 {
        if self.raster_scale.is_finite() && self.raster_scale > 0.0 {
            self.raster_scale.max(1.0)
        } else {
            1.0
        }
    }

    pub fn command_range(&self, node_id: UiNodeId) -> Option<Range<usize>> {
        self.list.commands.command_range(node_id)
    }

    pub fn commands_for_node(
        &self,
        node_id: UiNodeId,
    ) -> Option<impl ExactSizeIterator<Item = &UiRenderCommand> + '_> {
        self.list.commands.commands_for_node(node_id)
    }

    /// Iterates the immutable command leaves that define renderer cache identity.
    pub fn command_segments(&self) -> impl ExactSizeIterator<Item = &Arc<[UiRenderCommand]>> + '_ {
        self.list.commands.segments()
    }

    pub fn command_by_ref(&self, command_ref: UiRenderFrameCommandRef) -> Option<&UiRenderCommand> {
        let range = self.command_range(command_ref.node_id)?;
        let index = range
            .start
            .checked_add(command_ref.node_command_index as usize)?;
        (index < range.end).then(|| &self.list.commands[index])
    }
}

impl From<UiRenderExtract> for UiRenderFrameExtract {
    fn from(extract: UiRenderExtract) -> Self {
        Self::from_extract(&extract)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderFrameList {
    pub commands: UiRenderFrameCommands,
}

impl UiRenderFrameList {
    pub fn to_paint_elements(&self) -> Vec<UiPaintElement> {
        self.to_paint_elements_with_metrics(UiLayoutMetrics::default())
    }

    pub fn to_paint_elements_with_metrics(&self, metrics: UiLayoutMetrics) -> Vec<UiPaintElement> {
        let mut elements = Vec::new();
        let mut next_paint_order = 0;
        for command in &self.commands {
            let mut command_elements =
                command.to_paint_elements_with_metrics(next_paint_order, metrics);
            next_paint_order += command_elements.len() as u64;
            elements.append(&mut command_elements);
        }
        elements
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiRenderFramePatchStats {
    pub cloned_command_count: usize,
    pub cloned_segment_count: usize,
    pub cloned_directory_node_count: usize,
}

/// Stable command identity within one published render-frame generation.
///
/// Consumers must pair this relative reference with the owning `UiRenderFrameExtract`; it is not a
/// process-global command identifier.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct UiRenderFrameCommandRef {
    pub node_id: UiNodeId,
    pub node_command_index: u32,
}

impl UiRenderFrameCommandRef {
    pub const fn new(node_id: UiNodeId, node_command_index: u32) -> Self {
        Self {
            node_id,
            node_command_index,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiRenderFrameCommands {
    root: Option<Arc<UiRenderFrameCommandNode>>,
    len: usize,
    segment_count: usize,
    directory_depth: u8,
    directory_node_count: usize,
    command_ranges: Arc<HashMap<UiNodeId, Range<usize>>>,
}

impl Default for UiRenderFrameCommands {
    fn default() -> Self {
        Self {
            root: None,
            len: 0,
            segment_count: 0,
            directory_depth: 0,
            directory_node_count: 0,
            command_ranges: Arc::default(),
        }
    }
}

impl UiRenderFrameCommands {
    pub fn from_slice(commands: &[UiRenderCommand]) -> Self {
        if commands.is_empty() {
            return Self::default();
        }

        let mut nodes = commands
            .chunks(UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE)
            .map(|commands| Arc::new(UiRenderFrameCommandNode::Segment(commands.to_vec().into())))
            .collect::<Vec<_>>();
        let segment_count = nodes.len();
        let mut directory_depth = 0_u8;
        let mut directory_node_count = 0_usize;
        loop {
            nodes = nodes
                .chunks(UI_RENDER_FRAME_DIRECTORY_FANOUT)
                .map(|children| {
                    directory_node_count += 1;
                    Arc::new(UiRenderFrameCommandNode::Directory(
                        children.to_vec().into(),
                    ))
                })
                .collect();
            directory_depth = directory_depth.saturating_add(1);
            if nodes.len() == 1 {
                break;
            }
        }

        Self {
            root: nodes.pop(),
            len: commands.len(),
            segment_count,
            directory_depth,
            directory_node_count,
            command_ranges: build_command_ranges(commands),
        }
    }

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

    pub fn get(&self, index: usize) -> Option<&UiRenderCommand> {
        if index >= self.len {
            return None;
        }
        let segment_index = index / UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE;
        let segment_offset = index % UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE;
        let segment =
            segment_for_index(self.root.as_deref()?, self.directory_depth, segment_index)?;
        segment.get(segment_offset)
    }

    pub fn command_range(&self, node_id: UiNodeId) -> Option<Range<usize>> {
        self.command_ranges.get(&node_id).cloned()
    }

    pub fn commands_for_node(
        &self,
        node_id: UiNodeId,
    ) -> Option<impl ExactSizeIterator<Item = &UiRenderCommand> + '_> {
        self.command_range(node_id)
            .map(|range| range.map(|index| &self[index]))
    }

    pub fn first(&self) -> Option<&UiRenderCommand> {
        self.get(0)
    }

    pub fn iter(&self) -> UiRenderFrameCommandsIter<'_> {
        UiRenderFrameCommandsIter::new(self.root.as_deref(), self.len)
    }

    pub fn segments(&self) -> impl ExactSizeIterator<Item = &Arc<[UiRenderCommand]>> + '_ {
        UiRenderFrameCommandSegmentsIter::new(self.root.as_deref(), self.segment_count)
    }

    fn patch_ranges(
        &self,
        source: &[UiRenderCommand],
        ranges: &[Range<usize>],
    ) -> Option<(Self, UiRenderFramePatchStats)> {
        if source.len() != self.len {
            return None;
        }
        if ranges.is_empty() {
            return Some((self.clone(), UiRenderFramePatchStats::default()));
        }

        let mut replacements = BTreeMap::new();
        for range in ranges {
            if range.start > range.end || range.end > self.len {
                return None;
            }
            if range.is_empty() {
                continue;
            }
            let first_segment = range.start / UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE;
            let last_segment = (range.end - 1) / UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE;
            for segment_index in first_segment..=last_segment {
                replacements.entry(segment_index).or_insert_with(|| {
                    let start = segment_index * UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE;
                    let end = (start + UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE).min(source.len());
                    Arc::new(UiRenderFrameCommandNode::Segment(
                        source[start..end].to_vec().into(),
                    ))
                });
            }
        }
        if replacements.is_empty() {
            return Some((self.clone(), UiRenderFramePatchStats::default()));
        }
        if !self.patched_node_identity_is_stable(source, replacements.keys().copied()) {
            return None;
        }

        let mut cloned_directory_node_count = 0;
        let root = patch_directory(
            self.root.as_ref()?,
            self.directory_depth,
            0,
            &replacements,
            &mut cloned_directory_node_count,
        )?;
        let cloned_command_count = replacements
            .values()
            .map(|node| match node.as_ref() {
                UiRenderFrameCommandNode::Segment(commands) => commands.len(),
                UiRenderFrameCommandNode::Directory(_) => 0,
            })
            .sum();
        Some((
            Self {
                root: Some(root),
                len: self.len,
                segment_count: self.segment_count,
                directory_depth: self.directory_depth,
                directory_node_count: self.directory_node_count,
                command_ranges: Arc::clone(&self.command_ranges),
            },
            UiRenderFramePatchStats {
                cloned_command_count,
                cloned_segment_count: replacements.len(),
                cloned_directory_node_count,
            },
        ))
    }

    fn patched_node_identity_is_stable(
        &self,
        source: &[UiRenderCommand],
        mut segment_indices: impl Iterator<Item = usize>,
    ) -> bool {
        segment_indices.all(|segment_index| {
            let start = segment_index * UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE;
            let end = (start + UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE).min(self.len);
            let Some(root) = self.root.as_deref() else {
                return false;
            };
            segment_for_index(root, self.directory_depth, segment_index)
                .filter(|current| current.len() == end - start)
                .is_some_and(|current| {
                    current
                        .iter()
                        .zip(&source[start..end])
                        .all(|(current, next)| current.node_id == next.node_id)
                })
        })
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

#[derive(Clone, Copy)]
struct UiRenderFrameCommandRangeBuildState {
    start: usize,
    end: usize,
    contiguous: bool,
}

fn build_command_ranges(commands: &[UiRenderCommand]) -> Arc<HashMap<UiNodeId, Range<usize>>> {
    let mut states = HashMap::<UiNodeId, UiRenderFrameCommandRangeBuildState>::new();
    for (index, command) in commands.iter().enumerate() {
        if let Some(state) = states.get_mut(&command.node_id) {
            if state.end != index {
                state.contiguous = false;
            }
            state.end = index + 1;
        } else {
            states.insert(
                command.node_id,
                UiRenderFrameCommandRangeBuildState {
                    start: index,
                    end: index + 1,
                    contiguous: true,
                },
            );
        }
    }
    Arc::new(
        states
            .into_iter()
            .filter_map(|(node_id, state)| {
                state
                    .contiguous
                    .then_some((node_id, state.start..state.end))
            })
            .collect(),
    )
}

impl Index<usize> for UiRenderFrameCommands {
    type Output = UiRenderCommand;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .expect("render frame command index must be in bounds")
    }
}

impl Serialize for UiRenderFrameCommands {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.iter())
    }
}

impl<'de> Deserialize<'de> for UiRenderFrameCommands {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<UiRenderCommand>::deserialize(deserializer)
            .map(|commands| Self::from_slice(&commands))
    }
}

impl<'a> IntoIterator for &'a UiRenderFrameCommands {
    type Item = &'a UiRenderCommand;
    type IntoIter = UiRenderFrameCommandsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, PartialEq)]
enum UiRenderFrameCommandNode {
    Segment(Arc<[UiRenderCommand]>),
    Directory(Arc<[Arc<UiRenderFrameCommandNode>]>),
}

pub struct UiRenderFrameCommandsIter<'a> {
    directory_stack:
        [Option<UiRenderFrameDirectoryCursor<'a>>; UI_RENDER_FRAME_MAX_DIRECTORY_DEPTH],
    directory_stack_len: usize,
    segment: Option<slice::Iter<'a, UiRenderCommand>>,
    remaining: usize,
}

#[derive(Clone, Copy)]
struct UiRenderFrameDirectoryCursor<'a> {
    children: &'a [Arc<UiRenderFrameCommandNode>],
    next_child_index: usize,
}

impl<'a> UiRenderFrameCommandsIter<'a> {
    fn new(root: Option<&'a UiRenderFrameCommandNode>, len: usize) -> Self {
        let mut iter = Self {
            directory_stack: [None; UI_RENDER_FRAME_MAX_DIRECTORY_DEPTH],
            directory_stack_len: 0,
            segment: None,
            remaining: len,
        };
        if let Some(root) = root {
            iter.descend(root);
        }
        iter
    }

    fn descend(&mut self, mut node: &'a UiRenderFrameCommandNode) {
        loop {
            match node {
                UiRenderFrameCommandNode::Segment(commands) => {
                    self.segment = Some(commands.iter());
                    return;
                }
                UiRenderFrameCommandNode::Directory(children) => {
                    let Some(first_child) = children.first() else {
                        self.segment = None;
                        return;
                    };
                    assert!(
                        self.directory_stack_len < UI_RENDER_FRAME_MAX_DIRECTORY_DEPTH,
                        "render frame directory depth exceeds the platform bound"
                    );
                    self.directory_stack[self.directory_stack_len] =
                        Some(UiRenderFrameDirectoryCursor {
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

impl<'a> Iterator for UiRenderFrameCommandsIter<'a> {
    type Item = &'a UiRenderCommand;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(command) = self.segment.as_mut().and_then(Iterator::next) {
                self.remaining -= 1;
                return Some(command);
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

impl ExactSizeIterator for UiRenderFrameCommandsIter<'_> {}

struct UiRenderFrameCommandSegmentsIter<'a> {
    directory_stack:
        [Option<UiRenderFrameDirectoryCursor<'a>>; UI_RENDER_FRAME_MAX_DIRECTORY_DEPTH],
    directory_stack_len: usize,
    segment: Option<&'a Arc<[UiRenderCommand]>>,
    remaining: usize,
}

impl<'a> UiRenderFrameCommandSegmentsIter<'a> {
    fn new(root: Option<&'a UiRenderFrameCommandNode>, segment_count: usize) -> Self {
        let mut iter = Self {
            directory_stack: [None; UI_RENDER_FRAME_MAX_DIRECTORY_DEPTH],
            directory_stack_len: 0,
            segment: None,
            remaining: segment_count,
        };
        if let Some(root) = root {
            iter.descend(root);
        }
        iter
    }

    fn descend(&mut self, mut node: &'a UiRenderFrameCommandNode) {
        loop {
            match node {
                UiRenderFrameCommandNode::Segment(commands) => {
                    self.segment = Some(commands);
                    return;
                }
                UiRenderFrameCommandNode::Directory(children) => {
                    let Some(first_child) = children.first() else {
                        self.segment = None;
                        return;
                    };
                    assert!(
                        self.directory_stack_len < UI_RENDER_FRAME_MAX_DIRECTORY_DEPTH,
                        "render frame directory depth exceeds the platform bound"
                    );
                    self.directory_stack[self.directory_stack_len] =
                        Some(UiRenderFrameDirectoryCursor {
                            children,
                            next_child_index: 1,
                        });
                    self.directory_stack_len += 1;
                    node = first_child.as_ref();
                }
            }
        }
    }

    fn advance_segment(&mut self) {
        self.segment = None;
        while self.directory_stack_len > 0 {
            let cursor = self.directory_stack[self.directory_stack_len - 1]
                .as_mut()
                .expect("a retained directory level must own a cursor");
            if let Some(child) = cursor.children.get(cursor.next_child_index) {
                cursor.next_child_index += 1;
                self.descend(child.as_ref());
                return;
            }
            self.directory_stack_len -= 1;
            self.directory_stack[self.directory_stack_len] = None;
        }
    }
}

impl<'a> Iterator for UiRenderFrameCommandSegmentsIter<'a> {
    type Item = &'a Arc<[UiRenderCommand]>;

    fn next(&mut self) -> Option<Self::Item> {
        let segment = self.segment.take()?;
        self.remaining -= 1;
        self.advance_segment();
        Some(segment)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for UiRenderFrameCommandSegmentsIter<'_> {}

fn segment_for_index(
    mut node: &UiRenderFrameCommandNode,
    mut directory_depth: u8,
    mut segment_index: usize,
) -> Option<&[UiRenderCommand]> {
    while directory_depth > 0 {
        let UiRenderFrameCommandNode::Directory(children) = node else {
            return None;
        };
        let child_capacity = directory_child_segment_capacity(directory_depth);
        let child_index = segment_index / child_capacity;
        segment_index %= child_capacity;
        node = children.get(child_index)?.as_ref();
        directory_depth -= 1;
    }
    match node {
        UiRenderFrameCommandNode::Segment(commands) => Some(commands),
        UiRenderFrameCommandNode::Directory(_) => None,
    }
}

fn patch_directory(
    node: &Arc<UiRenderFrameCommandNode>,
    directory_depth: u8,
    first_segment_index: usize,
    replacements: &BTreeMap<usize, Arc<UiRenderFrameCommandNode>>,
    cloned_directory_node_count: &mut usize,
) -> Option<Arc<UiRenderFrameCommandNode>> {
    let UiRenderFrameCommandNode::Directory(children) = node.as_ref() else {
        return None;
    };
    let child_capacity = directory_child_segment_capacity(directory_depth);
    let mut next_children = None;
    for (child_index, child) in children.iter().enumerate() {
        let child_first_segment = first_segment_index + child_index * child_capacity;
        let child_end_segment = child_first_segment + child_capacity;
        if replacements
            .range(child_first_segment..child_end_segment)
            .next()
            .is_none()
        {
            continue;
        }
        let replacement = if directory_depth == 1 {
            Arc::clone(replacements.get(&child_first_segment)?)
        } else {
            patch_directory(
                child,
                directory_depth - 1,
                child_first_segment,
                replacements,
                cloned_directory_node_count,
            )?
        };
        let next_children = next_children.get_or_insert_with(|| children.to_vec());
        next_children[child_index] = replacement;
    }
    let Some(next_children) = next_children else {
        return Some(Arc::clone(node));
    };
    *cloned_directory_node_count += 1;
    Some(Arc::new(UiRenderFrameCommandNode::Directory(
        next_children.into(),
    )))
}

fn directory_child_segment_capacity(directory_depth: u8) -> usize {
    UI_RENDER_FRAME_DIRECTORY_FANOUT.pow(u32::from(directory_depth.saturating_sub(1)))
}

#[cfg(test)]
fn collect_segments<'a>(
    node: Option<&'a UiRenderFrameCommandNode>,
    segments: &mut Vec<&'a Arc<[UiRenderCommand]>>,
) {
    let Some(node) = node else {
        return;
    };
    match node {
        UiRenderFrameCommandNode::Segment(commands) => segments.push(commands),
        UiRenderFrameCommandNode::Directory(children) => {
            for child in children.iter() {
                collect_segments(Some(child.as_ref()), segments);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{
        event_ui::UiNodeId,
        layout::UiFrame,
        surface::{UiRenderCommandKind, UiRenderList, UiResolvedStyle},
    };

    #[test]
    fn local_patch_preserves_untouched_segments_and_flat_order() {
        let extract = extract_with_commands(130);
        let frame = UiRenderFrameExtract::from_extract(&extract);
        let mut changed = extract.clone();
        changed.list.commands[65].opacity = 0.25;

        let (patched, stats) = frame
            .patch_ranges_from_extract(&changed, &[65..66])
            .expect("fixed-cardinality patch should preserve the frame directory");

        assert_eq!(stats.cloned_command_count, 64);
        assert_eq!(stats.cloned_segment_count, 1);
        assert_eq!(stats.cloned_directory_node_count, 1);
        assert_eq!(
            frame
                .list
                .commands
                .shared_segment_count(&patched.list.commands),
            2
        );
        assert_eq!(patched.list.commands[65].opacity, 0.25);
        assert_eq!(patched.to_extract(), changed);
    }

    #[test]
    fn cross_segment_patch_clones_each_touched_leaf_once() {
        let extract = extract_with_commands(130);
        let frame = UiRenderFrameExtract::from_extract(&extract);
        let mut changed = extract.clone();
        changed.list.commands[63].opacity = 0.5;
        changed.list.commands[64].opacity = 0.75;

        let (patched, stats) = frame
            .patch_ranges_from_extract(&changed, &[63..65, 64..65])
            .expect("overlapping ranges should be coalesced by segment identity");

        assert_eq!(stats.cloned_command_count, 128);
        assert_eq!(stats.cloned_segment_count, 2);
        assert_eq!(stats.cloned_directory_node_count, 1);
        assert_eq!(
            frame
                .list
                .commands
                .shared_segment_count(&patched.list.commands),
            1
        );
    }

    #[test]
    fn deep_directory_patch_clones_one_node_per_level() {
        let command_count = UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE * 32 + 1;
        let extract = extract_with_commands(command_count);
        let frame = UiRenderFrameExtract::from_extract(&extract);
        let mut changed = extract.clone();
        changed.list.commands[command_count - 1].opacity = 0.5;

        let (patched, stats) = frame
            .patch_ranges_from_extract(&changed, &[command_count - 1..command_count])
            .expect("the last partial leaf should remain addressable through the directory");

        assert_eq!(frame.list.commands.segment_count(), 33);
        assert_eq!(frame.list.commands.directory_depth(), 2);
        assert_eq!(stats.cloned_command_count, 1);
        assert_eq!(stats.cloned_segment_count, 1);
        assert_eq!(stats.cloned_directory_node_count, 2);
        assert_eq!(
            frame
                .list
                .commands
                .shared_segment_count(&patched.list.commands),
            32
        );
        assert_eq!(patched.to_extract(), changed);
    }

    #[test]
    fn serialized_frame_extract_keeps_the_flat_command_schema() {
        let extract = extract_with_commands(65);
        let frame = UiRenderFrameExtract::from_extract(&extract);
        let flat = serde_json::to_value(&extract).unwrap();
        let segmented = serde_json::to_value(&frame).unwrap();

        assert_eq!(segmented, flat);
        assert_eq!(
            serde_json::from_value::<UiRenderFrameExtract>(segmented).unwrap(),
            frame
        );
    }

    #[test]
    fn sequential_iterator_keeps_exact_flat_order_without_heap_frontier() {
        let extract = extract_with_commands(UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE * 33 + 7);
        let frame = UiRenderFrameExtract::from_extract(&extract);

        assert_eq!(
            frame
                .list
                .commands
                .iter()
                .map(|command| command.node_id)
                .collect::<Vec<_>>(),
            extract
                .list
                .commands
                .iter()
                .map(|command| command.node_id)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn published_node_ranges_are_shared_across_local_payload_patches() {
        let mut extract = extract_with_commands(5);
        for (command, node_id) in extract.list.commands.iter_mut().zip([7_u64, 7, 9, 9, 11]) {
            command.node_id = UiNodeId::new(node_id);
        }
        let frame = UiRenderFrameExtract::from_extract(&extract);

        assert_eq!(frame.command_range(UiNodeId::new(7)), Some(0..2));
        assert_eq!(frame.command_range(UiNodeId::new(9)), Some(2..4));
        assert_eq!(
            frame
                .commands_for_node(UiNodeId::new(9))
                .expect("node commands")
                .map(|command| command.node_id)
                .collect::<Vec<_>>(),
            vec![UiNodeId::new(9), UiNodeId::new(9)]
        );

        let mut changed = extract.clone();
        changed.list.commands[2].opacity = 0.25;
        let (patched, _) = frame
            .patch_ranges_from_extract(&changed, &[2..3])
            .expect("payload-only patch");

        assert!(Arc::ptr_eq(
            &frame.list.commands.command_ranges,
            &patched.list.commands.command_ranges
        ));
    }

    #[test]
    fn frame_command_refs_resolve_only_inside_the_owner_range() {
        let mut extract = extract_with_commands(4);
        for (command, node_id) in extract.list.commands.iter_mut().zip([7_u64, 7, 9, 9]) {
            command.node_id = UiNodeId::new(node_id);
        }
        let frame = UiRenderFrameExtract::from_extract(&extract);

        let second = frame
            .command_by_ref(UiRenderFrameCommandRef::new(UiNodeId::new(7), 1))
            .expect("second command in node range");
        assert_eq!(second.frame.x, 1.0);
        assert!(frame
            .command_by_ref(UiRenderFrameCommandRef::new(UiNodeId::new(7), 2))
            .is_none());
        assert!(frame
            .command_by_ref(UiRenderFrameCommandRef::new(UiNodeId::new(11), 0))
            .is_none());
    }

    #[test]
    fn non_contiguous_owner_ranges_and_owner_changing_patches_fail_closed() {
        let mut extract = extract_with_commands(3);
        for (command, node_id) in extract.list.commands.iter_mut().zip([7_u64, 9, 7]) {
            command.node_id = UiNodeId::new(node_id);
        }
        let frame = UiRenderFrameExtract::from_extract(&extract);
        assert_eq!(frame.command_range(UiNodeId::new(7)), None);

        let mut changed = extract.clone();
        changed.list.commands[1].node_id = UiNodeId::new(11);
        assert!(frame.patch_ranges_from_extract(&changed, &[0..1]).is_none());
    }

    fn extract_with_commands(command_count: usize) -> UiRenderExtract {
        UiRenderExtract {
            tree_id: UiTreeId::new("frame.segmented.render"),
            list: UiRenderList {
                commands: (0..command_count).map(command).collect(),
            },
            raster_scale: 1.0,
        }
    }

    fn command(index: usize) -> UiRenderCommand {
        UiRenderCommand {
            node_id: UiNodeId::new(index as u64 + 1),
            kind: UiRenderCommandKind::Quad,
            frame: UiFrame::new(index as f32, 0.0, 1.0, 1.0),
            clip_frame: None,
            z_index: 0,
            style: UiResolvedStyle::default(),
            text_layout: None,
            text: None,
            image: None,
            opacity: 1.0,
        }
    }
}
