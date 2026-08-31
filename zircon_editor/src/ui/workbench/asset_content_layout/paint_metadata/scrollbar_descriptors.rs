use super::{AssetContentGeometry, AssetContentPaintMetadata, AssetContentRect};
use crate::ui::workbench::asset_content_layout::AssetContentSurface;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AssetContentScrollbarKind {
    Tree,
    Content,
    References,
    UsedBy,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AssetContentScrollbarViewport {
    ActivityTree,
    Local(AssetContentRect),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AssetContentScrollbarExtent {
    Pixels(f32),
    TreeRows(usize),
    ReferenceRows(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AssetContentScrollbarDescriptor {
    kind: AssetContentScrollbarKind,
}

const EMPTY_DESCRIPTOR: AssetContentScrollbarDescriptor = AssetContentScrollbarDescriptor {
    kind: AssetContentScrollbarKind::Content,
};

#[derive(Clone, Debug)]
pub(super) struct AssetContentScrollbarDescriptors {
    entries: [AssetContentScrollbarDescriptor; 4],
    len: usize,
}

impl AssetContentScrollbarDescriptors {
    fn new() -> Self {
        Self {
            entries: [EMPTY_DESCRIPTOR; 4],
            len: 0,
        }
    }

    fn push(&mut self, descriptor: AssetContentScrollbarDescriptor) {
        debug_assert!(self.len < self.entries.len());
        self.entries[self.len] = descriptor;
        self.len += 1;
    }

    pub(super) fn as_slice(&self) -> &[AssetContentScrollbarDescriptor] {
        &self.entries[..self.len]
    }
}

impl AssetContentScrollbarDescriptor {
    pub(crate) fn kind(self) -> AssetContentScrollbarKind {
        self.kind
    }
}

pub(super) fn build_scrollbar_descriptors(
    surface: AssetContentSurface,
    geometry: &AssetContentGeometry,
) -> AssetContentScrollbarDescriptors {
    let mut descriptors = AssetContentScrollbarDescriptors::new();
    match surface {
        AssetContentSurface::Activity => {
            descriptors.push(AssetContentScrollbarDescriptor {
                kind: AssetContentScrollbarKind::Tree,
            });
            push_local(
                &mut descriptors,
                AssetContentScrollbarKind::Content,
                geometry.viewport,
            );
            push_local(
                &mut descriptors,
                AssetContentScrollbarKind::References,
                geometry.activity_references_viewport,
            );
            push_local(
                &mut descriptors,
                AssetContentScrollbarKind::UsedBy,
                geometry.activity_used_by_viewport,
            );
        }
        AssetContentSurface::Browser => {
            push_local(
                &mut descriptors,
                AssetContentScrollbarKind::Tree,
                geometry.browser_source_tree_viewport,
            );
            push_local(
                &mut descriptors,
                AssetContentScrollbarKind::Content,
                geometry.viewport,
            );
            push_local(
                &mut descriptors,
                AssetContentScrollbarKind::References,
                geometry.browser_references_viewport,
            );
            push_local(
                &mut descriptors,
                AssetContentScrollbarKind::UsedBy,
                geometry.browser_used_by_viewport,
            );
        }
    }
    descriptors
}

fn push_local(
    descriptors: &mut AssetContentScrollbarDescriptors,
    kind: AssetContentScrollbarKind,
    viewport: Option<AssetContentRect>,
) {
    let Some(_) = viewport else {
        return;
    };
    descriptors.push(AssetContentScrollbarDescriptor { kind });
}

impl AssetContentPaintMetadata {
    pub(crate) fn scrollbar_viewport(
        &self,
        descriptor: AssetContentScrollbarDescriptor,
    ) -> Option<AssetContentScrollbarViewport> {
        match (self.surface, descriptor.kind) {
            (AssetContentSurface::Activity, AssetContentScrollbarKind::Tree) => {
                Some(AssetContentScrollbarViewport::ActivityTree)
            }
            (_, AssetContentScrollbarKind::Content) => {
                self.viewport.map(AssetContentScrollbarViewport::Local)
            }
            (AssetContentSurface::Activity, AssetContentScrollbarKind::References) => self
                .activity_references_viewport
                .map(AssetContentScrollbarViewport::Local),
            (AssetContentSurface::Activity, AssetContentScrollbarKind::UsedBy) => self
                .activity_used_by_viewport
                .map(AssetContentScrollbarViewport::Local),
            (AssetContentSurface::Browser, AssetContentScrollbarKind::Tree) => self
                .browser_source_tree_viewport
                .map(AssetContentScrollbarViewport::Local),
            (AssetContentSurface::Browser, AssetContentScrollbarKind::References) => self
                .browser_references_viewport
                .map(AssetContentScrollbarViewport::Local),
            (AssetContentSurface::Browser, AssetContentScrollbarKind::UsedBy) => self
                .browser_used_by_viewport
                .map(AssetContentScrollbarViewport::Local),
        }
    }

    pub(crate) fn scrollbar_extent(
        &self,
        descriptor: AssetContentScrollbarDescriptor,
    ) -> AssetContentScrollbarExtent {
        match (self.surface, descriptor.kind) {
            (AssetContentSurface::Activity, AssetContentScrollbarKind::Tree) => {
                AssetContentScrollbarExtent::TreeRows(self.activity_tree_rows.len())
            }
            (AssetContentSurface::Browser, AssetContentScrollbarKind::Tree) => {
                AssetContentScrollbarExtent::TreeRows(self.browser_source_tree_groups.len())
            }
            (_, AssetContentScrollbarKind::Content) => {
                AssetContentScrollbarExtent::Pixels(self.content_extent)
            }
            (AssetContentSurface::Activity, AssetContentScrollbarKind::References) => {
                AssetContentScrollbarExtent::ReferenceRows(self.activity_references_groups.len())
            }
            (AssetContentSurface::Activity, AssetContentScrollbarKind::UsedBy) => {
                AssetContentScrollbarExtent::ReferenceRows(self.activity_used_by_groups.len())
            }
            (AssetContentSurface::Browser, AssetContentScrollbarKind::References) => {
                AssetContentScrollbarExtent::ReferenceRows(self.browser_references_groups.len())
            }
            (AssetContentSurface::Browser, AssetContentScrollbarKind::UsedBy) => {
                AssetContentScrollbarExtent::ReferenceRows(self.browser_used_by_groups.len())
            }
        }
    }
}
