use std::{
    collections::{BTreeMap, HashMap},
    ops::{Index, Range},
    slice,
    sync::Arc,
};

use serde::{Deserialize, Serialize, Serializer};

use crate::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::UiLayoutMetrics,
};

use super::{UiPaintElement, UiRenderCommand, UiRenderExtract};

mod construction;
#[cfg(test)]
#[path = "frame_extract/tests/construction_performance_tests.rs"]
mod construction_performance_tests;
#[cfg(test)]
#[path = "frame_extract/tests/direct_append_tests.rs"]
mod direct_append_tests;
#[cfg(test)]
#[path = "frame_extract/tests/paint_element_capacity_tests.rs"]
mod paint_element_capacity_tests;

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

    /// 按调用方给出的范围替换整段命令叶并复用其余叶；调用方须覆盖全部负载变化。
    /// 这里只校验树、长度、范围及被替换段的 node_id，不扫描声明范围外的命令。
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

    /// 仅当该节点的命令在扁平帧中连续时返回区间；交错节点无法表示成单一范围。
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

    /// 按所属帧内该节点的连续命令区间解析相对索引；缺少区间或越界时返回 None。
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
        let mut elements = Vec::with_capacity(self.commands.len());
        let mut next_paint_order = 0;
        for command in &self.commands {
            let first_element_index = elements.len();
            command.append_paint_elements(next_paint_order, metrics, &mut elements);
            next_paint_order += (elements.len() - first_element_index) as u64;
        }
        elements
    }

    #[cfg(test)]
    fn to_paint_elements_with_metrics_unreserved(
        &self,
        metrics: UiLayoutMetrics,
    ) -> Vec<UiPaintElement> {
        let mut elements = Vec::new();
        let mut next_paint_order = 0;
        for command in &self.commands {
            let first_element_index = elements.len();
            command.append_paint_elements(next_paint_order, metrics, &mut elements);
            next_paint_order += (elements.len() - first_element_index) as u64;
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

    /// 该索引只记录连续命令段，因此交错出现的同节点命令不会产生区间。
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

        // 变更区间提升到整段叶粒度；BTreeMap 合并重叠范围并保证每段只复制一次。
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

    // command_ranges 可与旧帧共享的前提是每个替换叶的 node_id 序列逐项不变。
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

fn build_command_ranges<'a>(
    commands: impl IntoIterator<Item = &'a UiRenderCommand>,
) -> Arc<HashMap<UiNodeId, Range<usize>>> {
    let mut states = HashMap::<UiNodeId, UiRenderFrameCommandRangeBuildState>::new();
    for (index, command) in commands.into_iter().enumerate() {
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

// 分段树只是内部存储；序列化仍输出旧版扁平命令序列以维持既有数据格式。
impl Serialize for UiRenderFrameCommands {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.iter())
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
#[path = "tests/frame_extract.rs"]
mod tests;
