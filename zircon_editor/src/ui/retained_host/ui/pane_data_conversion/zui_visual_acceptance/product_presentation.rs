use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use zircon_runtime::core::CoreHandle;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::retained_host::callback_dispatch;
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::primitives::PhysicalSize;
use crate::ui::retained_host::ui::apply_presentation_with_template_v2_data;
use crate::ui::retained_host::{HostWindowPresentationData, UiHostWindow};
use crate::ui::template_runtime::{EditorUiHostRuntime, WORKBENCH_WINDOW_DOCUMENT_ID};
use crate::ui::v2_design_tokens::active_editor_v2_design_tokens_snapshot;
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry_with_region_defaults_and_scale_mode, ResolutionContext,
    ResolutionScaleMode, WorkbenchChromeMetrics, WorkbenchSkeleton,
};
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{
    EditorChromeSnapshot, MainPageSnapshot, SceneEntries, ViewContentKind,
};
use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId};
use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::scene::EntityId;
use zircon_runtime::ui::v2::{UiV2SourceFileReceipt, UiV2UnresolvedSourceImport};
use zircon_runtime_interface::ui::layout::UiSize;

use super::contract::canonical_hash;
use super::contract::ReviewCase;
mod inspector_components;
mod managed_inputs;
mod product_state;
pub(crate) use product_state::{
    build_product_workbench_snapshot, build_product_workbench_snapshot_with_context,
    export_product_workbench_case_snapshots, export_product_workbench_case_snapshots_with_context,
};
use product_state::{
    canonical_project_path, close_product_workbench_materialization,
    materialize_matching_product_workbench, review_project_root, ProductWorkbenchMaterialization,
    REVIEW_PROJECT_ROOT_ENV,
};

const PRESENTATION_SCHEMA: &str = "dev.zircon.editor.workbench-presentation";
const CASE_SNAPSHOTS_SCHEMA: &str = "dev.zircon.editor.workbench-case-snapshots";
const PRESENTATION_VERSION: u32 = 1;
const WORKBENCH_SOURCE_PATH: &str = "zircon_editor/assets/ui/editor/windows/workbench_window.zui";
const WORKBENCH_SHELL_DOCUMENT_ID: &str = "res://ui/editor/host/workbench_shell.zui";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SourceFingerprint {
    source_path: String,
    sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct WorkbenchPresentationSnapshot {
    schema: String,
    version: u32,
    source_fingerprint: SourceFingerprint,
    state_fingerprint: String,
    active_locale: String,
    layout: Value,
    window: WindowSnapshot,
    pages: PagesSnapshot,
    documents: DocumentsSnapshot,
    drawers: Vec<DrawerSnapshot>,
    hierarchy: HierarchySnapshot,
    inspector: Option<InspectorSnapshot>,
    status: StatusSnapshot,
}

impl WorkbenchPresentationSnapshot {
    pub(crate) fn state_fingerprint(&self) -> &str {
        &self.state_fingerprint
    }

    pub(crate) fn workbench_layout(&self) -> Result<WorkbenchLayout, String> {
        serde_json::from_value(self.layout.clone()).map_err(|error| error.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WindowSnapshot {
    id: String,
    title: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PagesSnapshot {
    active_id: String,
    items: Vec<PageSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PageSnapshot {
    id: String,
    title: String,
    activity_window_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DocumentsSnapshot {
    active_id: Option<String>,
    items: Vec<DocumentSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DocumentSnapshot {
    id: String,
    title: String,
    content_kind: String,
    source_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DrawerSnapshot {
    slot: String,
    mode: String,
    extent: f32,
    visible: bool,
    active_tab_id: Option<String>,
    active_view_id: Option<String>,
    tabs: Vec<DrawerTabSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DrawerTabSnapshot {
    id: String,
    descriptor_id: String,
    title: String,
    icon_key: String,
    content_kind: String,
    source_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HierarchySnapshot {
    filter_query: String,
    expanded_ids: Vec<String>,
    selected_ids: Vec<String>,
    rows: Vec<HierarchyRowSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HierarchyRowSnapshot {
    id: String,
    parent_id: Option<String>,
    name: String,
    kind: String,
    depth: u32,
    active: bool,
    has_children: bool,
    generation: String,
    subtree_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InspectorSnapshot {
    entity_id: String,
    name: String,
    parent: String,
    translation: [String; 3],
    #[serde(default)]
    rotation_degrees: Option<[String; 3]>,
    scale: [String; 3],
    render_layer_mask: u32,
    components: Vec<InspectorComponentSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InspectorComponentSnapshot {
    id: String,
    title: String,
    properties: Vec<InspectorPropertySnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InspectorPropertySnapshot {
    id: String,
    label: String,
    value: String,
    kind: String,
    editable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StatusSnapshot {
    primary: String,
    secondary: Option<String>,
    viewport_label: String,
    project_path: String,
}

pub(crate) struct ProductWorkbenchPresentation {
    pub(crate) snapshot: WorkbenchPresentationSnapshot,
    pub(crate) host_presentation: HostWindowPresentationData,
    pub(crate) source_document_ids: Vec<String>,
    pub(crate) source_receipts: Vec<UiV2SourceFileReceipt>,
    pub(crate) source_unresolved_imports: Vec<UiV2UnresolvedSourceImport>,
    _materialization: ProductWorkbenchMaterialization,
}

impl ProductWorkbenchPresentation {
    pub(crate) fn active_locale(&self) -> String {
        self._materialization
            ._controller
            .context()
            .i18n()
            .active_locale()
            .as_str()
            .to_owned()
    }

    pub(crate) fn close(self) -> Result<(), String> {
        close_product_workbench_materialization(self._materialization)
    }
}

/// Rebuilds a case from a matching actual editor state and runs it through the
/// same root/workbench bridges and host presentation builder used by the editor.
pub(crate) fn build_product_workbench_presentation(
    repo_root: &Path,
    logical_size: UiSize,
    dpi_scale: f32,
    case: &ReviewCase,
) -> Result<ProductWorkbenchPresentation, String> {
    let _ = (repo_root, logical_size, dpi_scale, case);
    Err("product workbench presentation requires the App-preflighted CoreHandle and runtime BuildSet".into())
}

pub(crate) fn build_product_workbench_presentation_with_context(
    repo_root: &Path,
    logical_size: UiSize,
    dpi_scale: f32,
    case: &ReviewCase,
    core: &CoreHandle,
    app_preflighted_build_set: &ZrRuntimeBuildSetId,
) -> Result<ProductWorkbenchPresentation, String> {
    let snapshot_value = case
        .data
        .get("workbenchPresentation")
        .ok_or("editor case has no workbenchPresentation snapshot")?;
    let expected = validate_workbench_presentation_snapshot(repo_root, snapshot_value)?;
    validate_case_viewport(logical_size, dpi_scale, case)?;
    let case_locale = canonical_product_locale(&case.locale)?;
    if expected.active_locale != case_locale {
        return Err(format!(
            "workbenchPresentation activeLocale {:?} does not match case locale {:?}",
            expected.active_locale, case.locale
        ));
    }
    let layout = expected.workbench_layout()?;
    let project_root = review_project_root()?;
    if canonical_project_path(&project_root)? != expected.status.project_path {
        return Err(format!(
            "workbenchPresentation project path does not match {REVIEW_PROJECT_ROOT_ENV}"
        ));
    }
    let managed_scene_fingerprint = case
        .data
        .get("managedSceneFingerprint")
        .ok_or("editor case has no managedSceneFingerprint")?;
    let checked_managed_scene_fingerprint =
        managed_inputs::validate_value(managed_scene_fingerprint, &project_root)?;
    let materialization = materialize_matching_product_workbench(
        repo_root,
        &expected,
        layout,
        core,
        app_preflighted_build_set,
    )?;
    match managed_inputs::capture(&project_root) {
        Ok(current) if current == checked_managed_scene_fingerprint => {}
        Ok(_) => {
            return match close_product_workbench_materialization(materialization) {
                Ok(()) => Err("managed project inputs changed while materializing the product case".into()),
                Err(close_error) => Err(format!(
                    "managed project inputs changed while materializing the product case; cleanup failed: {close_error}"
                )),
            };
        }
        Err(error) => {
            return match close_product_workbench_materialization(materialization) {
                Ok(()) => Err(error),
                Err(close_error) => Err(format!(
                    "{error}; managed project cleanup failed: {close_error}"
                )),
            };
        }
    }

    let physical_viewport = case.physical_viewport()?;
    let physical_shell_size = UiSize::new(
        physical_viewport.width as f32,
        physical_viewport.height as f32,
    );
    let metrics = WorkbenchChromeMetrics::default();
    // Normal host recompute passes this startup-loaded runtime into pane
    // projection; a fresh empty runtime silently falls back to native rows.
    let builtin_runtime = Arc::new(
        callback_dispatch::load_startup_builtin_template_runtime()
            .map_err(|error| error.to_string())?,
    );
    let mut root_bridge = callback_dispatch::BuiltinHostWindowTemplateBridge::new_with_runtime(
        builtin_runtime.clone(),
        physical_shell_size,
    )
    .map_err(|error| error.to_string())?;
    root_bridge
        .recompute_layout_with_workbench_model_at_scale(
            physical_shell_size,
            dpi_scale,
            &materialization.model,
            &metrics,
        )
        .map_err(|error| error.to_string())?;
    let mount_frame = root_bridge
        .root_shell_frames()
        .componentized_workbench_mount_frame(physical_shell_size);
    let mut workbench_bridge =
        callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge::new_mounted_with_runtime_and_i18n(
            builtin_runtime.clone(),
            mount_frame,
            materialization._controller.context().i18n_handle(),
        )
        .map_err(|error| error.to_string())?;
    workbench_bridge
        .prepare_chrome_state_for_layout(&materialization.chrome)
        .map_err(|error| error.to_string())?;
    let requested_expanded_ids = entity_ids_from_snapshot(
        &expected.hierarchy.expanded_ids,
        &materialization.chrome.scene_entries,
    )?;
    workbench_bridge
        .sync_scene_view_state(
            &materialization.chrome.scene_entries,
            &expected.hierarchy.filter_query,
            &requested_expanded_ids,
        )
        .map_err(|error| error.to_string())?;
    let effective_expanded_ids =
        workbench_bridge.effective_expanded_ids(&materialization.chrome.scene_entries);
    if effective_expanded_ids.into_iter().collect::<BTreeSet<_>>()
        != requested_expanded_ids.iter().copied().collect()
    {
        return Err(
            "workbenchPresentation expandedIds do not match the materialized hierarchy state"
                .into(),
        );
    }
    workbench_bridge
        .recompute_mounted_layout_with_workbench_model_at_scale(
            mount_frame,
            dpi_scale,
            &materialization.model,
            &metrics,
        )
        .map_err(|error| error.to_string())?;

    // Clone the model-synchronized production surface so review-only input is
    // applied before the retained runtime projection and cannot mutate the app.
    let workbench_template = builtin_runtime
        .project_document(WORKBENCH_WINDOW_DOCUMENT_ID)
        .map_err(|error| error.to_string())?;
    let mut workbench_surface = workbench_bridge.surface().clone();
    super::state::apply_case(&mut workbench_surface, case)?;
    workbench_surface
        .compute_layout(UiSize::new(
            mount_frame.width / dpi_scale,
            mount_frame.height / dpi_scale,
        ))
        .map_err(|error| error.to_string())?;
    if super::state::apply_scroll_position_for_case(&mut workbench_surface, case)? {
        workbench_surface
            .compute_layout(UiSize::new(
                mount_frame.width / dpi_scale,
                mount_frame.height / dpi_scale,
            ))
            .map_err(|error| error.to_string())?;
    }
    let workbench_projection = builtin_runtime
        .build_retained_host_projection_with_surface(&workbench_template, &workbench_surface)
        .map_err(|error| error.to_string())?;

    let resolution = ResolutionContext::from_physical_size_with_scale_mode(
        physical_shell_size,
        dpi_scale,
        ResolutionScaleMode::ConstantPhysical,
    );
    let token_region_preferred = WorkbenchSkeleton::default_region_extents_from_tokens(
        &active_editor_v2_design_tokens_snapshot(),
    )
    .into_iter()
    .map(|(region, logical_extent)| (region, resolution.to_physical(logical_extent)))
    .collect::<BTreeMap<_, _>>();
    let geometry = compute_workbench_shell_geometry_with_region_defaults_and_scale_mode(
        &materialization.model,
        &materialization.chrome,
        &materialization.layout,
        &materialization.descriptors,
        physical_shell_size,
        dpi_scale,
        ResolutionScaleMode::ConstantPhysical,
        &metrics,
        None,
        Some(&token_region_preferred),
    );

    let ui = UiHostWindow::new().map_err(|error| error.to_string())?;
    ui.window().set_size(PhysicalSize::new(
        physical_viewport.width,
        physical_viewport.height,
    ));
    ui.window().set_scale_factor(dpi_scale);
    let ui_asset_panes: BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation> =
        BTreeMap::new();
    let animation_panes: BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    > = BTreeMap::new();
    // These are the same capability-filtered pane data snapshots collected by
    // normal retained-host recompute from the live controller.
    let template_v2_data = materialization.template_v2_data.clone();
    let mut chrome_projection_cache =
        crate::ui::layouts::windows::workbench_host_window::HostChromeProjectionCache::default();
    let mut console_projection_cache = super::super::ConsolePaneProjectionCache::default();
    let mut module_plugins_projection_cache =
        super::super::ModulePluginsPaneProjectionCache::default();
    let _shell_presentation = apply_presentation_with_template_v2_data(
        &ui,
        &materialization.model,
        &materialization.chrome,
        &geometry,
        &materialization.preset_names,
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        &ModulePluginsPaneViewData::default(),
        &BuildExportPaneViewData::default(),
        &template_v2_data,
        Some(root_bridge.host_projection()),
        Some(&workbench_projection),
        workbench_bridge.layout_frames(),
        &FloatingWindowProjectionBundle::default(),
        Some(builtin_runtime.as_ref()),
        root_bridge.presentation_scale_factor(),
        "",
        &mut chrome_projection_cache,
        &mut console_projection_cache,
        &mut module_plugins_projection_cache,
        false,
    )
    .ok_or("full editor presentation unexpectedly declined the host shell update")?;
    let host_presentation = ui.get_host_presentation();
    let source_document_ids = mounted_template_document_ids(&materialization);
    let source_receipts = builtin_runtime
        .loaded_v2_source_receipts_for_documents(
            &source_document_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        )
        .map_err(|error| format!("mounted product template source receipt failed: {error}"))?;
    let source_unresolved_imports = builtin_runtime
        .loaded_v2_unresolved_imports_for_documents(
            &source_document_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        )
        .map_err(|error| format!("mounted product template import audit failed: {error}"))?;
    // The DTO contains model facts checked against the normal project model plus the exact
    // case-owned query/disclosure state synchronized into the same production bridge above.
    let snapshot = expected;
    Ok(ProductWorkbenchPresentation {
        snapshot,
        host_presentation,
        source_document_ids,
        source_receipts,
        source_unresolved_imports,
        _materialization: materialization,
    })
}

fn canonical_product_locale(locale: &str) -> Result<&'static str, String> {
    match locale {
        "en" | "en-US" => Ok("en"),
        "zh-CN" => Ok("zh-CN"),
        _ => Err(format!("unsupported product review locale {locale:?}")),
    }
}

fn validate_case_viewport(
    logical_size: UiSize,
    dpi_scale: f32,
    case: &ReviewCase,
) -> Result<UiSize, String> {
    if !dpi_scale.is_finite() || dpi_scale <= 0.0 {
        return Err("product presentation DPI scale must be positive and finite".into());
    }
    let expected_width = case.viewport.width as f32;
    let expected_height = case.viewport.height as f32;
    if !logical_size.width.is_finite()
        || !logical_size.height.is_finite()
        || (logical_size.width - expected_width).abs() > f32::EPSILON
        || (logical_size.height - expected_height).abs() > f32::EPSILON
        || (dpi_scale - case.dpi as f32).abs() > f32::EPSILON
    {
        return Err("product presentation viewport or DPI does not match the review case".into());
    }
    Ok(logical_size)
}

fn same_product_snapshot_state(
    actual: &WorkbenchPresentationSnapshot,
    expected: &WorkbenchPresentationSnapshot,
) -> Result<bool, String> {
    let mut actual = actual.clone();
    let mut expected = expected.clone();
    actual.state_fingerprint.clear();
    expected.state_fingerprint.clear();
    actual.hierarchy.filter_query.clear();
    expected.hierarchy.filter_query.clear();
    actual.hierarchy.expanded_ids.clear();
    expected.hierarchy.expanded_ids.clear();
    Ok(actual == expected)
}

fn entity_ids_from_snapshot(
    values: &[String],
    scene_entries: &SceneEntries,
) -> Result<Vec<EntityId>, String> {
    let mut entity_ids = values
        .iter()
        .map(|value| {
            let entity = value.parse::<EntityId>().map_err(|_| {
                format!("workbenchPresentation expandedIds has invalid entity {value:?}")
            })?;
            if entity.to_string() != *value {
                return Err(format!(
                    "workbenchPresentation expandedIds entity {value:?} is not canonical decimal"
                ));
            }
            let row = scene_entries
                .iter()
                .find(|row| row.entity == entity)
                .ok_or_else(|| {
                    format!(
                        "workbenchPresentation expandedIds references absent scene entity {entity}"
                    )
                })?;
            if !row.has_children {
                return Err(format!(
                    "workbenchPresentation expandedIds entity {entity} has no children"
                ));
            }
            Ok(entity)
        })
        .collect::<Result<Vec<_>, _>>()?;
    entity_ids.sort_unstable();
    if entity_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("workbenchPresentation expandedIds contains a duplicate entity".into());
    }
    Ok(entity_ids)
}

fn workbench_snapshot_from_model(
    repo_root: &Path,
    chrome: &EditorChromeSnapshot,
    model: &WorkbenchViewModel,
    layout: &WorkbenchLayout,
    descriptors: &[ViewDescriptor],
    active_locale: &str,
) -> Result<WorkbenchPresentationSnapshot, String> {
    let source_path = repo_root.join(WORKBENCH_SOURCE_PATH);
    let source_fingerprint = SourceFingerprint {
        source_path: WORKBENCH_SOURCE_PATH.to_owned(),
        sha256: super::contract::hash_file(&source_path)?,
    };
    with_state_fingerprint(WorkbenchPresentationSnapshot {
        schema: PRESENTATION_SCHEMA.to_owned(),
        version: PRESENTATION_VERSION,
        source_fingerprint,
        state_fingerprint: String::new(),
        active_locale: active_locale.to_owned(),
        layout: serde_json::to_value(layout).map_err(|error| error.to_string())?,
        window: window_snapshot(chrome),
        pages: pages_snapshot(chrome),
        documents: documents_snapshot(model, descriptors, repo_root),
        drawers: drawers_snapshot(model, chrome, descriptors, repo_root),
        hierarchy: hierarchy_snapshot(chrome)?,
        inspector: inspector_snapshot(chrome),
        status: status_snapshot(model, chrome),
    })
}

pub(crate) fn validate_workbench_presentation_snapshot(
    repo_root: &Path,
    value: &Value,
) -> Result<WorkbenchPresentationSnapshot, String> {
    let snapshot: WorkbenchPresentationSnapshot =
        serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
    if snapshot.schema != PRESENTATION_SCHEMA || snapshot.version != PRESENTATION_VERSION {
        return Err("unsupported workbench presentation schema or version".into());
    }
    if snapshot.source_fingerprint.source_path != WORKBENCH_SOURCE_PATH {
        return Err("workbench presentation points at an unsupported source asset".into());
    }
    let source_path = repo_root.join(WORKBENCH_SOURCE_PATH);
    if super::contract::hash_file(&source_path)? != snapshot.source_fingerprint.sha256 {
        return Err("workbench presentation source fingerprint is stale".into());
    }
    snapshot.workbench_layout()?;
    let mut canonical = serde_json::to_value(&snapshot).map_err(|error| error.to_string())?;
    canonical
        .as_object_mut()
        .ok_or("workbench presentation snapshot must serialize as an object")?
        .remove("stateFingerprint");
    if canonical_hash(&canonical)? != snapshot.state_fingerprint {
        return Err("workbench presentation state fingerprint is invalid".into());
    }
    Ok(snapshot)
}

fn window_snapshot(chrome: &EditorChromeSnapshot) -> WindowSnapshot {
    let active_id = &chrome.workbench.active_main_page;
    let page = chrome.workbench.main_pages.iter().find(|page| match page {
        MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => {
            id == active_id
        }
    });
    match page {
        Some(MainPageSnapshot::Workbench {
            title,
            activity_window,
            ..
        }) => WindowSnapshot {
            id: activity_window.0.clone(),
            title: title.clone(),
        },
        Some(MainPageSnapshot::Exclusive { id, title, .. }) => WindowSnapshot {
            id: id.0.clone(),
            title: title.clone(),
        },
        None => WindowSnapshot {
            id: active_id.0.clone(),
            title: String::new(),
        },
    }
}

fn pages_snapshot(chrome: &EditorChromeSnapshot) -> PagesSnapshot {
    let items = chrome
        .workbench
        .main_pages
        .iter()
        .map(|page| match page {
            MainPageSnapshot::Workbench {
                id,
                title,
                activity_window,
                ..
            } => PageSnapshot {
                id: id.0.clone(),
                title: title.clone(),
                activity_window_id: Some(activity_window.0.clone()),
            },
            MainPageSnapshot::Exclusive { id, title, .. } => PageSnapshot {
                id: id.0.clone(),
                title: title.clone(),
                activity_window_id: None,
            },
        })
        .collect();
    PagesSnapshot {
        active_id: chrome.workbench.active_main_page.0.clone(),
        items,
    }
}

fn documents_snapshot(
    model: &crate::ui::workbench::model::WorkbenchViewModel,
    descriptors: &[ViewDescriptor],
    repo_root: &Path,
) -> DocumentsSnapshot {
    DocumentsSnapshot {
        active_id: model
            .document_tabs
            .iter()
            .find(|tab| tab.active)
            .map(|tab| tab.instance_id.0.clone()),
        items: model
            .document_tabs
            .iter()
            .map(|tab| DocumentSnapshot {
                id: tab.instance_id.0.clone(),
                title: tab.title.clone(),
                content_kind: content_kind_name(tab.content_kind).to_owned(),
                source_path: descriptor_source_path(descriptors, &tab.descriptor_id, repo_root),
            })
            .collect(),
    }
}

fn drawers_snapshot(
    model: &crate::ui::workbench::model::WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    descriptors: &[ViewDescriptor],
    repo_root: &Path,
) -> Vec<DrawerSnapshot> {
    model
        .tool_windows
        .iter()
        .map(|(slot, drawer)| {
            let snapshot = chrome.workbench.drawers.get(slot);
            DrawerSnapshot {
                slot: drawer_slot_name(*slot).to_owned(),
                mode: drawer_mode_name(drawer.mode).to_owned(),
                extent: snapshot.map(|value| value.extent).unwrap_or_default(),
                visible: drawer.visible,
                active_tab_id: drawer.active_tab.as_ref().map(|id| id.0.clone()),
                active_view_id: snapshot
                    .and_then(|value| value.active_view.as_ref())
                    .map(|id| id.0.clone()),
                tabs: drawer
                    .tabs
                    .iter()
                    .map(|tab| DrawerTabSnapshot {
                        id: tab.instance_id.0.clone(),
                        descriptor_id: tab.descriptor_id.0.clone(),
                        title: tab.title.clone(),
                        icon_key: tab.icon_key.clone(),
                        content_kind: content_kind_name(tab.content_kind).to_owned(),
                        source_path: descriptor_source_path(
                            descriptors,
                            &tab.descriptor_id,
                            repo_root,
                        ),
                    })
                    .collect(),
            }
        })
        .collect()
}

fn hierarchy_snapshot(chrome: &EditorChromeSnapshot) -> Result<HierarchySnapshot, String> {
    let generation = chrome
        .scene_entries
        .inspection_generation()
        .ok_or("product hierarchy snapshot has no runtime inspection generation")?;
    Ok(HierarchySnapshot {
        filter_query: String::new(),
        expanded_ids: Vec::new(),
        selected_ids: chrome
            .scene_entries
            .selected_entities()
            .iter()
            .map(u64::to_string)
            .collect(),
        rows: chrome
            .scene_entries
            .iter()
            .map(|row| HierarchyRowSnapshot {
                id: row.entity.to_string(),
                parent_id: row.parent.map(|id| id.to_string()),
                name: row.display_name.clone(),
                kind: row.kind.clone(),
                depth: row.depth,
                active: row.active_in_hierarchy,
                has_children: row.has_children,
                generation: generation.to_string(),
                subtree_hash: row.subtree_hash.to_string(),
            })
            .collect(),
    })
}

fn inspector_snapshot(chrome: &EditorChromeSnapshot) -> Option<InspectorSnapshot> {
    chrome
        .inspector
        .as_ref()
        .map(|inspector| InspectorSnapshot {
            entity_id: inspector.id.to_string(),
            name: inspector.name.clone(),
            parent: inspector.parent.clone(),
            translation: inspector.translation.clone(),
            rotation_degrees: inspector.rotation_degrees.clone(),
            scale: inspector.scale.clone(),
            render_layer_mask: inspector.render_layer_mask,
            components: inspector_components::project_inspector_components(inspector),
        })
}

fn status_snapshot(
    model: &crate::ui::workbench::model::WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
) -> StatusSnapshot {
    StatusSnapshot {
        primary: model.status_bar.primary_text.clone(),
        secondary: model.status_bar.secondary_text.clone(),
        viewport_label: model.status_bar.viewport_label.clone(),
        project_path: chrome.project_path.clone(),
    }
}

fn descriptor_source_path(
    descriptors: &[ViewDescriptor],
    descriptor_id: &ViewDescriptorId,
    repo_root: &Path,
) -> Option<String> {
    let document_id = descriptors
        .iter()
        .find(|descriptor| descriptor.descriptor_id == *descriptor_id)?
        .pane_template
        .as_ref()?
        .body
        .document_id
        .strip_prefix("res://")?;
    if document_id.contains(['\\', ':'])
        || document_id
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    let relative = format!("zircon_editor/assets/{document_id}");
    repo_root.join(&relative).is_file().then_some(relative)
}

fn mounted_template_document_ids(materialization: &ProductWorkbenchMaterialization) -> Vec<String> {
    let mut document_ids = BTreeSet::from([
        WORKBENCH_SHELL_DOCUMENT_ID.to_owned(),
        WORKBENCH_WINDOW_DOCUMENT_ID.to_owned(),
    ]);
    for tab in materialization
        .model
        .document_tabs
        .iter()
        .filter(|tab| tab.active)
    {
        insert_descriptor_template_document(
            &mut document_ids,
            &materialization.descriptors,
            &tab.descriptor_id,
        );
    }
    for drawer in materialization
        .model
        .tool_windows
        .values()
        .filter(|drawer| drawer.visible)
    {
        let Some(active_tab_id) = drawer.active_tab.as_ref() else {
            continue;
        };
        let Some(active_tab) = drawer
            .tabs
            .iter()
            .find(|tab| &tab.instance_id == active_tab_id)
        else {
            continue;
        };
        insert_descriptor_template_document(
            &mut document_ids,
            &materialization.descriptors,
            &active_tab.descriptor_id,
        );
    }
    document_ids.into_iter().collect()
}

fn insert_descriptor_template_document(
    document_ids: &mut BTreeSet<String>,
    descriptors: &[ViewDescriptor],
    descriptor_id: &ViewDescriptorId,
) {
    let Some(document_id) = descriptors
        .iter()
        .find(|descriptor| descriptor.descriptor_id == *descriptor_id)
        .and_then(|descriptor| descriptor.pane_template.as_ref())
        .map(|template| template.body.document_id.clone())
    else {
        return;
    };
    document_ids.insert(document_id);
}

fn content_kind_name(kind: ViewContentKind) -> &'static str {
    match kind {
        ViewContentKind::Welcome => "welcome",
        ViewContentKind::Project => "project",
        ViewContentKind::Hierarchy => "hierarchy",
        ViewContentKind::Inspector => "inspector",
        ViewContentKind::Scene => "scene",
        ViewContentKind::Game => "game",
        ViewContentKind::Assets => "assets",
        ViewContentKind::Console => "console",
        ViewContentKind::PrefabEditor => "prefab_editor",
        ViewContentKind::AssetBrowser => "asset_browser",
        ViewContentKind::UiAssetEditor => "ui_asset_editor",
        ViewContentKind::UiComponentShowcase => "ui_component_showcase",
        ViewContentKind::AnimationSequenceEditor => "animation_sequence_editor",
        ViewContentKind::AnimationGraphEditor => "animation_graph_editor",
        ViewContentKind::RuntimeDiagnostics => "runtime_diagnostics",
        ViewContentKind::PerformanceTimeline => "performance_timeline",
        ViewContentKind::ModulePlugins => "module_plugins",
        ViewContentKind::BuildExport => "build_export",
        ViewContentKind::GeneratedBottom => "generated_bottom",
        ViewContentKind::Placeholder => "placeholder",
    }
}

fn drawer_slot_name(slot: crate::ui::workbench::layout::ActivityDrawerSlot) -> &'static str {
    match slot {
        crate::ui::workbench::layout::ActivityDrawerSlot::LeftTop => "leftTop",
        crate::ui::workbench::layout::ActivityDrawerSlot::LeftBottom => "leftBottom",
        crate::ui::workbench::layout::ActivityDrawerSlot::RightTop => "rightTop",
        crate::ui::workbench::layout::ActivityDrawerSlot::RightBottom => "rightBottom",
        crate::ui::workbench::layout::ActivityDrawerSlot::Bottom => "bottom",
    }
}

fn drawer_mode_name(mode: crate::ui::workbench::layout::ActivityDrawerMode) -> &'static str {
    match mode {
        crate::ui::workbench::layout::ActivityDrawerMode::Pinned => "pinned",
        crate::ui::workbench::layout::ActivityDrawerMode::AutoHide => "autoHide",
        crate::ui::workbench::layout::ActivityDrawerMode::Collapsed => "collapsed",
    }
}

fn with_state_fingerprint(
    mut snapshot: WorkbenchPresentationSnapshot,
) -> Result<WorkbenchPresentationSnapshot, String> {
    let mut value = serde_json::to_value(&snapshot).map_err(|error| error.to_string())?;
    value
        .as_object_mut()
        .ok_or("workbench presentation snapshot must serialize as an object")?
        .remove("stateFingerprint");
    snapshot.state_fingerprint = canonical_hash(&value)?;
    Ok(snapshot)
}
