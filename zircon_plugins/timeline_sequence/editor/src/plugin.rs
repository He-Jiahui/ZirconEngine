use zircon_editor::core::asset::{AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId};
use zircon_editor::core::commands::{EditorCommandDescriptor, EditorCommandMenuPath};
use zircon_editor::core::editor_authoring_extension::{
    TimelineEditorDescriptor, TimelineTrackDescriptor,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::{
    core::framework::project::ExportPackagingStrategy,
    core::framework::project::ExportTargetPlatform, plugin::PluginDistributionManifest,
    plugin::PluginModuleManifest, plugin::PluginPackageManifest,
};
use zircon_runtime_interface::resource::ResourceKind;

use crate::{
    ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY, CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID,
    TIMELINE_SEQUENCE_DRAWER_ID, TIMELINE_SEQUENCE_TEMPLATE_ID, TIMELINE_SEQUENCE_VIEW_ID,
};

pub const TIMELINE_SEQUENCE_DIST_CRATE_NAME: &str = "zircon_plugin_timeline_sequence_dist";
pub const TIMELINE_SEQUENCE_DIST_EDITOR_ENTRY: &str =
    "zircon_plugin_timeline_sequence_editor_entry_v3";
const TIMELINE_SEQUENCE_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct TimelineSequenceEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl TimelineSequenceEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for TimelineSequenceEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: TIMELINE_SEQUENCE_DRAWER_ID,
                drawer_display_name: "Timeline Sequence",
                template_id: TIMELINE_SEQUENCE_TEMPLATE_ID,
                template_document: "plugins://timeline_sequence/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    TIMELINE_SEQUENCE_VIEW_ID,
                    "Timeline Sequence",
                    "Animation",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, timeline_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Timeline Sequence",
        "zircon_plugin_timeline_sequence_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> TimelineSequenceEditorPlugin {
    TimelineSequenceEditorPlugin::new()
}

pub fn package_manifest() -> PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(&editor_plugin(), base_package_manifest())
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        base_package_manifest(),
    )
}

fn base_package_manifest() -> PluginPackageManifest {
    PluginPackageManifest::new(PLUGIN_ID, "Timeline Sequence")
        .with_category("authoring")
        .with_supported_targets([RuntimeTargetMode::EditorHost])
        .with_supported_platforms([
            ExportTargetPlatform::Windows,
            ExportTargetPlatform::Linux,
            ExportTargetPlatform::Macos,
        ])
        .with_capabilities(EDITOR_CAPABILITIES.iter().copied())
        .with_default_packaging([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
            ExportPackagingStrategy::NativeDynamic,
        ])
        .with_dependency(
            zircon_runtime::plugin::PluginDependencyManifest::new("animation", true)
                .with_capability(ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY),
        )
        .with_native_module(timeline_sequence_dist_module_manifest())
        .with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: TIMELINE_SEQUENCE_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: TIMELINE_SEQUENCE_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            editor_entry: TIMELINE_SEQUENCE_DIST_EDITOR_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
}

pub fn timeline_sequence_dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native("timeline_sequence.dist", TIMELINE_SEQUENCE_DIST_CRATE_NAME)
        .with_target_modes([RuntimeTargetMode::EditorHost])
        .with_capabilities([CAPABILITY])
}

fn timeline_authoring_batch() -> EditorAuthoringContributionBatch {
    let open = operation("timeline_sequence.authoring.open");
    let create_track = operation("timeline_sequence.track.create");
    let delete_track = operation("timeline_sequence.track.delete");
    let move_key = operation("timeline_sequence.keyframe.move");
    let validate = operation("timeline_sequence.authoring.validate");
    EditorAuthoringContributionBatch {
        commands: vec![
            EditorCommandDescriptor::operation(open.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &open,
                    "plugins",
                    &["timeline_sequence"],
                ))
                .with_payload_schema_id("timeline_sequence.open.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(create_track.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &create_track,
                    "plugins",
                    &["timeline_sequence"],
                ))
                .with_payload_schema_id("timeline_sequence.create_track.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(delete_track.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &delete_track,
                    "plugins",
                    &["timeline_sequence"],
                ))
                .with_payload_schema_id("timeline_sequence.delete_track.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(move_key.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &move_key,
                    "plugins",
                    &["timeline_sequence"],
                ))
                .with_payload_schema_id("timeline_sequence.move_keyframe.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(validate.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &validate,
                    "plugins",
                    &["timeline_sequence"],
                ))
                .with_payload_schema_id("timeline_sequence.validate.v1")
                .with_required_capabilities([CAPABILITY]),
        ],
        menu_items: Vec::new(),
        asset_type_contributions: vec![AssetTypeContribution::augment(
            AssetTypeId::from_resource_kind(ResourceKind::AnimationSequence),
        )
        .with_toolkit(
            AssetToolkitDescriptor::new(TIMELINE_SEQUENCE_VIEW_ID, open.clone())
                .with_required_capabilities([CAPABILITY]),
        )],
        timeline_track_types: vec![
            TimelineTrackDescriptor::new(
                "timeline_sequence.track.transform",
                "Transform",
                "transform",
            )
            .with_required_capabilities([CAPABILITY]),
            TimelineTrackDescriptor::new(
                "timeline_sequence.track.component_property",
                "Component Property",
                "component_property",
            )
            .with_required_capabilities([CAPABILITY]),
            TimelineTrackDescriptor::new(
                "timeline_sequence.track.event_marker",
                "Event Marker",
                "event_marker",
            )
            .with_required_capabilities([CAPABILITY]),
        ],
        timeline_editors: vec![TimelineEditorDescriptor::new(
            AssetTypeId::from_resource_kind(ResourceKind::AnimationSequence),
            TIMELINE_SEQUENCE_VIEW_ID,
            "Timeline Sequence",
            open,
        )
        .with_track_type("timeline_sequence.track.transform")
        .with_track_type("timeline_sequence.track.component_property")
        .with_track_type("timeline_sequence.track.event_marker")
        .with_required_capabilities([CAPABILITY])],
        ..Default::default()
    }
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid timeline operation path")
}
