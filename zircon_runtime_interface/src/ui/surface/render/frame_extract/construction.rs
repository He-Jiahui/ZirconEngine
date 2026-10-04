use std::{fmt, marker::PhantomData, sync::Arc};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use super::{
    build_command_ranges, UiRenderCommand, UiRenderFrameCommandNode, UiRenderFrameCommands,
    UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE, UI_RENDER_FRAME_DIRECTORY_FANOUT,
};

impl UiRenderFrameCommands {
    pub fn from_slice(commands: &[UiRenderCommand]) -> Self {
        Self::from_owned_iter(commands.iter().cloned())
    }

    fn from_owned_iter<I>(mut commands: I) -> Self
    where
        I: Iterator<Item = UiRenderCommand>,
    {
        let (lower_bound, _) = commands.size_hint();
        let mut nodes =
            Vec::with_capacity(lower_bound.div_ceil(UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE));
        let mut len = 0usize;

        while let Some(first) = commands.next() {
            let mut segment = Vec::with_capacity(Self::owned_segment_capacity(&commands));
            segment.push(first);
            segment.extend(
                commands
                    .by_ref()
                    .take(UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE - 1),
            );
            len += segment.len();
            nodes.push(Arc::new(UiRenderFrameCommandNode::Segment(segment.into())));
        }

        Self::from_segment_nodes(nodes, len)
    }

    fn owned_segment_capacity<I>(commands: &I) -> usize
    where
        I: Iterator<Item = UiRenderCommand>,
    {
        let (lower_bound, upper_bound) = commands.size_hint();
        if upper_bound == Some(lower_bound) {
            1 + lower_bound.min(UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE - 1)
        } else {
            UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE
        }
    }

    fn from_segment_nodes(mut nodes: Vec<Arc<UiRenderFrameCommandNode>>, len: usize) -> Self {
        if nodes.is_empty() {
            return Self::default();
        }

        let segment_count = nodes.len();
        let mut directory_depth = 0_u8;
        let mut directory_node_count = 0_usize;
        loop {
            nodes = Self::promote_directory_level(nodes, &mut directory_node_count);
            directory_depth = directory_depth.saturating_add(1);
            if nodes.len() == 1 {
                break;
            }
        }

        let mut commands = Self {
            root: nodes.pop(),
            len,
            segment_count,
            directory_depth,
            directory_node_count,
            command_ranges: Arc::default(),
        };
        commands.command_ranges = build_command_ranges(commands.iter());
        commands
    }

    pub(super) fn promote_directory_level(
        nodes: Vec<Arc<UiRenderFrameCommandNode>>,
        directory_node_count: &mut usize,
    ) -> Vec<Arc<UiRenderFrameCommandNode>> {
        let parent_count = nodes.len().div_ceil(UI_RENDER_FRAME_DIRECTORY_FANOUT);
        let mut children = nodes.into_iter();
        let mut parents = Vec::with_capacity(parent_count);

        while let Some(first) = children.next() {
            let child_capacity = 1 + children.len().min(UI_RENDER_FRAME_DIRECTORY_FANOUT - 1);
            let mut directory = Vec::with_capacity(child_capacity);
            directory.push(first);
            directory.extend(children.by_ref().take(UI_RENDER_FRAME_DIRECTORY_FANOUT - 1));
            *directory_node_count += 1;
            parents.push(Arc::new(UiRenderFrameCommandNode::Directory(
                directory.into(),
            )));
        }

        parents
    }
}

struct UiRenderFrameCommandsVisitor(PhantomData<fn() -> UiRenderCommand>);

impl<'de> Visitor<'de> for UiRenderFrameCommandsVisitor {
    type Value = UiRenderFrameCommands;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a flat render-frame command sequence")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // 输入仍是旧的扁平 serde 序列；分段消费而不先收集成第二份完整命令向量。
        let command_count = sequence.size_hint().unwrap_or(0);
        let mut nodes =
            Vec::with_capacity(command_count.div_ceil(UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE));
        let mut len = 0usize;

        // 每收满一段立即封存为共享叶，末段可较短，保留迭代顺序。
        loop {
            let Some(first) = sequence.next_element::<UiRenderCommand>()? else {
                break;
            };
            let segment_capacity = sequence
                .size_hint()
                .map_or(UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE, |remaining| {
                    1 + remaining.min(UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE - 1)
                });
            let mut segment = Vec::with_capacity(segment_capacity);
            segment.push(first);
            let mut reached_end = false;
            while segment.len() < UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE {
                let Some(command) = sequence.next_element::<UiRenderCommand>()? else {
                    reached_end = true;
                    break;
                };
                segment.push(command);
            }
            len += segment.len();
            nodes.push(Arc::new(UiRenderFrameCommandNode::Segment(segment.into())));
            if reached_end {
                break;
            }
        }

        Ok(UiRenderFrameCommands::from_segment_nodes(nodes, len))
    }
}

impl<'de> Deserialize<'de> for UiRenderFrameCommands {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(UiRenderFrameCommandsVisitor(PhantomData))
    }
}
