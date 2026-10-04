use std::{ops::Deref, sync::Arc};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    surface::{UiRenderCommand, UiRenderExtract, UiRenderFrameExtract},
};

/// 把各运行时 UI 表面的本地节点 ID 投影到共享路由树命名空间。
/// 调用方必须为前缀和本地掩码分配不重叠的位域，并保证本地 ID 不超出掩码。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiRenderNodeIdProjection {
    prefix: u64,
    local_mask: u64,
}

impl UiRenderNodeIdProjection {
    pub const fn new(prefix: u64, local_mask: u64) -> Self {
        Self { prefix, local_mask }
    }

    pub const fn project(self, node_id: UiNodeId) -> UiNodeId {
        UiNodeId::new(self.prefix | (node_id.0 & self.local_mask))
    }
}

/// A retained command domain plus the route identity projected by its submission owner.
#[derive(Clone, Debug)]
pub struct UiRenderSubmissionSegment {
    extract: Arc<UiRenderFrameExtract>,
    route_tree_id: Arc<str>,
    node_id_projection: Option<UiRenderNodeIdProjection>,
}

impl UiRenderSubmissionSegment {
    pub fn identity(extract: Arc<UiRenderFrameExtract>) -> Self {
        let route_tree_id = Arc::<str>::from(extract.tree_id.0.as_str());
        Self {
            extract,
            route_tree_id,
            node_id_projection: None,
        }
    }

    pub fn projected(
        extract: Arc<UiRenderFrameExtract>,
        route_tree_id: UiTreeId,
        node_id_projection: UiRenderNodeIdProjection,
    ) -> Self {
        Self {
            extract,
            route_tree_id: Arc::from(route_tree_id.0),
            node_id_projection: Some(node_id_projection),
        }
    }

    pub fn extract(&self) -> &Arc<UiRenderFrameExtract> {
        &self.extract
    }

    pub fn route_tree_id(&self) -> &Arc<str> {
        &self.route_tree_id
    }

    pub fn node_id_projection(&self) -> Option<UiRenderNodeIdProjection> {
        self.node_id_projection
    }

    pub fn project_node_id(&self, node_id: UiNodeId) -> UiNodeId {
        self.node_id_projection
            .map_or(node_id, |projection| projection.project(node_id))
    }

    pub fn command_count(&self) -> usize {
        self.extract.list.commands.len()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UiRenderSubmissionCommand<'a> {
    command: &'a UiRenderCommand,
    pub node_id: UiNodeId,
}

impl Deref for UiRenderSubmissionCommand<'_> {
    type Target = UiRenderCommand;

    fn deref(&self) -> &Self::Target {
        self.command
    }
}

/// Ordered, immutable UI command segments retained through renderer submission.
#[derive(Clone, Debug)]
pub struct UiRenderSubmission {
    segments: Arc<[UiRenderSubmissionSegment]>,
    command_count: usize,
}

impl UiRenderSubmission {
    pub fn single(extract: Arc<UiRenderExtract>) -> Arc<Self> {
        Self::from_segments(vec![extract])
    }

    pub fn from_segments(segments: Vec<Arc<UiRenderExtract>>) -> Arc<Self> {
        Self::from_frame_segments(
            segments
                .into_iter()
                .map(|extract| Arc::new(UiRenderFrameExtract::from_extract(&extract)))
                .collect(),
        )
    }

    pub fn single_frame(extract: Arc<UiRenderFrameExtract>) -> Arc<Self> {
        Self::from_frame_segments(vec![extract])
    }

    pub fn from_frame_segments(segments: Vec<Arc<UiRenderFrameExtract>>) -> Arc<Self> {
        Self::from_submission_segments(
            segments
                .into_iter()
                .map(UiRenderSubmissionSegment::identity)
                .collect(),
        )
    }

    pub fn from_submission_segments(segments: Vec<UiRenderSubmissionSegment>) -> Arc<Self> {
        let command_count = segments.iter().fold(0_usize, |count, segment| {
            count.saturating_add(segment.command_count())
        });
        Arc::new(Self {
            segments: Arc::from(segments),
            command_count,
        })
    }

    pub fn segments(&self) -> &[UiRenderSubmissionSegment] {
        &self.segments
    }

    /// 按段保留绘制顺序，迭代时才投影路由 ID；原始 Arc 提取命令保持可复用。
    pub fn commands(&self) -> impl Iterator<Item = UiRenderSubmissionCommand<'_>> {
        self.segments.iter().flat_map(|segment| {
            segment
                .extract
                .list
                .commands
                .iter()
                .map(|command| UiRenderSubmissionCommand {
                    command,
                    node_id: segment.project_node_id(command.node_id),
                })
        })
    }

    pub fn command_count(&self) -> usize {
        self.command_count
    }

    pub fn is_empty(&self) -> bool {
        self.command_count == 0
    }
}

#[cfg(test)]
#[path = "tests/ui_submission.rs"]
mod tests;
