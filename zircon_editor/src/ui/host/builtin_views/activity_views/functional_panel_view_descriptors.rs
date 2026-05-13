use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    DockPolicy, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn functional_panel_view_descriptors() -> Vec<ViewDescriptor> {
    vec![
        document_view(
            "editor.prefab.viewport",
            "Prefab Viewport",
            "prefab-viewport",
            ViewContentKind::PrefabEditor,
        ),
        document_view(
            "editor.prefab.inspector",
            "Prefab Inspector",
            "prefab-inspector",
            ViewContentKind::Inspector,
        ),
        document_view(
            "editor.material.graph",
            "Material Graph",
            "material-graph",
            ViewContentKind::AssetBrowser,
        ),
        document_view(
            "editor.material.preview",
            "Material Preview",
            "material-preview",
            ViewContentKind::AssetBrowser,
        ),
        document_view(
            "editor.ui.designer",
            "UI Designer",
            "ui-designer",
            ViewContentKind::UiAssetEditor,
        ),
        document_view(
            "editor.ui.source",
            "UI Source",
            "ui-source",
            ViewContentKind::UiAssetEditor,
        ),
        document_view(
            "editor.animation.timeline",
            "Animation Timeline",
            "animation-timeline",
            ViewContentKind::AnimationSequenceEditor,
        ),
        document_view(
            "editor.animation.graph",
            "Animation Graph",
            "animation-graph",
            ViewContentKind::AnimationGraphEditor,
        ),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.asset_preview"),
            ViewKind::ActivityView,
            "Asset Preview",
        )
        .with_preferred_host(PreferredHost::Drawer(
            crate::ui::workbench::layout::ActivityDrawerSlot::RightTop,
        ))
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::AssetBrowser,
        ))
        .with_icon_key("asset-preview"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.asset_metadata"),
            ViewKind::ActivityView,
            "Asset Metadata",
        )
        .with_preferred_host(PreferredHost::Drawer(
            crate::ui::workbench::layout::ActivityDrawerSlot::RightTop,
        ))
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Inspector))
        .with_icon_key("asset-metadata"),
    ]
}

fn document_view(
    descriptor_id: &'static str,
    title: &'static str,
    icon_key: &'static str,
    content_kind: ViewContentKind,
) -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new(descriptor_id),
        ViewKind::ActivityView,
        title,
    )
    .with_dock_policy(DockPolicy::DocumentOnly)
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(content_kind))
    .with_icon_key(icon_key)
}
