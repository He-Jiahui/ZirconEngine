use std::{ops::Deref, sync::Arc};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    surface::{UiRenderCommand, UiRenderExtract, UiRenderFrameExtract},
};

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
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::layout::UiFrame;
    use zircon_runtime_interface::ui::surface::{
        UiRenderCommandKind, UiRenderList, UiResolvedStyle,
    };

    #[test]
    fn ordered_segments_preserve_extract_allocations() {
        let first = extract("first");
        let second = extract("second");

        let submission =
            UiRenderSubmission::from_frame_segments(vec![Arc::clone(&first), Arc::clone(&second)]);

        assert!(Arc::ptr_eq(submission.segments()[0].extract(), &first));
        assert!(Arc::ptr_eq(submission.segments()[1].extract(), &second));
        assert_eq!(submission.segments()[0].route_tree_id().as_ref(), "first");
        assert_eq!(submission.segments()[1].route_tree_id().as_ref(), "second");
        assert_eq!(submission.command_count(), 0);
        assert!(submission.is_empty());
    }

    #[test]
    fn commands_preserve_segment_and_command_order() {
        let first = extract_with_nodes("first", &[1, 2]);
        let second = extract_with_nodes("second", &[3]);
        let submission = UiRenderSubmission::from_segments(vec![first, second]);

        assert_eq!(
            submission
                .commands()
                .map(|command| command.node_id.0)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(submission.command_count(), 3);
        assert!(!submission.is_empty());
    }

    #[test]
    fn projected_segment_shares_commands_and_projects_only_route_identity() {
        let flat = extract_with_nodes("surface-local", &[7]);
        let frame = Arc::new(UiRenderFrameExtract::from_extract(&flat));
        let segment = UiRenderSubmissionSegment::projected(
            Arc::clone(&frame),
            UiTreeId::new("runtime-project-ui"),
            UiRenderNodeIdProjection::new(2_u64 << 48, (1_u64 << 48) - 1),
        );

        let submission = UiRenderSubmission::from_submission_segments(vec![segment]);

        assert!(Arc::ptr_eq(submission.segments()[0].extract(), &frame));
        assert_eq!(frame.list.commands[0].node_id, UiNodeId::new(7));
        assert_eq!(
            submission.commands().next().unwrap().node_id,
            UiNodeId::new((2_u64 << 48) | 7)
        );
        assert_eq!(
            submission.segments()[0].route_tree_id().as_ref(),
            "runtime-project-ui"
        );
    }

    fn extract(tree_id: &str) -> Arc<UiRenderFrameExtract> {
        Arc::new(UiRenderFrameExtract::from_extract(&UiRenderExtract {
            tree_id: UiTreeId::new(tree_id),
            list: UiRenderList::default(),
            raster_scale: 1.0,
        }))
    }

    fn extract_with_nodes(tree_id: &str, node_ids: &[u64]) -> Arc<UiRenderExtract> {
        Arc::new(UiRenderExtract {
            tree_id: UiTreeId::new(tree_id),
            list: UiRenderList {
                commands: node_ids
                    .iter()
                    .copied()
                    .map(|node_id| UiRenderCommand {
                        node_id: zircon_runtime_interface::ui::event_ui::UiNodeId::new(node_id),
                        kind: UiRenderCommandKind::Quad,
                        frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
                        clip_frame: None,
                        z_index: 0,
                        style: UiResolvedStyle::default(),
                        text_layout: None,
                        text: None,
                        image: None,
                        opacity: 1.0,
                    })
                    .collect(),
            },
            raster_scale: 1.0,
        })
    }
}
