use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use serde_json::json;

use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::CoreHandle;
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId, ProjectLaunchIntent,
    ProjectLaunchProfile, ProjectLaunchSource,
};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

use crate::core::editing::intent::EditorIntent;
use crate::core::editor_extension::EditorUiTemplatePaneDataSnapshot;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::core::project::SceneOpenRequest;
use crate::core::settings::{
    SettingValue, SettingValueSource, SettingsKey, SettingsScope, EDITOR_LOCALE_KEY,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::{EditorHostEventController, EditorHostStartupSession, EditorManager};
use crate::ui::retained_host::apply_host_appearance_from_tokens;
use crate::ui::v2_design_tokens::install_editor_v2_design_tokens;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::ViewDescriptor;

use super::super::contract::hash_file;
use super::managed_inputs;
use super::{
    same_product_snapshot_state, workbench_snapshot_from_model, WorkbenchPresentationSnapshot,
    CASE_SNAPSHOTS_SCHEMA, PRESENTATION_VERSION, WORKBENCH_SOURCE_PATH,
};

const REVIEW_EMPTY_SCENE_URI: &str = "res://scenes/workbench-review-empty.scene.toml";
const REVIEW_EMPTY_SCENE_RELATIVE_PATH: &str = "assets/scenes/workbench-review-empty.scene.toml";
pub(super) const REVIEW_PROJECT_ROOT_ENV: &str = "ZIRCON_EDITOR_WORKBENCH_REVIEW_PROJECT_ROOT";
const DEFAULT_LOGICAL_SIZE: (u32, u32) = (1280, 800);
static REVIEW_PROJECT_OPERATION_IDS: OnceLock<ProjectActivationOperationIdGenerator> =
    OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProductWorkbenchState {
    Default,
    Selected,
    Hierarchy,
    Empty,
    LongEnglish,
    LongChinese,
}

impl ProductWorkbenchState {
    const ALL: [Self; 6] = [
        Self::Default,
        Self::Selected,
        Self::Hierarchy,
        Self::Empty,
        Self::LongEnglish,
        Self::LongChinese,
    ];

    fn locale(self) -> &'static str {
        match self {
            Self::LongChinese => "zh-CN",
            Self::Default | Self::Selected | Self::Hierarchy | Self::Empty | Self::LongEnglish => {
                "en"
            }
        }
    }

    fn expected_long_text(self) -> Option<&'static str> {
        match self {
            Self::LongEnglish => Some(LONG_ENGLISH_SCENE_NAME),
            Self::LongChinese => Some(LONG_CHINESE_SCENE_NAME),
            Self::Default | Self::Selected | Self::Hierarchy | Self::Empty => None,
        }
    }
}

const LONG_ENGLISH_SCENE_NAME: &str = "Cube — an intentionally long English scene object label used to check hierarchy row and inspector title overflow without losing entity identity";
const LONG_CHINESE_SCENE_NAME: &str =
    "立方体节点的超长中文名称，用于验证层级行与检查器标题的溢出处理，同时保留实体身份与父子关系。";

pub(super) struct ProductWorkbenchMaterialization {
    pub(super) _core: CoreHandle,
    pub(super) _controller: EditorHostEventController,
    pub(super) chrome: EditorChromeSnapshot,
    pub(super) model: WorkbenchViewModel,
    pub(super) layout: WorkbenchLayout,
    pub(super) descriptors: Vec<ViewDescriptor>,
    pub(super) preset_names: Vec<String>,
    pub(super) template_v2_data: BTreeMap<String, EditorUiTemplatePaneDataSnapshot>,
    pub(super) snapshot: WorkbenchPresentationSnapshot,
    locale_overlay: Option<ProductWorkbenchLocaleOverlay>,
}

struct ProductWorkbenchLocaleOverlay {
    settings: Arc<crate::core::settings::SettingsAuthority>,
    key: SettingsKey,
    previous_session_value: Option<SettingValue>,
    restored: bool,
}

impl ProductWorkbenchLocaleOverlay {
    fn install(manager: &EditorManager, locale: &str) -> Result<Self, String> {
        let settings = Arc::clone(manager.context().settings());
        let key = SettingsKey::parse(EDITOR_LOCALE_KEY).map_err(|error| error.to_string())?;
        let previous = settings
            .resolved_setting(&key)
            .map_err(|error| error.to_string())?;
        let previous_session_value = (previous.source()
            == SettingValueSource::Scope(SettingsScope::Session))
        .then(|| previous.value().clone());
        let mut overlay = Self {
            settings,
            key,
            previous_session_value,
            restored: false,
        };
        overlay
            .settings
            .set(
                SettingsScope::Session,
                &overlay.key,
                SettingValue::Enum(locale.to_owned()),
            )
            .map_err(|error| error.to_string())?;
        let active_locale = manager.context().i18n().active_locale();
        if active_locale.as_str() != locale || overlay.settings.snapshot().locale() != locale {
            overlay.restore()?;
            return Err(format!(
                "settings locale overlay requested {locale:?} but the active editor locale is {:?}",
                active_locale.as_str()
            ));
        }
        Ok(overlay)
    }

    fn restore(&mut self) -> Result<(), String> {
        if self.restored {
            return Ok(());
        }
        if let Some(value) = self.previous_session_value.as_ref() {
            self.settings
                .set(SettingsScope::Session, &self.key, value.clone())
                .map_err(|error| error.to_string())?;
        } else {
            self.settings
                .clear(SettingsScope::Session, &self.key)
                .map_err(|error| error.to_string())?;
        }
        let snapshot = self.settings.snapshot();
        apply_host_appearance_from_tokens(snapshot.design_tokens());
        install_editor_v2_design_tokens(snapshot.as_ref());
        self.previous_session_value = None;
        self.restored = true;
        Ok(())
    }
}

impl Drop for ProductWorkbenchLocaleOverlay {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

impl Drop for ProductWorkbenchMaterialization {
    fn drop(&mut self) {
        let manager = self._controller.shell().lock().manager.clone();
        let _ = close_editor_project(&manager);
        if let Some(overlay) = &mut self.locale_overlay {
            let _ = overlay.restore();
        }
    }
}

/// Builds a factual shell snapshot from the editor's normal startup state, manager, and model.
pub(crate) fn build_product_workbench_snapshot(
    _repo_root: &Path,
    _state: ProductWorkbenchState,
) -> Result<WorkbenchPresentationSnapshot, String> {
    Err(
        "product workbench snapshot requires the App-preflighted CoreHandle and runtime BuildSet"
            .into(),
    )
}

pub(crate) fn build_product_workbench_snapshot_with_context(
    repo_root: &Path,
    state: ProductWorkbenchState,
    core: &CoreHandle,
    app_preflighted_build_set: &ZrRuntimeBuildSetId,
) -> Result<WorkbenchPresentationSnapshot, String> {
    let materialization = materialize_product_workbench(
        repo_root,
        state,
        None,
        state.locale(),
        core,
        app_preflighted_build_set,
    )?;
    let snapshot = materialization.snapshot.clone();
    close_product_workbench_materialization(materialization)?;
    Ok(snapshot)
}

pub(super) fn materialize_product_workbench(
    repo_root: &Path,
    state: ProductWorkbenchState,
    layout_override: Option<WorkbenchLayout>,
    locale: &str,
    core: &CoreHandle,
    app_preflighted_build_set: &ZrRuntimeBuildSetId,
) -> Result<ProductWorkbenchMaterialization, String> {
    let manager = core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .map_err(|error| {
            format!("App-preflighted CoreHandle has no active EditorManager: {error}")
        })?;
    manager.configure_project_runtime_build_set(Some(app_preflighted_build_set.clone()));
    let settings_snapshot = manager.context().settings().snapshot();
    apply_host_appearance_from_tokens(settings_snapshot.design_tokens());
    install_editor_v2_design_tokens(settings_snapshot.as_ref());
    let project_root = review_project_root()?;
    let project_root = canonicalize_review_project_root(&project_root)?;
    let mut locale_overlay = ProductWorkbenchLocaleOverlay::install(&manager, locale)?;
    let controller = match editor_controller_for(&project_root, manager.clone()) {
        Ok(controller) => controller,
        Err(error) => {
            let close_result = close_editor_project(&manager);
            let restore_result = locale_overlay.restore();
            close_result.map_err(|close_error| {
                format!("{error}; managed fixture cleanup failed: {close_error}")
            })?;
            restore_result.map_err(|restore_error| {
                format!("{error}; session locale restore failed: {restore_error}")
            })?;
            return Err(error);
        }
    };
    let mut materialization = match product_workbench_materialization(
        repo_root,
        core.clone(),
        controller,
        locale_overlay,
    ) {
        Ok(materialization) => materialization,
        Err(error) => {
            close_editor_project(&manager).map_err(|close_error| {
                format!("{error}; managed fixture cleanup failed: {close_error}")
            })?;
            return Err(error);
        }
    };
    if let Some(layout) = layout_override {
        let mut workspace = manager.project_workspace();
        workspace.workbench = layout;
        manager
            .apply_project_workspace(Some(workspace))
            .map_err(|error| error.to_string())?;
    }
    apply_product_workbench_state(&mut materialization, state)?;
    refresh_product_workbench_materialization(repo_root, &mut materialization)?;
    Ok(materialization)
}

pub(super) fn materialize_matching_product_workbench(
    repo_root: &Path,
    expected: &WorkbenchPresentationSnapshot,
    layout: WorkbenchLayout,
    core: &CoreHandle,
    app_preflighted_build_set: &ZrRuntimeBuildSetId,
) -> Result<ProductWorkbenchMaterialization, String> {
    for state in ProductWorkbenchState::ALL {
        if state.locale() != expected.active_locale {
            continue;
        }
        let materialization = materialize_product_workbench(
            repo_root,
            state,
            Some(layout.clone()),
            state.locale(),
            core,
            app_preflighted_build_set,
        )?;
        if same_product_snapshot_state(&materialization.snapshot, expected)? {
            return Ok(materialization);
        }
        close_product_workbench_materialization(materialization)?;
    }
    Err("workbenchPresentation does not match a default, selected, hierarchy, or empty managed-project state".into())
}

fn product_workbench_materialization(
    repo_root: &Path,
    core: CoreHandle,
    controller: EditorHostEventController,
    locale_overlay: ProductWorkbenchLocaleOverlay,
) -> Result<ProductWorkbenchMaterialization, String> {
    let chrome = controller.chrome_snapshot();
    let context = controller.project_command_eval_snapshot(&chrome);
    let model = controller.build_workbench_view_model(&chrome, &context);
    let layout = controller.current_layout();
    let descriptors = controller.descriptors();
    let preset_names = controller.preset_names();
    let template_v2_data = controller.ui_template_pane_data_snapshots();
    let active_locale = controller.context().i18n().active_locale();
    let snapshot = workbench_snapshot_from_model(
        repo_root,
        &chrome,
        &model,
        &layout,
        &descriptors,
        active_locale.as_str(),
    )?;
    Ok(ProductWorkbenchMaterialization {
        _core: core,
        _controller: controller,
        chrome,
        model,
        layout,
        descriptors,
        preset_names,
        template_v2_data,
        snapshot,
        locale_overlay: Some(locale_overlay),
    })
}

/// Context-free export is rejected because the editor crate cannot preflight the App BuildSet.
pub(crate) fn export_product_workbench_case_snapshots(
    _repo_root: &Path,
    _output_path: &Path,
) -> Result<(), String> {
    Err(
        "product workbench snapshot export requires the App-preflighted CoreHandle and runtime BuildSet"
            .into(),
    )
}

/// Writes distinct states rebuilt from the normal managed-project startup route.
pub(crate) fn export_product_workbench_case_snapshots_with_context(
    repo_root: &Path,
    output_path: &Path,
    core: &CoreHandle,
    app_preflighted_build_set: &ZrRuntimeBuildSetId,
) -> Result<(), String> {
    let project_root = review_project_root()?;
    let managed_scene_fingerprint = managed_inputs::capture(&project_root)?;

    let default_materialization = materialize_product_workbench(
        repo_root,
        ProductWorkbenchState::Default,
        None,
        ProductWorkbenchState::Default.locale(),
        core,
        app_preflighted_build_set,
    )?;
    let default_snapshot = default_materialization.snapshot.clone();
    let shared_layout = default_snapshot.workbench_layout()?;
    close_product_workbench_materialization(default_materialization)?;
    assert_default_product_selection(&default_snapshot)?;

    let selected_materialization = materialize_product_workbench(
        repo_root,
        ProductWorkbenchState::Selected,
        Some(shared_layout.clone()),
        ProductWorkbenchState::Selected.locale(),
        core,
        app_preflighted_build_set,
    )?;
    let selected_snapshot = selected_materialization.snapshot.clone();
    close_product_workbench_materialization(selected_materialization)?;
    assert_selected_product_state(&default_snapshot, &selected_snapshot)?;

    let hierarchy_materialization = materialize_product_workbench(
        repo_root,
        ProductWorkbenchState::Hierarchy,
        Some(shared_layout.clone()),
        ProductWorkbenchState::Hierarchy.locale(),
        core,
        app_preflighted_build_set,
    )?;
    let hierarchy_snapshot = hierarchy_materialization.snapshot.clone();
    close_product_workbench_materialization(hierarchy_materialization)?;
    assert_hierarchy_product_state(&default_snapshot, &hierarchy_snapshot)?;

    let empty_materialization = materialize_product_workbench(
        repo_root,
        ProductWorkbenchState::Empty,
        Some(shared_layout.clone()),
        ProductWorkbenchState::Empty.locale(),
        core,
        app_preflighted_build_set,
    )?;
    let empty_snapshot = empty_materialization.snapshot.clone();
    close_product_workbench_materialization(empty_materialization)?;
    assert_empty_product_state(&empty_snapshot)?;
    for snapshot in [
        &default_snapshot,
        &selected_snapshot,
        &hierarchy_snapshot,
        &empty_snapshot,
    ] {
        if snapshot.active_locale != "en" {
            return Err(
                "default, selected, hierarchy, and empty workbench states must use en locale"
                    .into(),
            );
        }
    }

    let long_english_materialization = materialize_product_workbench(
        repo_root,
        ProductWorkbenchState::LongEnglish,
        Some(shared_layout.clone()),
        ProductWorkbenchState::LongEnglish.locale(),
        core,
        app_preflighted_build_set,
    )?;
    let long_english_snapshot = long_english_materialization.snapshot.clone();
    close_product_workbench_materialization(long_english_materialization)?;
    assert_long_text_product_state(&long_english_snapshot, ProductWorkbenchState::LongEnglish)?;

    let long_chinese_materialization = materialize_product_workbench(
        repo_root,
        ProductWorkbenchState::LongChinese,
        Some(shared_layout.clone()),
        ProductWorkbenchState::LongChinese.locale(),
        core,
        app_preflighted_build_set,
    )?;
    let long_chinese_snapshot = long_chinese_materialization.snapshot.clone();
    close_product_workbench_materialization(long_chinese_materialization)?;
    assert_long_text_product_state(&long_chinese_snapshot, ProductWorkbenchState::LongChinese)?;

    if default_snapshot.workbench_layout()? != shared_layout
        || selected_snapshot.workbench_layout()? != shared_layout
        || hierarchy_snapshot.workbench_layout()? != shared_layout
        || empty_snapshot.workbench_layout()? != shared_layout
        || long_english_snapshot.workbench_layout()? != shared_layout
        || long_chinese_snapshot.workbench_layout()? != shared_layout
    {
        return Err("managed fixture state snapshots do not share one workbench layout".into());
    }
    if managed_inputs::capture(&project_root)? != managed_scene_fingerprint {
        return Err("product state export changed the managed fixture scene inputs".into());
    }
    let mut states = serde_json::Map::new();
    states.insert(
        "default".to_owned(),
        serde_json::to_value(default_snapshot).map_err(|e| e.to_string())?,
    );
    states.insert(
        "selected".to_owned(),
        serde_json::to_value(selected_snapshot).map_err(|e| e.to_string())?,
    );
    states.insert(
        "hierarchy".to_owned(),
        serde_json::to_value(hierarchy_snapshot).map_err(|e| e.to_string())?,
    );
    states.insert(
        "empty".to_owned(),
        serde_json::to_value(empty_snapshot).map_err(|e| e.to_string())?,
    );
    states.insert(
        "longEnglish".to_owned(),
        serde_json::to_value(long_english_snapshot).map_err(|e| e.to_string())?,
    );
    states.insert(
        "longChinese".to_owned(),
        serde_json::to_value(long_chinese_snapshot).map_err(|e| e.to_string())?,
    );
    let source_path = repo_root.join(WORKBENCH_SOURCE_PATH);
    let value = json!({
        "schema": CASE_SNAPSHOTS_SCHEMA,
        "version": PRESENTATION_VERSION,
        "sourceFingerprint": {
            "sourcePath": WORKBENCH_SOURCE_PATH,
            "sha256": hash_file(&source_path)?,
        },
        "managedSceneFingerprint": managed_scene_fingerprint,
        "states": states,
    });
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut bytes = serde_json::to_vec_pretty(&value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    std::fs::write(output_path, bytes).map_err(|error| error.to_string())
}

fn assert_default_product_selection(
    snapshot: &WorkbenchPresentationSnapshot,
) -> Result<(), String> {
    let cube = hierarchy_row_named(snapshot, "Cube")?;
    if snapshot.hierarchy.selected_ids.as_slice() != [cube.id.as_str()]
        || snapshot
            .inspector
            .as_ref()
            .is_none_or(|inspector| inspector.entity_id != cube.id)
    {
        return Err(
            "normal project startup did not select and inspect its authored Cube node".into(),
        );
    }
    Ok(())
}

fn assert_selected_product_state(
    default_snapshot: &WorkbenchPresentationSnapshot,
    selected_snapshot: &WorkbenchPresentationSnapshot,
) -> Result<(), String> {
    let sun = hierarchy_row_named(selected_snapshot, "Sun")?;
    if selected_snapshot.hierarchy.selected_ids.as_slice() != [sun.id.as_str()]
        || selected_snapshot
            .inspector
            .as_ref()
            .is_none_or(|inspector| inspector.entity_id != sun.id)
        || selected_snapshot.hierarchy.selected_ids == default_snapshot.hierarchy.selected_ids
        || selected_snapshot == default_snapshot
    {
        return Err(
            "selected product state must select the authored Sun instead of the startup Cube"
                .into(),
        );
    }
    Ok(())
}

fn assert_hierarchy_product_state(
    default_snapshot: &WorkbenchPresentationSnapshot,
    hierarchy_snapshot: &WorkbenchPresentationSnapshot,
) -> Result<(), String> {
    let default_cube = hierarchy_row_named(default_snapshot, "Cube")?;
    let default_sun = hierarchy_row_named(default_snapshot, "Sun")?;
    let cube = hierarchy_row_named(hierarchy_snapshot, "Cube")?;
    let sun = hierarchy_row_named(hierarchy_snapshot, "Sun")?;
    if default_cube.parent_id.is_some()
        || default_sun.parent_id.is_some()
        || cube.id != default_cube.id
        || sun.id != default_sun.id
        || cube.parent_id.as_deref() != Some(sun.id.as_str())
        || !sun.has_children
        || cube.depth != sun.depth + 1
        || hierarchy_snapshot == default_snapshot
    {
        return Err(
            "hierarchy product state must parent the authored Cube under Sun without saving it"
                .into(),
        );
    }
    Ok(())
}

fn assert_empty_product_state(snapshot: &WorkbenchPresentationSnapshot) -> Result<(), String> {
    if !snapshot.hierarchy.rows.is_empty()
        || !snapshot.hierarchy.selected_ids.is_empty()
        || snapshot.inspector.is_some()
    {
        return Err(
            "managed fixture empty state must contain no hierarchy rows, selection, or inspector"
                .into(),
        );
    }
    Ok(())
}

fn assert_long_text_product_state(
    snapshot: &WorkbenchPresentationSnapshot,
    state: ProductWorkbenchState,
) -> Result<(), String> {
    let expected_name = state
        .expected_long_text()
        .ok_or("long text assertion requires a long text product state")?;
    let cube = hierarchy_row_named(snapshot, expected_name)?;
    if snapshot.active_locale != state.locale()
        || snapshot.hierarchy.selected_ids.as_slice() != [cube.id.as_str()]
        || snapshot.inspector.as_ref().is_none_or(|inspector| {
            inspector.entity_id != cube.id || inspector.name != expected_name
        })
    {
        return Err(format!(
            "{} product state did not preserve its exact locale, renamed hierarchy row, selection, and inspector value",
            state.locale()
        ));
    }
    Ok(())
}

fn hierarchy_row_named<'a>(
    snapshot: &'a WorkbenchPresentationSnapshot,
    name: &str,
) -> Result<&'a super::HierarchyRowSnapshot, String> {
    let mut matches = snapshot
        .hierarchy
        .rows
        .iter()
        .filter(|row| row.name == name);
    let row = matches.next().ok_or_else(|| {
        format!("managed review product state has no authored {name} hierarchy row")
    })?;
    if matches.next().is_some() {
        return Err(format!(
            "managed review product state has duplicate {name} hierarchy rows"
        ));
    }
    Ok(row)
}

fn editor_controller_for(
    project_root: &Path,
    manager: std::sync::Arc<EditorManager>,
) -> Result<EditorHostEventController, String> {
    let operation_id = REVIEW_PROJECT_OPERATION_IDS
        .get_or_init(|| ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new()))
        .allocate()
        .ok_or("review project launch operation ID sequence is exhausted")?;
    let intent = ProjectLaunchIntent::open_existing(
        operation_id,
        ProjectLaunchSource::Cli,
        ProjectLaunchProfile::Normal,
        project_root,
    )
    .map_err(|error| error.to_string())?;
    // This is the application/CLI project route: it resolves the project,
    // prepares its authoring world, binds the startup scene document, and
    // constructs EditorState through EditorHostStartupSession.
    let startup = EditorHostStartupSession::open(
        manager,
        Some(EditorGuiStartupRequest::project(intent)),
        UVec2::new(DEFAULT_LOGICAL_SIZE.0, DEFAULT_LOGICAL_SIZE.1),
    )
    .map_err(|error| error.to_string())?;
    let (_session, controller) = startup.into_parts();
    Ok(controller)
}

pub(super) fn review_project_root() -> Result<PathBuf, String> {
    let configured = std::env::var_os(REVIEW_PROJECT_ROOT_ENV)
        .ok_or_else(|| format!("{REVIEW_PROJECT_ROOT_ENV} must name the managed review project"))?;
    let path = PathBuf::from(configured);
    if !path.is_absolute() {
        return Err(format!(
            "{REVIEW_PROJECT_ROOT_ENV} must be an absolute path"
        ));
    }
    let path = canonicalize_review_project_root(&path)?;
    if !path.is_dir() {
        return Err(format!(
            "{REVIEW_PROJECT_ROOT_ENV} does not name a project directory: {}",
            path.display()
        ));
    }
    Ok(path)
}

fn canonicalize_review_project_root(path: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(path).map_err(|error| {
        format!(
            "cannot resolve {REVIEW_PROJECT_ROOT_ENV} project root {}: {error}",
            path.display()
        )
    })
}

pub(super) fn canonical_project_path(path: &Path) -> Result<String, String> {
    Ok(canonicalize_review_project_root(path)?
        .to_string_lossy()
        .into_owned())
}

fn apply_product_workbench_state(
    materialization: &mut ProductWorkbenchMaterialization,
    state: ProductWorkbenchState,
) -> Result<(), String> {
    use crate::ui::workbench::shell_state::WorkbenchShellStateData;

    if state == ProductWorkbenchState::Empty {
        open_review_empty_scene(&materialization._controller)?;
        let snapshot = materialization._controller.chrome_snapshot();
        if !snapshot.scene_entries.is_empty() || snapshot.inspector.is_some() {
            return Err("managed fixture empty scene did not load with zero hierarchy rows and no inspector".into());
        }
        return Ok(());
    }

    fn apply(
        shell: &mut WorkbenchShellStateData,
        state: ProductWorkbenchState,
    ) -> Result<(), String> {
        match state {
            ProductWorkbenchState::Default => Ok(()),
            ProductWorkbenchState::Selected => {
                let selected = shell.state.viewport_controller.selection().active_primary();
                let target = shell
                    .state
                    .world
                    .with_world(|scene| {
                        let nodes = scene.node_records();
                        nodes
                            .iter()
                            .find(|node| {
                                node.directional_light.is_some() && Some(node.id) != selected
                            })
                            .or_else(|| {
                                nodes
                                    .iter()
                                    .find(|node| node.camera.is_some() && Some(node.id) != selected)
                            })
                            .map(|node| node.id)
                    })
                    .map_err(|error| error.to_string())?
                    .flatten()
                    .ok_or("managed review project has no distinct Sun or Camera to select")?;
                if !shell
                    .state
                    .apply_intent(EditorIntent::SelectNode(target))
                    .map_err(|error| error.to_string())?
                {
                    return Err(
                        "managed review project selection intent made no state change".into(),
                    );
                }
                Ok(())
            }
            ProductWorkbenchState::Hierarchy => {
                let relationship = shell
                    .state
                    .world
                    .with_world(|scene| {
                        let nodes = scene.node_records();
                        let suns = nodes
                            .iter()
                            .filter(|node| node.name == "Sun" && node.directional_light.is_some())
                            .collect::<Vec<_>>();
                        let cubes = nodes
                            .iter()
                            .filter(|node| node.name == "Cube" && node.mesh.is_some())
                            .collect::<Vec<_>>();
                        (suns.len() == 1 && cubes.len() == 1).then(|| {
                            (cubes[0].id, cubes[0].parent, suns[0].id)
                        })
                    })
                    .map_err(|error| error.to_string())?
                    .flatten()
                    .ok_or("managed review hierarchy state requires exactly one authored Sun and Cube node")?;
                let (cube, cube_parent, sun) = relationship;
                if cube_parent.is_some() {
                    return Err("managed review hierarchy state requires the authored Cube to start at the scene root".into());
                }
                if !shell
                    .state
                    .apply_intent(EditorIntent::SetParent(cube, Some(sun)))
                    .map_err(|error| error.to_string())?
                {
                    return Err("managed review hierarchy intent made no state change".into());
                }
                Ok(())
            }
            ProductWorkbenchState::LongEnglish | ProductWorkbenchState::LongChinese => {
                let expected_name = state
                    .expected_long_text()
                    .ok_or("long text state has no deterministic scene name")?;
                let selected = shell.state.viewport_controller.selection().active_primary();
                let cube = shell
                    .state
                    .world
                    .with_world(|scene| {
                        scene
                            .node_records()
                            .iter()
                            .find(|node| node.name == "Cube" && node.mesh.is_some())
                            .map(|node| node.id)
                    })
                    .map_err(|error| error.to_string())?
                    .flatten()
                    .ok_or("managed review long text state requires the authored Cube node")?;
                if selected != Some(cube)
                    && !shell
                        .state
                        .apply_intent(EditorIntent::SelectNode(cube))
                        .map_err(|error| error.to_string())?
                {
                    return Err("long text product state could not select the authored Cube".into());
                }
                if !shell
                    .state
                    .apply_intent(EditorIntent::RenameNode(cube, expected_name.to_owned()))
                    .map_err(|error| error.to_string())?
                {
                    return Err(
                        "long text product state RenameNode did not change the authored Cube"
                            .into(),
                    );
                }
                Ok(())
            }
            ProductWorkbenchState::Empty => {
                unreachable!("empty scene opens outside the locked shell mutation")
            }
        }
    }

    let shell = materialization._controller.shell();
    apply(&mut shell.lock(), state)
}

fn open_review_empty_scene(controller: &EditorHostEventController) -> Result<(), String> {
    let project_root = review_project_root()?;
    let source_path = project_root.join(REVIEW_EMPTY_SCENE_RELATIVE_PATH);
    if !source_path.is_file() {
        return Err(format!(
            "managed review empty state requires the existing project-owned scene source {}; it will not create or overwrite it",
            source_path.display()
        ));
    }
    let scene_uri = AssetUri::parse(REVIEW_EMPTY_SCENE_URI).map_err(|error| error.to_string())?;
    let ticket = controller.begin_scene_picker()?;
    controller.submit_scene_open_request(ticket, SceneOpenRequest::new(scene_uri))?;
    let snapshot = controller.chrome_snapshot();
    if !snapshot.scene_entries.is_empty() || snapshot.inspector.is_some() {
        return Err(format!(
            "project-owned empty scene {REVIEW_EMPTY_SCENE_URI} opened with authored scene rows"
        ));
    }
    Ok(())
}

fn refresh_product_workbench_materialization(
    repo_root: &Path,
    materialization: &mut ProductWorkbenchMaterialization,
) -> Result<(), String> {
    let controller = &materialization._controller;
    let chrome = controller.chrome_snapshot();
    let context = controller.project_command_eval_snapshot(&chrome);
    let model = controller.build_workbench_view_model(&chrome, &context);
    let layout = controller.current_layout();
    let descriptors = controller.descriptors();
    let preset_names = controller.preset_names();
    let template_v2_data = controller.ui_template_pane_data_snapshots();
    let active_locale = controller.context().i18n().active_locale();
    let snapshot = workbench_snapshot_from_model(
        repo_root,
        &chrome,
        &model,
        &layout,
        &descriptors,
        active_locale.as_str(),
    )?;
    materialization.chrome = chrome;
    materialization.model = model;
    materialization.layout = layout;
    materialization.descriptors = descriptors;
    materialization.preset_names = preset_names;
    materialization.template_v2_data = template_v2_data;
    materialization.snapshot = snapshot;
    Ok(())
}

pub(super) fn close_product_workbench_materialization(
    mut materialization: ProductWorkbenchMaterialization,
) -> Result<(), String> {
    let manager = materialization._controller.shell().lock().manager.clone();
    close_editor_project(&manager)?;
    if let Some(overlay) = &mut materialization.locale_overlay {
        overlay.restore()?;
    }
    materialization.locale_overlay = None;
    Ok(())
}

fn close_editor_project(manager: &EditorManager) -> Result<(), String> {
    let Some(operation) = manager
        .begin_project_close()
        .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };
    manager
        .commit_project_close(&operation)
        .map_err(|error| error.to_string())?;
    manager
        .finalize_project_close(&operation)
        .map_err(|error| error.to_string())?;
    Ok(())
}
