use slint::{ComponentHandle, Model};
use std::collections::BTreeMap;

use super::{
    apply_presentation as apply_presentation_with_module_plugins,
    apply_presentation_impl::to_slint_host_scene_data,
};
use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};
use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    self as host_window, collect_floating_windows as collect_floating_windows_with_module_plugins,
    document_pane as document_pane_with_module_plugins,
};
use crate::ui::slint_host::callback_dispatch::BuiltinHostWindowTemplateBridge;
use crate::ui::slint_host::floating_window_projection::{
    build_floating_window_projection_bundle, FloatingWindowProjectionBundle,
};
use crate::ui::slint_host::shell_pointer::HostShellPointerRoute;
use crate::ui::slint_host::tab_drag::host_shell_pointer_route_group_key;
use crate::ui::template_runtime::{
    EditorUiCompatibilityHarness, EditorUiHostRuntime, UiComponentShowcaseDemoEventInput,
};
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::fixture::{default_preview_fixture, PreviewFixture};
use crate::ui::workbench::layout::{DockEdge, MainHostPageLayout, WorkbenchLayout};
use crate::ui::workbench::layout::{
    DocumentNode, FloatingWindowLayout, MainPageId, TabStackLayout,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, ProjectOverviewSnapshot,
};
use crate::ui::workbench::startup::{
    EditorSessionMode, NewProjectFormSnapshot, RecentProjectItemSnapshot, RecentProjectValidation,
    WelcomePaneSnapshot,
};
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PreferredHost,
    ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId, ViewKind,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::ui::layout::{UiFrame, UiSize};

fn root_shell_fixture() -> (
    PreviewFixture,
    EditorChromeSnapshot,
    WorkbenchViewModel,
    BTreeMap<String, UiAssetEditorPanePresentation>,
    BTreeMap<String, AnimationEditorPanePresentation>,
) {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    (fixture, chrome, model, BTreeMap::new(), BTreeMap::new())
}

fn welcome_shell_fixture() -> (
    EditorChromeSnapshot,
    WorkbenchViewModel,
    BTreeMap<String, UiAssetEditorPanePresentation>,
    BTreeMap<String, AnimationEditorPanePresentation>,
) {
    let descriptors = vec![ViewDescriptor::new(
        ViewDescriptorId::new("editor.welcome"),
        ViewKind::ActivityWindow,
        "Welcome",
    )
    .with_preferred_host(PreferredHost::ExclusiveMainPage)
    .with_icon_key("welcome")];
    let welcome_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.welcome#1"),
        descriptor_id: ViewDescriptorId::new("editor.welcome"),
        title: "Welcome".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::ExclusivePage(MainPageId::new("page:welcome")),
    };
    let chrome = EditorChromeSnapshot::build(
        EditorDataSnapshot {
            scene_entries: Vec::new(),
            inspector: None,
            status_line: "Ready".to_string(),
            hovered_axis: None,
            viewport_size: UVec2::new(1280, 720),
            scene_viewport_settings: crate::scene::viewport::SceneViewportSettings::default(),
            mesh_import_path: String::new(),
            project_overview: ProjectOverviewSnapshot::default(),
            asset_activity: AssetWorkspaceSnapshot::default(),
            asset_browser: AssetWorkspaceSnapshot::default(),
            project_path: String::new(),
            session_mode: EditorSessionMode::Welcome,
            welcome: WelcomePaneSnapshot {
                title: "Open or Create".to_string(),
                subtitle: "Recent projects and a renderable empty-project template".to_string(),
                status_message: "No recent project".to_string(),
                browse_supported: false,
                recent_projects: vec![RecentProjectItemSnapshot {
                    display_name: "Broken".to_string(),
                    path: "E:/Missing/Broken".to_string(),
                    validation: RecentProjectValidation::Missing,
                    last_opened_label: "Just now".to_string(),
                    selected: true,
                }],
                form: NewProjectFormSnapshot {
                    project_name: "WelcomeProject".to_string(),
                    location: "E:/Work".to_string(),
                    project_path_preview: "E:/Work/WelcomeProject".to_string(),
                    template_label: "Renderable Empty".to_string(),
                    can_create: true,
                    can_open_existing: true,
                    validation_message: String::new(),
                },
            },
            project_open: false,
            can_undo: false,
            can_redo: false,
        },
        &WorkbenchLayout {
            active_main_page: MainPageId::new("page:welcome"),
            main_pages: vec![MainHostPageLayout::ExclusiveActivityWindowPage {
                id: MainPageId::new("page:welcome"),
                title: "Welcome".to_string(),
                window_instance: welcome_instance.instance_id.clone(),
            }],
            drawers: BTreeMap::new(),
            activity_windows: Default::default(),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        vec![welcome_instance],
        descriptors,
    );
    let model = WorkbenchViewModel::build(&chrome);
    (chrome, model, BTreeMap::new(), BTreeMap::new())
}

fn apply_presentation(
    ui: &crate::ui::slint_host::UiHostWindow,
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    preset_names: &[String],
    active_preset_name: Option<&str>,
    ui_asset_panes: &BTreeMap<String, UiAssetEditorPanePresentation>,
    animation_panes: &BTreeMap<String, AnimationEditorPanePresentation>,
    runtime_diagnostics: Option<&zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
    shared_root_frames: Option<
        &crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames,
    >,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) {
    apply_presentation_with_module_plugins(
        ui,
        model,
        chrome,
        geometry,
        preset_names,
        active_preset_name,
        ui_asset_panes,
        animation_panes,
        runtime_diagnostics,
        &host_window::ModulePluginsPaneViewData::default(),
        shared_root_frames,
        floating_window_projection_bundle,
        None,
    );
}

fn document_pane(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    ui_asset_panes: &BTreeMap<String, UiAssetEditorPanePresentation>,
    animation_panes: &BTreeMap<String, AnimationEditorPanePresentation>,
    runtime_diagnostics: Option<&zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
) -> host_window::PaneData {
    document_pane_with_module_plugins(
        model,
        chrome,
        ui_asset_panes,
        animation_panes,
        runtime_diagnostics,
        &host_window::ModulePluginsPaneViewData::default(),
    )
}

fn collect_floating_windows(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    ui_asset_panes: &BTreeMap<String, UiAssetEditorPanePresentation>,
    animation_panes: &BTreeMap<String, AnimationEditorPanePresentation>,
    runtime_diagnostics: Option<&zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> Vec<host_window::FloatingWindowData> {
    collect_floating_windows_with_module_plugins(
        model,
        chrome,
        geometry,
        ui_asset_panes,
        animation_panes,
        runtime_diagnostics,
        &host_window::ModulePluginsPaneViewData::default(),
        floating_window_projection_bundle,
    )
}

fn host_frame_rect(x: f32, y: f32, width: f32, height: f32) -> host_window::FrameRect {
    host_window::FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn template_frame(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> crate::ui::layouts::views::ViewTemplateFrameData {
    crate::ui::layouts::views::ViewTemplateFrameData {
        x,
        y,
        width,
        height,
    }
}

fn mount_node(
    node_id: &str,
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> crate::ui::layouts::views::ViewTemplateNodeData {
    crate::ui::layouts::views::ViewTemplateNodeData {
        node_id: node_id.into(),
        control_id: control_id.into(),
        role: "Mount".into(),
        text: "".into(),
        dispatch_kind: "".into(),
        action_id: "".into(),
        surface_variant: "".into(),
        text_tone: "".into(),
        button_variant: "".into(),
        font_size: 0.0,
        font_weight: 0,
        text_align: "left".into(),
        overflow: "".into(),
        corner_radius: 0.0,
        border_width: 0.0,
        frame: template_frame(x, y, width, height),
    }
}

fn host_tabs(ids: &[&str]) -> slint::ModelRc<host_window::TabData> {
    model_rc(
        ids.iter()
            .enumerate()
            .map(|(index, id)| host_window::TabData {
                id: (*id).into(),
                slot: format!("slot-{index}").into(),
                title: format!("Tab {index}").into(),
                icon_key: "tab".into(),
                active: index == 0,
                closeable: true,
            })
            .collect(),
    )
}

fn host_pane(id: &str, title: &str) -> host_window::PaneData {
    host_window::PaneData {
        id: id.into(),
        slot: format!("{id}-slot").into(),
        kind: format!("{title}Kind").into(),
        title: title.into(),
        icon_key: format!("{title}-icon").into(),
        subtitle: format!("{title} subtitle").into(),
        info: format!("{title} info").into(),
        show_empty: false,
        empty_title: format!("{title} empty").into(),
        empty_body: format!("{title} body").into(),
        primary_action_label: format!("{title} primary").into(),
        primary_action_id: format!("{title}.primary").into(),
        secondary_action_label: format!("{title} secondary").into(),
        secondary_action_id: format!("{title}.secondary").into(),
        secondary_hint: format!("{title} hint").into(),
        show_toolbar: true,
        viewport: blank_viewport_chrome(),
        body_compat: host_window::PaneBodyCompatData {
            hierarchy: host_window::HierarchyPaneViewData::default(),
            inspector: host_window::InspectorPaneViewData::default(),
            console: host_window::ConsolePaneViewData::default(),
            assets_activity: host_window::AssetsActivityPaneViewData::default(),
            asset_browser: host_window::AssetBrowserPaneViewData::default(),
            project_overview: host_window::ProjectOverviewPaneViewData::default(),
            module_plugins: host_window::ModulePluginsPaneViewData::default(),
            ui_asset: UiAssetEditorPanePresentation::default(),
            animation: host_window::AnimationEditorPaneViewData::default(),
        },
        pane_presentation: None,
    }
}

#[test]
fn component_showcase_template_numeric_drag_tracks_two_axis_delta() {
    let template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));

    assert!(
        template.contains("property <float> drag_last_y"),
        "numeric showcase drag should track vertical pointer movement"
    );
    assert!(
        template.contains("root.drag_last_y = self.mouse-y / 1px"),
        "numeric showcase drag should record the y origin and last y position"
    );
    assert!(
        template.contains(
            "(self.mouse-x / 1px - root.drag_last_x) - (self.mouse-y / 1px - root.drag_last_y)"
        ),
        "numeric showcase drag should map right/up to positive deltas and left/down to negative deltas"
    );
}

#[test]
fn component_showcase_template_fields_dispatch_live_edits() {
    let template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));

    assert!(
        template.contains("import { LineEdit } from \"std-widgets.slint\";"),
        "component showcase fields should use the generic Slint text input primitive"
    );
    assert!(
        template.contains("callback node_edited(control_id: string, dispatch_kind: string, action_id: string, value: string);"),
        "component showcase fields should expose live edited text through a generic callback"
    );
    assert!(
        template.contains("root.node.component_role == \"input-field\""),
        "InputField rows should render an editable field instead of a static value label"
    );
    assert!(
        template.contains("root.node.component_role == \"number-field\""),
        "NumberField rows should render an editable field instead of a static value label"
    );
    assert!(
        template.contains("edited(value) => {"),
        "editable component rows should dispatch typed value text changes"
    );
}

#[test]
fn component_showcase_template_popup_options_dispatch_candidate_selection() {
    let template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));
    let pane_content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_content.slint"
    ));
    let host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface_host_context.slint"
    ));
    let callback_wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    assert!(
        template.contains("callback node_option_selected(control_id: string, dispatch_kind: string, action_id: string, option_id: string);"),
        "component showcase popup rows should expose the selected option id through a generic callback"
    );
    assert!(
        template.contains("for option[index] in root.node.options"),
        "component showcase popup should render real candidate rows instead of only a summary label"
    );
    assert!(
        template.contains("root.node_option_selected("),
        "candidate rows should dispatch an option-selection event when clicked"
    );
    assert!(
        pane_content.contains("component_showcase_option_selected(control_id, action_id, option_id)"),
        "PaneContent should route generic option-selection callbacks into the showcase host context"
    );
    assert!(
        host_context.contains("callback component_showcase_option_selected(control_id: string, action_id: string, option_id: string);"),
        "PaneSurfaceHostContext should expose a typed option-selection callback"
    );
    assert!(
        callback_wiring.contains("on_component_showcase_option_selected"),
        "Rust callback wiring should forward option row clicks to the editor host"
    );
}

#[test]
fn component_showcase_template_action_chips_dispatch_secondary_actions() {
    let template_node_data = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_node_data.slint"
    ));
    let template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));
    let pane_content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_content.slint"
    ));
    let pane_actions = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/pane_surface_actions.rs"
    ));

    assert!(
        template_node_data.contains("export struct TemplatePaneActionData"),
        "TemplatePane should expose generic secondary action metadata"
    );
    assert!(
        template.contains("callback node_action_invoked(control_id: string, dispatch_kind: string, action_id: string);"),
        "TemplatePane action chips should dispatch their action ids generically"
    );
    assert!(
        template.contains("for action[index] in root.node.actions"),
        "TemplatePane should render all projected secondary actions as rows/chips"
    );
    assert!(
        pane_content.contains("node_action_invoked(control_id, dispatch_kind, action_id)"),
        "PaneContent should route secondary component actions"
    );
    assert!(
        pane_actions.contains("AssetFieldClear")
            && pane_actions.contains("AssetFieldLocate")
            && pane_actions.contains("AssetFieldOpen"),
        "asset action chips should map clear/locate/open into the showcase reducer"
    );
}

#[test]
fn component_showcase_template_materializes_runtime_component_primitives() {
    let template_node_data = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_node_data.slint"
    ));
    let template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));

    assert!(
        template_node_data.contains("value_number: float"),
        "TemplatePane should carry the numeric value behind NumberField and RangeField rows"
    );
    assert!(
        template_node_data.contains("value_percent: float"),
        "TemplatePane should carry normalized progress/range values for retained static rendering"
    );
    assert!(
        template_node_data.contains("value_color: color"),
        "TemplatePane should carry parsed ColorField swatches as Slint colors"
    );
    assert!(
        template.contains("root.node.component_role == \"checkbox\""),
        "Checkbox rows should render a checkbox primitive instead of only a value label"
    );
    assert!(
        template.contains("root.node.component_role == \"radio\""),
        "Radio rows should render a radio primitive instead of only a value label"
    );
    assert!(
        template.contains("root.node.component_role == \"toggle-button\""),
        "ToggleButton rows should render a switch primitive instead of only a value label"
    );
    assert!(
        template.contains("root.node.component_role == \"progress-bar\""),
        "ProgressBar rows should render a progress primitive"
    );
    assert!(
        template.contains("root.node.component_role == \"range-field\""),
        "RangeField rows should render a slider track and editable value"
    );
    assert!(
        template.contains("root.node.component_role == \"color-field\""),
        "ColorField rows should render a swatch primitive"
    );
    assert!(
        template.contains("root.node.value_percent"),
        "Progress and range primitives should use normalized retained values"
    );
    assert!(
        template.contains("root.node.value_color"),
        "Color swatches should use the parsed retained color"
    );
}

#[test]
fn component_showcase_template_materializes_visual_feedback_and_vector_primitives() {
    let template_node_data = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_node_data.slint"
    ));
    let template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));

    assert!(
        template_node_data.contains("media_source: string"),
        "TemplatePane should carry retained image/svg sources for generic host rendering"
    );
    assert!(
        template_node_data.contains("icon_name: string"),
        "TemplatePane should carry semantic icon names separately from value text"
    );
    assert!(
        template_node_data.contains("vector_components: [float]"),
        "TemplatePane should carry Vector2/3/4 components as typed retained values"
    );
    assert!(
        template.contains("root.node.component_role == \"image\""),
        "Image rows should render a thumbnail primitive"
    );
    assert!(
        template.contains("root.node.component_role == \"icon\""),
        "Icon rows should render an icon primitive"
    );
    assert!(
        template.contains("root.node.component_role == \"svg-icon\""),
        "SvgIcon rows should render an SVG/icon primitive"
    );
    assert!(
        template.contains("root.node.component_role == \"spinner\""),
        "Spinner rows should render a retained feedback primitive"
    );
    assert!(
        template.contains("root.node.component_role == \"badge\""),
        "Badge rows should render a tag primitive"
    );
    assert!(
        template.contains("root.node.component_role == \"help-row\""),
        "Help rows should render a validation/help primitive"
    );
    assert!(
        template.contains("root.node.component_role == \"separator\""),
        "Separator rows should render a retained divider primitive"
    );
    assert!(
        template.contains("root.node.component_role == \"vector3-field\""),
        "Vector fields should render axis chips instead of plain value text"
    );
    assert!(
        template.contains("root.node.media_source"),
        "Image and svg primitives should use retained media source metadata"
    );
    assert!(
        template.contains("root.node.icon_name"),
        "Icon primitives should use retained icon metadata"
    );
    assert!(
        template.contains("root.node.vector_components"),
        "Vector primitives should iterate retained component values"
    );
}

#[test]
fn component_showcase_pane_projects_runtime_component_nodes_for_template_pane() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let (_fixture, chrome, _model, _ui_asset_panes, _animation_panes) = root_shell_fixture();
    let body_spec = PaneBodySpec::new(
        "editor.window.ui_component_showcase",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    );
    let body = host_window::build_pane_body_presentation(
        &body_spec,
        &host_window::PanePayloadBuildContext::new(&chrome),
    );
    let mut pane = host_pane("component-showcase", "UI Component Showcase");
    pane.kind = "UiComponentShowcase".into();
    pane.pane_presentation = Some(host_window::PanePresentation::new(
        host_window::PaneShellPresentation::new(
            "UI Component Showcase",
            "ui-components",
            "Runtime components",
            "",
            None,
            false,
            blank_viewport_chrome(),
        ),
        body,
    ));

    let slint_pane = super::pane_data_conversion::to_slint_component_showcase_pane_from_host_pane(
        &pane,
        host_window::PaneContentSize::new(1080.0, 720.0),
    );

    let nodes = (0..slint_pane.nodes.row_count())
        .filter_map(|row| slint_pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let number = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "NumberFieldDemo")
        .expect("component showcase pane should expose NumberFieldDemo");
    assert_eq!(number.component_role.as_str(), "number-field");
    assert_eq!(number.value_text.as_str(), "42");
    assert_eq!(number.value_number, 42.0);
    assert_eq!(number.value_percent, 0.42);
    assert_eq!(number.dispatch_kind.as_str(), "showcase");
    assert_eq!(
        number.action_id.as_str(),
        "UiComponentShowcase/NumberFieldDragUpdate"
    );
    assert_eq!(
        number.drag_action_id.as_str(),
        "UiComponentShowcase/NumberFieldDragUpdate"
    );
    assert_eq!(
        number.begin_drag_action_id.as_str(),
        "UiComponentShowcase/NumberFieldDragBegin"
    );
    assert_eq!(
        number.end_drag_action_id.as_str(),
        "UiComponentShowcase/NumberFieldDragEnd"
    );
    assert_eq!(
        number.edit_action_id.as_str(),
        "UiComponentShowcase/NumberFieldChanged"
    );

    let input = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "InputFieldDemo")
        .expect("component showcase pane should expose InputFieldDemo");
    assert_eq!(
        input.edit_action_id.as_str(),
        "UiComponentShowcase/InputFieldChanged"
    );

    let range = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "RangeFieldDemo")
        .expect("component showcase pane should expose RangeFieldDemo");
    assert_eq!(range.value_number, 68.0);
    assert_eq!(range.value_percent, 0.68);
    assert_eq!(
        range.drag_action_id.as_str(),
        "UiComponentShowcase/RangeFieldDragUpdate"
    );
    assert_eq!(
        range.edit_action_id.as_str(),
        "UiComponentShowcase/RangeFieldChanged"
    );

    let dropdown = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "DropdownDemo")
        .expect("component showcase pane should expose DropdownDemo");
    assert!(dropdown.popup_open);
    assert_eq!(dropdown.selection_state.as_str(), "multi");
    assert_eq!(dropdown.options_text.as_str(), "runtime, editor, debug");
    assert_eq!(dropdown.options.row_count(), 3);
    assert_eq!(dropdown.options.row_data(0).as_deref(), Some("runtime"));
    assert_eq!(dropdown.options.row_data(1).as_deref(), Some("editor"));
    assert_eq!(dropdown.options.row_data(2).as_deref(), Some("debug"));

    let combo_box = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComboBoxDemo")
        .expect("component showcase pane should expose ComboBoxDemo");
    assert!(!combo_box.popup_open);
    assert_eq!(
        combo_box.action_id.as_str(),
        "UiComponentShowcase/ComboBoxOpenPopup"
    );
    assert_eq!(combo_box.options_text.as_str(), "material, fluent, native");
    assert_eq!(combo_box.options.row_count(), 3);

    let progress = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ProgressBarDemo")
        .expect("component showcase pane should expose ProgressBarDemo");
    assert_eq!(progress.value_number, 0.62);
    assert_eq!(progress.value_percent, 0.62);

    let color = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ColorFieldDemo")
        .expect("component showcase pane should expose ColorFieldDemo");
    assert_eq!(color.value_color, slint::Color::from_rgb_u8(77, 137, 255));

    let image = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ImageDemo")
        .expect("component showcase pane should expose ImageDemo");
    assert_eq!(
        image.media_source.as_str(),
        "res://textures/grid.albedo.png"
    );

    let icon = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "IconDemo")
        .expect("component showcase pane should expose IconDemo");
    assert_eq!(icon.icon_name.as_str(), "options-outline");

    let svg_icon = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "SvgIconDemo")
        .expect("component showcase pane should expose SvgIconDemo");
    assert_eq!(
        svg_icon.media_source.as_str(),
        "ionicons/options-outline.svg"
    );

    let vector2 = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "Vector2FieldDemo")
        .expect("component showcase pane should expose Vector2Demo");
    assert_eq!(vector2.vector_components.row_count(), 2);
    assert_eq!(vector2.vector_components.row_data(0), Some(12.0));
    assert_eq!(vector2.vector_components.row_data(1), Some(24.0));

    let vector3 = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "Vector3FieldDemo")
        .expect("component showcase pane should expose Vector3Demo");
    assert_eq!(vector3.vector_components.row_count(), 3);
    assert_eq!(vector3.vector_components.row_data(0), Some(0.0));
    assert_eq!(vector3.vector_components.row_data(1), Some(1.0));
    assert_eq!(vector3.vector_components.row_data(2), Some(0.0));

    let vector4 = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "Vector4FieldDemo")
        .expect("component showcase pane should expose Vector4Demo");
    assert_eq!(vector4.vector_components.row_count(), 4);
    assert_eq!(vector4.vector_components.row_data(3), Some(1.0));

    let asset = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "AssetFieldDemo")
        .expect("component showcase pane should expose AssetFieldDemo");
    assert_eq!(asset.actions.row_count(), 3);
    assert_eq!(
        asset.actions.row_data(0).map(|action| action.label),
        Some("Find".into())
    );
    assert_eq!(
        asset.actions.row_data(1).map(|action| action.label),
        Some("Open".into())
    );
    assert_eq!(
        asset.actions.row_data(2).map(|action| action.label),
        Some("Clear".into())
    );

    let array = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ArrayFieldDemo")
        .expect("component showcase pane should expose ArrayFieldDemo");
    assert_eq!(array.actions.row_count(), 4);

    let map = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "MapFieldDemo")
        .expect("component showcase pane should expose MapFieldDemo");
    assert_eq!(map.actions.row_count(), 3);

    let event_log = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComponentShowcaseEventLog")
        .expect("component showcase pane should expose event log node");
    assert!(event_log.text.contains("Registered events"));
}

#[test]
fn component_showcase_pane_uses_supplied_runtime_demo_state() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let (_fixture, chrome, _model, _ui_asset_panes, _animation_panes) = root_shell_fixture();
    let body_spec = PaneBodySpec::new(
        "editor.window.ui_component_showcase",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    );
    let body = host_window::build_pane_body_presentation(
        &body_spec,
        &host_window::PanePayloadBuildContext::new(&chrome),
    );
    let mut pane = host_pane("component-showcase", "UI Component Showcase");
    pane.kind = "UiComponentShowcase".into();
    pane.pane_presentation = Some(host_window::PanePresentation::new(
        host_window::PaneShellPresentation::new(
            "UI Component Showcase",
            "ui-components",
            "Runtime components",
            "",
            None,
            false,
            blank_viewport_chrome(),
        ),
        body,
    ));

    let mut runtime = EditorUiHostRuntime::default();
    runtime
        .load_builtin_host_templates()
        .expect("built-in host templates should load");
    let binding = runtime
        .project_document("editor.window.ui_component_showcase")
        .expect("showcase projection should compile")
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == "UiComponentShowcase/NumberFieldDragUpdate")
        .expect("showcase should expose NumberField drag binding")
        .binding;
    runtime
        .apply_showcase_demo_binding(&binding, UiComponentShowcaseDemoEventInput::DragDelta(5.0))
        .expect("showcase runtime should accept NumberField drag input");
    let combo_open_binding = runtime
        .project_document("editor.window.ui_component_showcase")
        .expect("showcase projection should compile")
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == "UiComponentShowcase/ComboBoxOpenPopup")
        .expect("showcase should expose ComboBox open binding")
        .binding;
    runtime
        .apply_showcase_demo_binding(&combo_open_binding, UiComponentShowcaseDemoEventInput::None)
        .expect("showcase runtime should accept ComboBox popup input");

    let slint_pane =
        super::pane_data_conversion::to_slint_component_showcase_pane_from_host_pane_with_runtime(
            &pane,
            host_window::PaneContentSize::new(1080.0, 720.0),
            &runtime,
        );

    let nodes = (0..slint_pane.nodes.row_count())
        .filter_map(|row| slint_pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let number = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "NumberFieldDemo")
        .expect("component showcase pane should expose NumberFieldDemo");
    assert_eq!(number.value_text.as_str(), "47");

    let combo_box = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComboBoxDemo")
        .expect("component showcase pane should expose ComboBoxDemo");
    assert!(combo_box.popup_open);
    assert_eq!(
        combo_box.action_id.as_str(),
        "UiComponentShowcase/ComboBoxChanged"
    );

    let event_log = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComponentShowcaseEventLog")
        .expect("component showcase pane should expose event log node");
    assert!(
        event_log
            .text
            .contains("NumberFieldDemo -> DragDelta.NumberField = 47"),
        "event log should reflect the supplied runtime state: {}",
        event_log.text
    );
}

#[test]
fn host_scene_projection_converts_host_owned_panes_to_slint_panes() {
    let mut scene = host_window::HostWindowSceneData {
        layout: host_window::HostWindowLayoutData {
            center_band_frame: host_frame_rect(0.0, 0.0, 1200.0, 700.0),
            status_bar_frame: host_frame_rect(0.0, 700.0, 1200.0, 24.0),
            left_region_frame: host_frame_rect(0.0, 0.0, 240.0, 700.0),
            document_region_frame: host_frame_rect(240.0, 0.0, 720.0, 700.0),
            right_region_frame: host_frame_rect(960.0, 0.0, 240.0, 700.0),
            bottom_region_frame: host_frame_rect(240.0, 500.0, 720.0, 200.0),
            left_splitter_frame: host_frame_rect(240.0, 0.0, 4.0, 700.0),
            right_splitter_frame: host_frame_rect(956.0, 0.0, 4.0, 700.0),
            bottom_splitter_frame: host_frame_rect(240.0, 496.0, 720.0, 4.0),
            viewport_content_frame: host_frame_rect(240.0, 64.0, 720.0, 432.0),
        },
        metrics: host_window::HostWindowSurfaceMetricsData {
            outer_margin_px: 8.0,
            rail_width_px: 40.0,
            top_bar_height_px: 40.0,
            host_bar_height_px: 24.0,
            panel_header_height_px: 28.0,
            document_header_height_px: 32.0,
        },
        orchestration: host_window::HostWindowSurfaceOrchestrationData {
            left_rail_width_px: 40.0,
            right_rail_width_px: 40.0,
            left_stack_width_px: 240.0,
            right_stack_width_px: 240.0,
            left_panel_width_px: 200.0,
            right_panel_width_px: 200.0,
            bottom_panel_height_px: 200.0,
            main_content_y_px: 40.0,
            document_zone_x_px: 240.0,
            right_stack_x_px: 960.0,
            bottom_panel_y_px: 500.0,
            left_tab_origin_x_px: 8.0,
            left_tab_origin_y_px: 48.0,
            document_tab_origin_x_px: 248.0,
            document_tab_origin_y_px: 48.0,
            right_tab_origin_x_px: 968.0,
            right_tab_origin_y_px: 48.0,
            bottom_tab_origin_x_px: 248.0,
            bottom_tab_origin_y_px: 508.0,
        },
        menu_chrome: host_window::HostMenuChromeData {
            outer_margin_px: 8.0,
            top_bar_height_px: 40.0,
            save_project_enabled: true,
            undo_enabled: true,
            redo_enabled: true,
            delete_enabled: true,
            preset_names: model_rc(vec!["Default".into()]),
            active_preset_name: "Default".into(),
            resolved_preset_name: "Default".into(),
        },
        page_chrome: host_window::HostPageChromeData {
            top_bar_height_px: 40.0,
            host_bar_height_px: 24.0,
            tabs: host_tabs(&["document-tab"]),
            project_path: "res://project".into(),
        },
        status_bar: host_window::HostStatusBarData {
            status_bar_frame: host_frame_rect(0.0, 700.0, 1200.0, 24.0),
            status_primary: "Ready".into(),
            status_secondary: "Idle".into(),
            viewport_label: "Viewport".into(),
        },
        resize_layer: host_window::HostResizeLayerData {
            left_splitter_frame: host_frame_rect(240.0, 0.0, 4.0, 700.0),
            right_splitter_frame: host_frame_rect(956.0, 0.0, 4.0, 700.0),
            bottom_splitter_frame: host_frame_rect(240.0, 496.0, 720.0, 4.0),
        },
        drag_overlay: host_window::HostTabDragOverlayData {
            left_drop_enabled: true,
            right_drop_enabled: true,
            bottom_drop_enabled: true,
            left_drop_width_px: 120.0,
            right_drop_width_px: 120.0,
            bottom_drop_height_px: 120.0,
            main_content_y_px: 40.0,
            main_content_height_px: 660.0,
            document_zone_x_px: 240.0,
            document_zone_width_px: 720.0,
            bottom_drop_top_px: 500.0,
            drag_overlay_bottom_px: 700.0,
        },
        left_dock: host_window::HostSideDockSurfaceData {
            region_frame: host_frame_rect(0.0, 0.0, 240.0, 700.0),
            surface_key: "left-surface".into(),
            rail_before_panel: true,
            tabs: host_tabs(&["left-tab"]),
            pane: host_pane("left-pane", "Left"),
            rail_width_px: 40.0,
            panel_width_px: 200.0,
            panel_header_height_px: 28.0,
            tab_origin_x_px: 8.0,
            tab_origin_y_px: 48.0,
        },
        document_dock: host_window::HostDocumentDockSurfaceData {
            region_frame: host_frame_rect(240.0, 0.0, 720.0, 700.0),
            surface_key: "document-surface".into(),
            tabs: host_tabs(&["document-tab"]),
            pane: host_pane("document-pane", "Document"),
            header_height_px: 32.0,
            tab_origin_x_px: 248.0,
            tab_origin_y_px: 48.0,
        },
        right_dock: host_window::HostSideDockSurfaceData {
            region_frame: host_frame_rect(960.0, 0.0, 240.0, 700.0),
            surface_key: "right-surface".into(),
            rail_before_panel: false,
            tabs: host_tabs(&["right-tab"]),
            pane: host_pane("right-pane", "Right"),
            rail_width_px: 40.0,
            panel_width_px: 200.0,
            panel_header_height_px: 28.0,
            tab_origin_x_px: 968.0,
            tab_origin_y_px: 48.0,
        },
        bottom_dock: host_window::HostBottomDockSurfaceData {
            region_frame: host_frame_rect(240.0, 500.0, 720.0, 200.0),
            surface_key: "bottom-surface".into(),
            tabs: host_tabs(&["bottom-tab"]),
            pane: host_pane("bottom-pane", "Bottom"),
            expanded: true,
            header_height_px: 28.0,
            tab_origin_x_px: 248.0,
            tab_origin_y_px: 508.0,
        },
        floating_layer: host_window::HostFloatingWindowLayerData {
            floating_windows: model_rc(vec![host_window::FloatingWindowData {
                window_id: "floating-window".into(),
                title: "Floating".into(),
                frame: host_frame_rect(320.0, 160.0, 360.0, 240.0),
                target_group: "floating/window".into(),
                left_edge_target_group: "floating/window/left".into(),
                right_edge_target_group: "floating/window/right".into(),
                top_edge_target_group: "floating/window/top".into(),
                bottom_edge_target_group: "floating/window/bottom".into(),
                focus_target_id: "floating-focus".into(),
                tabs: host_tabs(&["floating-tab"]),
                active_pane: host_pane("floating-pane", "Floating"),
            }]),
            header_height_px: 24.0,
        },
    };
    let ui_asset_nodes = vec![
        mount_node(
            "ui_asset/header_panel",
            "HeaderPanel",
            11.0,
            12.0,
            640.0,
            56.0,
        ),
        mount_node(
            "ui_asset/header_asset_row",
            "HeaderAssetRow",
            21.0,
            18.0,
            620.0,
            10.0,
        ),
        mount_node(
            "ui_asset/header_status_row",
            "HeaderStatusRow",
            21.0,
            28.0,
            620.0,
            10.0,
        ),
        mount_node(
            "ui_asset/header_action_row",
            "HeaderActionRow",
            21.0,
            40.0,
            620.0,
            20.0,
        ),
        mount_node(
            "ui_asset/left_column",
            "LeftColumn",
            16.0,
            80.0,
            220.0,
            240.0,
        ),
        mount_node(
            "ui_asset/palette_panel",
            "PalettePanel",
            16.0,
            80.0,
            220.0,
            240.0,
        ),
        mount_node(
            "ui_asset/center_column",
            "CenterColumn",
            260.0,
            80.0,
            420.0,
            928.0,
        ),
        mount_node(
            "ui_asset/designer_panel",
            "DesignerPanel",
            260.0,
            80.0,
            420.0,
            300.0,
        ),
        mount_node(
            "ui_asset/designer_canvas_panel",
            "DesignerCanvasPanel",
            270.0,
            108.0,
            400.0,
            214.0,
        ),
        mount_node(
            "ui_asset/render_stack_panel",
            "RenderStackPanel",
            270.0,
            328.0,
            400.0,
            80.0,
        ),
        mount_node(
            "ui_asset/action_bar_panel",
            "ActionBarPanel",
            270.0,
            414.0,
            400.0,
            88.0,
        ),
        mount_node(
            "ui_asset/action_insert_row",
            "ActionInsertRow",
            280.0,
            422.0,
            380.0,
            24.0,
        ),
        mount_node(
            "ui_asset/action_reparent_row",
            "ActionReparentRow",
            280.0,
            450.0,
            380.0,
            24.0,
        ),
        mount_node(
            "ui_asset/action_structure_row",
            "ActionStructureRow",
            280.0,
            478.0,
            380.0,
            24.0,
        ),
        mount_node(
            "ui_asset/source_info_panel",
            "SourceInfoPanel",
            270.0,
            392.0,
            400.0,
            58.0,
        ),
        mount_node(
            "ui_asset/mock_workspace_panel",
            "MockWorkspacePanel",
            270.0,
            528.0,
            400.0,
            326.0,
        ),
        mount_node(
            "ui_asset/mock_subjects_panel",
            "MockSubjectsPanel",
            270.0,
            528.0,
            400.0,
            72.0,
        ),
        mount_node(
            "ui_asset/mock_editor_panel",
            "MockEditorPanel",
            270.0,
            606.0,
            400.0,
            170.0,
        ),
        mount_node(
            "ui_asset/mock_state_graph_panel",
            "MockStateGraphPanel",
            270.0,
            782.0,
            400.0,
            72.0,
        ),
        mount_node(
            "ui_asset/source_text_panel",
            "SourceTextPanel",
            270.0,
            860.0,
            400.0,
            148.0,
        ),
        mount_node(
            "ui_asset/inspector_panel",
            "InspectorPanel",
            700.0,
            80.0,
            260.0,
            240.0,
        ),
        mount_node(
            "ui_asset/inspector_content_panel",
            "InspectorContentPanel",
            710.0,
            106.0,
            240.0,
            204.0,
        ),
        mount_node(
            "ui_asset/stylesheet_panel",
            "StylesheetPanel",
            700.0,
            330.0,
            260.0,
            170.0,
        ),
        mount_node(
            "ui_asset/stylesheet_action_row",
            "StylesheetActionRow",
            710.0,
            356.0,
            240.0,
            24.0,
        ),
        mount_node(
            "ui_asset/stylesheet_state_primary_row",
            "StylesheetStatePrimaryRow",
            710.0,
            384.0,
            240.0,
            24.0,
        ),
        mount_node(
            "ui_asset/stylesheet_state_secondary_row",
            "StylesheetStateSecondaryRow",
            710.0,
            412.0,
            240.0,
            24.0,
        ),
        mount_node(
            "ui_asset/stylesheet_content_panel",
            "StylesheetContentPanel",
            710.0,
            441.0,
            240.0,
            49.0,
        ),
    ];
    let ui_asset_node = |control_id: &str| {
        ui_asset_nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .cloned()
            .unwrap_or_else(|| panic!("missing ui asset test node `{control_id}`"))
    };
    scene.left_dock.pane.body_compat.ui_asset = UiAssetEditorPanePresentation {
        asset_id: "asset://ui/test.ui.toml".to_string(),
        mode: "split".to_string(),
        last_error: "clean".to_string(),
        selection_summary: "Root".to_string(),
        palette_items: vec!["Button".to_string()],
        palette_selected_index: 0,
        nodes: ui_asset_nodes.clone(),
        center_column_node: ui_asset_node("CenterColumn"),
        designer_panel_node: ui_asset_node("DesignerPanel"),
        designer_canvas_panel_node: ui_asset_node("DesignerCanvasPanel"),
        inspector_panel_node: ui_asset_node("InspectorPanel"),
        stylesheet_panel_node: ui_asset_node("StylesheetPanel"),
        ..UiAssetEditorPanePresentation::default()
    };
    scene.right_dock.pane.body_compat.animation = host_window::AnimationEditorPaneViewData {
        nodes: model_rc(vec![
            mount_node(
                "root/header_panel",
                "AnimationEditorHeaderPanel",
                14.0,
                18.0,
                520.0,
                64.0,
            ),
            mount_node(
                "root/header_mode_row",
                "AnimationEditorHeaderModeRow",
                26.0,
                28.0,
                496.0,
                12.0,
            ),
            mount_node(
                "root/header_path_row",
                "AnimationEditorHeaderPathRow",
                26.0,
                44.0,
                496.0,
                14.0,
            ),
            mount_node(
                "root/header_status_row",
                "AnimationEditorHeaderStatusRow",
                26.0,
                62.0,
                496.0,
                12.0,
            ),
            mount_node(
                "root/body_panel",
                "AnimationEditorBodyPanel",
                14.0,
                82.0,
                520.0,
                318.0,
            ),
            mount_node(
                "root/sequence_content_panel",
                "AnimationSequenceContentPanel",
                26.0,
                94.0,
                496.0,
                294.0,
            ),
            mount_node(
                "root/sequence_timeline_row",
                "AnimationSequenceTimelineRow",
                26.0,
                94.0,
                496.0,
                12.0,
            ),
            mount_node(
                "root/sequence_selection_row",
                "AnimationSequenceSelectionRow",
                26.0,
                112.0,
                496.0,
                12.0,
            ),
            mount_node(
                "root/sequence_tracks_panel",
                "AnimationSequenceTracksPanel",
                26.0,
                138.0,
                496.0,
                250.0,
            ),
            mount_node(
                "root/graph_content_panel",
                "AnimationGraphContentPanel",
                26.0,
                94.0,
                496.0,
                294.0,
            ),
            mount_node(
                "root/graph_parameters_panel",
                "AnimationGraphParametersPanel",
                26.0,
                94.0,
                496.0,
                120.0,
            ),
            mount_node(
                "root/graph_nodes_panel",
                "AnimationGraphNodesPanel",
                26.0,
                234.0,
                496.0,
                154.0,
            ),
            mount_node(
                "root/state_machine_content_panel",
                "AnimationStateMachineContentPanel",
                26.0,
                94.0,
                496.0,
                294.0,
            ),
            mount_node(
                "root/state_machine_entry_row",
                "AnimationStateMachineEntryRow",
                26.0,
                94.0,
                496.0,
                12.0,
            ),
            mount_node(
                "root/state_machine_states_panel",
                "AnimationStateMachineStatesPanel",
                26.0,
                118.0,
                496.0,
                112.0,
            ),
            mount_node(
                "root/state_machine_transitions_panel",
                "AnimationStateMachineTransitionsPanel",
                26.0,
                242.0,
                496.0,
                146.0,
            ),
        ]),
        mode: "sequence".into(),
        asset_path: "asset://animation/walk.anim".into(),
        status: "Looping".into(),
        selection: "Track: Root/Hips".into(),
        current_frame: 24,
        timeline_start_frame: 0,
        timeline_end_frame: 48,
        playback_label: "Playing".into(),
        track_items: model_rc(vec!["Root Translation".into(), "Left Hand".into()]),
        parameter_items: model_rc(vec!["Speed".into()]),
        node_items: model_rc(vec!["Blend".into()]),
        state_items: model_rc(vec!["Idle".into()]),
        transition_items: model_rc(vec!["Idle -> Walk".into()]),
    };
    scene.document_dock.pane.kind = "Project".into();
    scene.document_dock.pane.body_compat.project_overview =
        host_window::ProjectOverviewPaneViewData {
            nodes: model_rc(vec![
                crate::ui::layouts::views::ViewTemplateNodeData {
                    node_id: "root/outer_panel".into(),
                    control_id: "ProjectOverviewOuterPanel".into(),
                    role: "Panel".into(),
                    text: "".into(),
                    dispatch_kind: "".into(),
                    action_id: "".into(),
                    surface_variant: "panel".into(),
                    text_tone: "".into(),
                    button_variant: "".into(),
                    font_size: 0.0,
                    font_weight: 0,
                    text_align: "left".into(),
                    overflow: "".into(),
                    corner_radius: 8.0,
                    border_width: 1.0,
                    frame: crate::ui::layouts::views::ViewTemplateFrameData {
                        x: 16.0,
                        y: 14.0,
                        width: 688.0,
                        height: 654.0,
                    },
                },
                crate::ui::layouts::views::ViewTemplateNodeData {
                    node_id: "root/header_path".into(),
                    control_id: "ProjectOverviewPathText".into(),
                    role: "Label".into(),
                    text: "res://project".into(),
                    dispatch_kind: "".into(),
                    action_id: "".into(),
                    surface_variant: "".into(),
                    text_tone: "muted".into(),
                    button_variant: "".into(),
                    font_size: 10.0,
                    font_weight: 400,
                    text_align: "left".into(),
                    overflow: "elide".into(),
                    corner_radius: 0.0,
                    border_width: 0.0,
                    frame: crate::ui::layouts::views::ViewTemplateFrameData {
                        x: 32.0,
                        y: 52.0,
                        width: 656.0,
                        height: 16.0,
                    },
                },
                crate::ui::layouts::views::ViewTemplateNodeData {
                    node_id: "root/catalog_panel".into(),
                    control_id: "ProjectOverviewCatalogPanel".into(),
                    role: "Panel".into(),
                    text: "".into(),
                    dispatch_kind: "".into(),
                    action_id: "".into(),
                    surface_variant: "inset".into(),
                    text_tone: "".into(),
                    button_variant: "".into(),
                    font_size: 0.0,
                    font_weight: 0,
                    text_align: "left".into(),
                    overflow: "".into(),
                    corner_radius: 6.0,
                    border_width: 1.0,
                    frame: crate::ui::layouts::views::ViewTemplateFrameData {
                        x: 32.0,
                        y: 206.0,
                        width: 656.0,
                        height: 68.0,
                    },
                },
            ]),
        };
    scene.bottom_dock.pane.kind = "Assets".into();
    scene.bottom_dock.pane.body_compat.assets_activity = host_window::AssetsActivityPaneViewData {
        nodes: model_rc(vec![
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/toolbar_panel".into(),
                control_id: "AssetsActivityToolbarPanel".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 18.0,
                    y: 16.0,
                    width: 680.0,
                    height: 122.0,
                },
            },
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/tree_panel".into(),
                control_id: "AssetsActivityTreePanel".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 18.0,
                    y: 148.0,
                    width: 248.0,
                    height: 284.0,
                },
            },
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/utility_tabs_row".into(),
                control_id: "AssetsActivityUtilityTabsRow".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 30.0,
                    y: 454.0,
                    width: 656.0,
                    height: 32.0,
                },
            },
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/reference_right_panel".into(),
                control_id: "AssetsActivityReferenceRightPanel".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 364.0,
                    y: 496.0,
                    width: 322.0,
                    height: 110.0,
                },
            },
        ]),
    };
    scene.left_dock.pane.body_compat.hierarchy = host_window::HierarchyPaneViewData {
        nodes: model_rc(vec![crate::ui::layouts::views::ViewTemplateNodeData {
            node_id: "root/list_panel".into(),
            control_id: "HierarchyListPanel".into(),
            role: "Mount".into(),
            text: "".into(),
            dispatch_kind: "".into(),
            action_id: "".into(),
            surface_variant: "".into(),
            text_tone: "".into(),
            button_variant: "".into(),
            font_size: 0.0,
            font_weight: 0,
            text_align: "left".into(),
            overflow: "".into(),
            corner_radius: 0.0,
            border_width: 0.0,
            frame: crate::ui::layouts::views::ViewTemplateFrameData {
                x: 8.0,
                y: 8.0,
                width: 184.0,
                height: 260.0,
            },
        }]),
        hierarchy_nodes: model_rc(vec![
            host_window::SceneNodeData {
                id: "entity://root".into(),
                name: "Root".into(),
                depth: 0,
                selected: true,
            },
            host_window::SceneNodeData {
                id: "entity://child".into(),
                name: "Child".into(),
                depth: 1,
                selected: false,
            },
        ]),
    };
    scene.right_dock.pane.body_compat.inspector = host_window::InspectorPaneViewData {
        nodes: model_rc(vec![
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/content_panel".into(),
                control_id: "InspectorContentPanel".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 10.0,
                    y: 10.0,
                    width: 220.0,
                    height: 180.0,
                },
            },
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/header_panel".into(),
                control_id: "InspectorHeaderPanel".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 10.0,
                    y: 10.0,
                    width: 220.0,
                    height: 22.0,
                },
            },
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/name_row".into(),
                control_id: "InspectorNameRow".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 10.0,
                    y: 36.0,
                    width: 220.0,
                    height: 22.0,
                },
            },
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/parent_row".into(),
                control_id: "InspectorParentRow".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 10.0,
                    y: 62.0,
                    width: 220.0,
                    height: 22.0,
                },
            },
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/position_row".into(),
                control_id: "InspectorPositionRow".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 10.0,
                    y: 88.0,
                    width: 220.0,
                    height: 22.0,
                },
            },
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/separator_row".into(),
                control_id: "InspectorSeparatorRow".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 10.0,
                    y: 114.0,
                    width: 220.0,
                    height: 1.0,
                },
            },
            crate::ui::layouts::views::ViewTemplateNodeData {
                node_id: "root/actions_row".into(),
                control_id: "InspectorActionsRow".into(),
                role: "Mount".into(),
                text: "".into(),
                dispatch_kind: "".into(),
                action_id: "".into(),
                surface_variant: "".into(),
                text_tone: "".into(),
                button_variant: "".into(),
                font_size: 0.0,
                font_weight: 0,
                text_align: "left".into(),
                overflow: "".into(),
                corner_radius: 0.0,
                border_width: 0.0,
                frame: crate::ui::layouts::views::ViewTemplateFrameData {
                    x: 10.0,
                    y: 119.0,
                    width: 220.0,
                    height: 24.0,
                },
            },
        ]),
        info: "Node 42".into(),
        inspector_name: "CameraRig".into(),
        inspector_parent: "entity://root".into(),
        inspector_x: "1.0".into(),
        inspector_y: "2.0".into(),
        inspector_z: "3.0".into(),
        delete_enabled: true,
    };
    scene.bottom_dock.pane.body_compat.console = host_window::ConsolePaneViewData {
        nodes: model_rc(vec![crate::ui::layouts::views::ViewTemplateNodeData {
            node_id: "root/text_panel".into(),
            control_id: "ConsoleTextPanel".into(),
            role: "Mount".into(),
            text: "".into(),
            dispatch_kind: "".into(),
            action_id: "".into(),
            surface_variant: "".into(),
            text_tone: "".into(),
            button_variant: "".into(),
            font_size: 0.0,
            font_weight: 0,
            text_align: "left".into(),
            overflow: "".into(),
            corner_radius: 0.0,
            border_width: 0.0,
            frame: crate::ui::layouts::views::ViewTemplateFrameData {
                x: 10.0,
                y: 8.0,
                width: 640.0,
                height: 152.0,
            },
        }]),
        status_text: "Build finished".into(),
    };

    let projected = to_slint_host_scene_data(&scene);
    let floating_window = projected
        .floating_layer
        .floating_windows
        .row_data(0)
        .expect("floating window should project");

    assert_eq!(floating_window.active_pane.id, "floating-pane");
    assert_eq!(floating_window.active_pane.kind, "FloatingKind");
    assert_eq!(projected.left_dock.pane.id, "left-pane");
    assert_eq!(projected.left_dock.pane.title, "Left");
    assert_eq!(
        projected.left_dock.pane.ui_asset.header.asset_id,
        "asset://ui/test.ui.toml"
    );
    assert_eq!(projected.left_dock.pane.ui_asset.header.mode, "split");
    assert_eq!(projected.left_dock.pane.ui_asset.header.selection, "Root");
    let projected_ui_asset_nodes = (0..projected.left_dock.pane.ui_asset.nodes.row_count())
        .filter_map(|row| projected.left_dock.pane.ui_asset.nodes.row_data(row))
        .collect::<Vec<_>>();
    let projected_ui_asset_node = |control_id: &str| {
        projected_ui_asset_nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("projected ui asset node `{control_id}` should exist"))
    };
    assert_eq!(projected_ui_asset_node("HeaderPanel").frame.x, 11.0);
    assert_eq!(projected_ui_asset_node("HeaderPanel").frame.width, 640.0);
    assert_eq!(projected_ui_asset_node("HeaderAssetRow").frame.x, 21.0);
    assert_eq!(projected_ui_asset_node("HeaderStatusRow").frame.y, 28.0);
    assert_eq!(
        projected_ui_asset_node("HeaderActionRow").frame.height,
        20.0
    );
    assert_eq!(projected_ui_asset_node("PalettePanel").frame.height, 240.0);
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .center_column_node
            .control_id,
        "CenterColumn"
    );
    assert_eq!(
        projected.left_dock.pane.ui_asset.center_column_node.frame.x,
        260.0
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .designer_panel_node
            .frame
            .y,
        80.0
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .designer_canvas_panel_node
            .frame
            .height,
        214.0
    );
    assert_eq!(projected_ui_asset_node("RenderStackPanel").frame.y, 328.0);
    assert_eq!(projected_ui_asset_node("ActionBarPanel").frame.height, 88.0);
    assert_eq!(projected_ui_asset_node("ActionInsertRow").frame.x, 280.0);
    assert_eq!(projected_ui_asset_node("ActionReparentRow").frame.y, 450.0);
    assert_eq!(
        projected_ui_asset_node("ActionStructureRow").frame.width,
        380.0
    );
    assert_eq!(
        projected_ui_asset_node("SourceInfoPanel").frame.height,
        58.0
    );
    assert_eq!(
        projected_ui_asset_node("MockWorkspacePanel").frame.width,
        400.0
    );
    assert_eq!(
        projected_ui_asset_node("MockSubjectsPanel").frame.height,
        72.0
    );
    assert_eq!(projected_ui_asset_node("MockEditorPanel").frame.y, 606.0);
    assert_eq!(
        projected_ui_asset_node("MockStateGraphPanel").frame.y,
        782.0
    );
    assert_eq!(projected_ui_asset_node("SourceTextPanel").frame.y, 860.0);
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .inspector_panel_node
            .frame
            .height,
        240.0
    );
    assert_eq!(
        projected_ui_asset_node("InspectorContentPanel").frame.y,
        106.0
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .stylesheet_panel_node
            .frame
            .width,
        260.0
    );
    assert_eq!(
        projected_ui_asset_node("StylesheetActionRow").frame.y,
        356.0
    );
    assert_eq!(
        projected_ui_asset_node("StylesheetStatePrimaryRow")
            .frame
            .height,
        24.0
    );
    assert_eq!(
        projected_ui_asset_node("StylesheetStateSecondaryRow")
            .frame
            .x,
        710.0
    );
    assert_eq!(
        projected_ui_asset_node("StylesheetContentPanel")
            .frame
            .height,
        49.0
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .collections
            .palette
            .items
            .row_data(0)
            .expect("palette item should project"),
        "Button"
    );
    let projected_hierarchy_nodes = (0..projected.left_dock.pane.hierarchy.nodes.row_count())
        .filter_map(|row| projected.left_dock.pane.hierarchy.nodes.row_data(row))
        .collect::<Vec<_>>();
    assert_eq!(
        projected_hierarchy_nodes
            .iter()
            .find(|node| node.control_id == "HierarchyListPanel")
            .expect("hierarchy list panel node should project")
            .frame
            .x,
        8.0
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes
            .row_count(),
        2
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes
            .row_data(0)
            .expect("hierarchy node should project")
            .name,
        "Root"
    );
    assert_eq!(projected.document_dock.pane.id, "document-pane");
    assert_eq!(projected.document_dock.pane.title, "Document");
    assert_eq!(projected.right_dock.pane.id, "right-pane");
    assert_eq!(projected.right_dock.pane.title, "Right");
    assert_eq!(projected.right_dock.pane.inspector.info, "Node 42");
    let projected_inspector_nodes = (0..projected.right_dock.pane.inspector.nodes.row_count())
        .filter_map(|row| projected.right_dock.pane.inspector.nodes.row_data(row))
        .collect::<Vec<_>>();
    assert_eq!(
        projected_inspector_nodes
            .iter()
            .find(|node| node.control_id == "InspectorPositionRow")
            .expect("inspector position row should project")
            .frame
            .y,
        88.0
    );
    assert!(projected.right_dock.pane.inspector.delete_enabled);
    assert_eq!(projected.right_dock.pane.animation.mode, "sequence");
    assert_eq!(
        projected.right_dock.pane.animation.asset_path,
        "asset://animation/walk.anim"
    );
    let animation_nodes = &projected.right_dock.pane.animation.nodes;
    let animation_node = |control_id: &str| {
        (0..animation_nodes.row_count())
            .filter_map(|row| animation_nodes.row_data(row))
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("animation node `{control_id}` should project"))
    };
    assert_eq!(animation_node("AnimationEditorHeaderPanel").frame.x, 14.0);
    assert_eq!(
        animation_node("AnimationEditorHeaderStatusRow").frame.y,
        62.0
    );
    assert_eq!(
        animation_node("AnimationSequenceTracksPanel").frame.height,
        250.0
    );
    assert_eq!(projected.document_dock.pane.kind, "Project");
    assert_eq!(
        projected
            .document_dock
            .pane
            .project_overview
            .nodes
            .row_count(),
        3
    );
    assert_eq!(
        projected
            .document_dock
            .pane
            .project_overview
            .nodes
            .row_data(0)
            .expect("project overview node should project")
            .control_id,
        "ProjectOverviewOuterPanel"
    );
    assert_eq!(
        projected
            .document_dock
            .pane
            .project_overview
            .nodes
            .row_data(1)
            .expect("project overview path node should project")
            .text,
        "res://project"
    );
    assert_eq!(animation_node("AnimationGraphNodesPanel").frame.y, 234.0);
    assert_eq!(
        animation_node("AnimationStateMachineTransitionsPanel")
            .frame
            .height,
        146.0
    );
    assert_eq!(
        projected
            .right_dock
            .pane
            .animation
            .track_items
            .row_data(0)
            .expect("track item should project"),
        "Root Translation"
    );
    assert_eq!(projected.bottom_dock.pane.id, "bottom-pane");
    assert_eq!(projected.bottom_dock.pane.title, "Bottom");
    assert_eq!(projected.bottom_dock.pane.kind, "Assets");
    let projected_assets_nodes = (0..projected.bottom_dock.pane.assets_activity.nodes.row_count())
        .filter_map(|row| {
            projected
                .bottom_dock
                .pane
                .assets_activity
                .nodes
                .row_data(row)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        projected_assets_nodes
            .iter()
            .find(|node| node.control_id == "AssetsActivityToolbarPanel")
            .expect("toolbar node should project")
            .frame
            .x,
        18.0
    );
    assert_eq!(
        projected_assets_nodes
            .iter()
            .find(|node| node.control_id == "AssetsActivityTreePanel")
            .expect("tree node should project")
            .frame
            .width,
        248.0
    );
    assert_eq!(
        projected_assets_nodes
            .iter()
            .find(|node| node.control_id == "AssetsActivityUtilityTabsRow")
            .expect("utility tabs node should project")
            .frame
            .height,
        32.0
    );
    assert_eq!(
        projected_assets_nodes
            .iter()
            .find(|node| node.control_id == "AssetsActivityReferenceRightPanel")
            .expect("reference node should project")
            .frame
            .x,
        364.0
    );
    assert_eq!(
        projected.bottom_dock.pane.console.status_text,
        "Build finished"
    );
    let projected_console_nodes = (0..projected.bottom_dock.pane.console.nodes.row_count())
        .filter_map(|row| projected.bottom_dock.pane.console.nodes.row_data(row))
        .collect::<Vec<_>>();
    assert_eq!(
        projected_console_nodes
            .iter()
            .find(|node| node.control_id == "ConsoleBodySection")
            .expect("console body section node should project")
            .frame
            .height,
        152.0
    );
}

#[test]
fn apply_presentation_uses_shared_root_projection_frames_when_drawers_are_collapsed() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let projection_frames = bridge.root_shell_frames();
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            9.0, 19.0, 333.0, 444.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            15.0, 520.0, 640.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                crate::ui::workbench::autolayout::ShellFrame::new(21.0, 37.0, 410.0, 250.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
        ]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    let center_band = host_layout.center_band_frame;
    assert_eq!(center_band.x, 0.0);
    assert_eq!(center_band.y, 40.0);
    assert_eq!(center_band.width, 1280.0);
    assert_eq!(center_band.height, 656.0);

    let document_region = host_layout.document_region_frame;
    assert_eq!(document_region.x, 56.0);
    assert_eq!(document_region.y, 40.0);
    assert_eq!(document_region.width, 1224.0);
    assert_eq!(document_region.height, 656.0);

    let status_bar = host_layout.status_bar_frame;
    assert_eq!(status_bar.x, 0.0);
    assert_eq!(status_bar.y, 696.0);
    assert_eq!(status_bar.width, 1280.0);
    assert_eq!(status_bar.height, 24.0);

    let viewport_content = host_layout.viewport_content_frame;
    assert_eq!(viewport_content.x, 56.0);
    assert_eq!(viewport_content.y, 100.0);
    assert_eq!(viewport_content.width, 1224.0);
    assert_eq!(viewport_content.height, 596.0);
}

#[test]
fn apply_presentation_prefers_shared_root_projection_for_visible_drawer_document_region() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let projection_frames = bridge.root_shell_frames();
    let shell_frame = projection_frames
        .shell_frame
        .expect("root shell projection frame should exist");
    let body_frame = projection_frames
        .host_body_frame
        .expect("workbench body projection frame should exist");
    let metrics = crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default();
    let left_geometry =
        crate::ui::workbench::autolayout::ShellFrame::new(180.0, 91.0, 312.0, 440.0);
    let right_geometry =
        crate::ui::workbench::autolayout::ShellFrame::new(1024.0, 117.0, 256.0, 440.0);
    let bottom_geometry =
        crate::ui::workbench::autolayout::ShellFrame::new(48.0, 712.0, 1232.0, 180.0);
    let expected_document_frame = crate::ui::workbench::autolayout::ShellFrame::new(
        shell_frame.x + left_geometry.width + metrics.separator_thickness,
        body_frame.y,
        body_frame.width
            - left_geometry.width
            - right_geometry.width
            - metrics.separator_thickness * 2.0,
        body_frame.height - bottom_geometry.height - metrics.separator_thickness,
    );
    let geometry_document_frame =
        crate::ui::workbench::autolayout::ShellFrame::new(734.0, 91.0, 222.0, 109.0);
    let document_chrome_height = metrics.document_header_height + metrics.separator_thickness;
    let expected_viewport_frame = crate::ui::workbench::autolayout::ShellFrame::new(
        expected_document_frame.x,
        expected_document_frame.y + document_chrome_height + metrics.viewport_toolbar_height,
        expected_document_frame.width,
        expected_document_frame.height - document_chrome_height - metrics.viewport_toolbar_height,
    );
    let geometry_viewport_frame =
        crate::ui::workbench::autolayout::ShellFrame::new(888.0, 144.0, 155.0, 77.0);
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            11.0, 520.0, 700.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                left_geometry,
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                geometry_document_frame,
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                right_geometry,
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                bottom_geometry,
            ),
        ]
        .into_iter()
        .collect(),
        viewport_content_frame: geometry_viewport_frame,
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    let center_band = host_layout.center_band_frame;
    assert_eq!(center_band.x, 0.0);
    assert_eq!(center_band.y, 40.0);
    assert_eq!(center_band.width, 1280.0);
    assert_eq!(center_band.height, 656.0);

    let document_region = host_layout.document_region_frame;
    assert_eq!(document_region.x, expected_document_frame.x);
    assert_eq!(document_region.y, expected_document_frame.y);
    assert_eq!(document_region.width, expected_document_frame.width);
    assert_eq!(document_region.height, expected_document_frame.height);

    let status_bar = host_layout.status_bar_frame;
    assert_eq!(status_bar.x, 0.0);
    assert_eq!(status_bar.y, 696.0);
    assert_eq!(status_bar.width, 1280.0);
    assert_eq!(status_bar.height, 24.0);

    let viewport_content = host_layout.viewport_content_frame;
    assert_eq!(viewport_content.x, expected_viewport_frame.x);
    assert_eq!(viewport_content.y, expected_viewport_frame.y);
    assert_eq!(viewport_content.width, expected_viewport_frame.width);
    assert_eq!(viewport_content.height, expected_viewport_frame.height);
}

#[test]
fn apply_presentation_prefers_drawer_derived_viewport_when_pane_surface_is_stale() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let mut projection_frames = bridge.root_shell_frames();
    projection_frames.left_drawer_shell_frame = Some(UiFrame::new(56.0, 40.0, 312.0, 480.0));
    projection_frames.pane_surface_frame = Some(UiFrame::new(369.0, 100.0, 602.0, 420.0));

    let metrics = crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default();
    let shell_frame = projection_frames
        .shell_frame
        .expect("root shell projection frame should exist");
    let body_frame = projection_frames
        .host_body_frame
        .expect("workbench body projection frame should exist");
    let expected_document_frame = crate::ui::workbench::autolayout::ShellFrame::new(
        shell_frame.x + 312.0 + metrics.separator_thickness,
        body_frame.y,
        body_frame.width - 312.0 - metrics.separator_thickness,
        body_frame.height,
    );
    let document_chrome_height = metrics.document_header_height + metrics.separator_thickness;
    let expected_viewport_frame = crate::ui::workbench::autolayout::ShellFrame::new(
        expected_document_frame.x,
        expected_document_frame.y + document_chrome_height + metrics.viewport_toolbar_height,
        expected_document_frame.width,
        expected_document_frame.height - document_chrome_height - metrics.viewport_toolbar_height,
    );
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        region_frames: [(
            crate::ui::workbench::autolayout::ShellRegionId::Document,
            crate::ui::workbench::autolayout::ShellFrame::new(734.0, 91.0, 222.0, 109.0),
        )]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let viewport_content = ui
        .get_host_presentation()
        .host_layout
        .viewport_content_frame;
    assert_eq!(viewport_content.x, expected_viewport_frame.x);
    assert_eq!(viewport_content.y, expected_viewport_frame.y);
    assert_eq!(viewport_content.width, expected_viewport_frame.width);
    assert_eq!(viewport_content.height, expected_viewport_frame.height);
}

#[test]
fn apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_positions() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let projection_frames = bridge.root_shell_frames();
    let shell_frame = projection_frames
        .shell_frame
        .expect("root shell projection frame should exist");
    let body_frame = projection_frames
        .host_body_frame
        .expect("workbench body projection frame should exist");
    let metrics = crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default();
    let left_geometry =
        crate::ui::workbench::autolayout::ShellFrame::new(180.0, 91.0, 312.0, 519.0);
    let right_geometry =
        crate::ui::workbench::autolayout::ShellFrame::new(1024.0, 117.0, 256.0, 401.0);
    let bottom_geometry =
        crate::ui::workbench::autolayout::ShellFrame::new(48.0, 712.0, 777.0, 180.0);
    let expected_center_height =
        body_frame.height - bottom_geometry.height - metrics.separator_thickness;
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            11.0, 520.0, 700.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                left_geometry,
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                crate::ui::workbench::autolayout::ShellFrame::new(493.0, 91.0, 531.0, 440.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                right_geometry,
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                bottom_geometry,
            ),
        ]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    let left_region = host_layout.left_region_frame;
    assert_eq!(left_region.x, shell_frame.x);
    assert_eq!(left_region.y, body_frame.y);
    assert_eq!(left_region.width, left_geometry.width);
    assert_eq!(left_region.height, expected_center_height);

    let right_region = host_layout.right_region_frame;
    assert_eq!(
        right_region.x,
        shell_frame.x + shell_frame.width - right_geometry.width
    );
    assert_eq!(right_region.y, body_frame.y);
    assert_eq!(right_region.width, right_geometry.width);
    assert_eq!(right_region.height, expected_center_height);

    let bottom_region = host_layout.bottom_region_frame;
    assert_eq!(bottom_region.x, shell_frame.x);
    assert_eq!(
        bottom_region.y,
        body_frame.y + body_frame.height - bottom_geometry.height
    );
    assert_eq!(bottom_region.width, body_frame.width);
    assert_eq!(bottom_region.height, bottom_geometry.height);
}

#[test]
fn apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_extents() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        )
        .unwrap();
    let projection_frames = bridge.root_shell_frames();
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            11.0, 520.0, 700.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                crate::ui::workbench::autolayout::ShellFrame::new(180.0, 91.0, 180.0, 519.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                crate::ui::workbench::autolayout::ShellFrame::new(493.0, 91.0, 531.0, 440.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                crate::ui::workbench::autolayout::ShellFrame::new(1024.0, 117.0, 144.0, 401.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                crate::ui::workbench::autolayout::ShellFrame::new(48.0, 712.0, 777.0, 120.0),
            ),
        ]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    assert_eq!(
        host_layout.left_region_frame,
        crate::ui::slint_host::FrameRect {
            x: projection_frames.left_drawer_shell_frame.unwrap().x,
            y: projection_frames.left_drawer_shell_frame.unwrap().y,
            width: projection_frames.left_drawer_shell_frame.unwrap().width,
            height: projection_frames.left_drawer_shell_frame.unwrap().height,
        }
    );
    assert_eq!(
        host_layout.right_region_frame,
        crate::ui::slint_host::FrameRect {
            x: projection_frames.right_drawer_shell_frame.unwrap().x,
            y: projection_frames.right_drawer_shell_frame.unwrap().y,
            width: projection_frames.right_drawer_shell_frame.unwrap().width,
            height: projection_frames.right_drawer_shell_frame.unwrap().height,
        }
    );
    assert_eq!(
        host_layout.bottom_region_frame,
        crate::ui::slint_host::FrameRect {
            x: projection_frames.bottom_drawer_shell_frame.unwrap().x,
            y: projection_frames.bottom_drawer_shell_frame.unwrap().y,
            width: projection_frames.bottom_drawer_shell_frame.unwrap().width,
            height: projection_frames.bottom_drawer_shell_frame.unwrap().height,
        }
    );
}

#[test]
fn apply_presentation_prefers_shared_visible_drawer_projection_when_legacy_geometry_is_zeroed() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        )
        .unwrap();
    let projection_frames = bridge.root_shell_frames();
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            11.0, 520.0, 700.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                crate::ui::workbench::autolayout::ShellFrame::new(21.0, 37.0, 410.0, 250.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
        ]
        .into_iter()
        .collect(),
        viewport_content_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            66.0, 120.0, 320.0, 180.0,
        ),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    assert_eq!(
        host_layout.left_region_frame,
        crate::ui::slint_host::FrameRect {
            x: projection_frames.left_drawer_shell_frame.unwrap().x,
            y: projection_frames.left_drawer_shell_frame.unwrap().y,
            width: projection_frames.left_drawer_shell_frame.unwrap().width,
            height: projection_frames.left_drawer_shell_frame.unwrap().height,
        }
    );
    assert_eq!(
        host_layout.right_region_frame,
        crate::ui::slint_host::FrameRect {
            x: projection_frames.right_drawer_shell_frame.unwrap().x,
            y: projection_frames.right_drawer_shell_frame.unwrap().y,
            width: projection_frames.right_drawer_shell_frame.unwrap().width,
            height: projection_frames.right_drawer_shell_frame.unwrap().height,
        }
    );
    assert_eq!(
        host_layout.bottom_region_frame,
        crate::ui::slint_host::FrameRect {
            x: projection_frames.bottom_drawer_shell_frame.unwrap().x,
            y: projection_frames.bottom_drawer_shell_frame.unwrap().y,
            width: projection_frames.bottom_drawer_shell_frame.unwrap().width,
            height: projection_frames.bottom_drawer_shell_frame.unwrap().height,
        }
    );
    assert_eq!(
        host_layout.document_region_frame,
        crate::ui::slint_host::FrameRect {
            x: 313.0,
            y: 40.0,
            width: 658.0,
            height: 491.0,
        }
    );
    assert_eq!(
        host_layout.viewport_content_frame,
        crate::ui::slint_host::FrameRect {
            x: 313.0,
            y: 68.0,
            width: 658.0,
            height: 463.0,
        }
    );
}

#[test]
fn apply_presentation_projects_welcome_mount_nodes_into_global_context() {
    i_slint_backend_testing::init_no_event_loop();

    let (chrome, model, ui_asset_panes, animation_panes) = welcome_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let projection_frames = bridge.root_shell_frames();
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            9.0, 19.0, 333.0, 444.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            15.0, 520.0, 640.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                crate::ui::workbench::autolayout::ShellFrame::new(21.0, 37.0, 410.0, 250.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
        ]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let expected_nodes = crate::ui::layouts::views::welcome_pane_nodes(UiSize::new(1224.0, 624.0));
    let projected = ui
        .global::<crate::ui::slint_host::PaneSurfaceHostContext>()
        .get_welcome_pane();
    let expected_nodes = (0..expected_nodes.row_count())
        .filter_map(|row| expected_nodes.row_data(row))
        .collect::<Vec<_>>();
    let projected_nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();

    assert_eq!(projected.title, "Open or Create");
    assert_eq!(projected_nodes.len(), expected_nodes.len());

    for control_id in [
        "WelcomeOuterPanel",
        "WelcomeRecentPanel",
        "WelcomeMainPanel",
        "WelcomePreviewPanel",
        "WelcomeActionsRow",
    ] {
        let expected = expected_nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .expect("expected welcome node");
        let actual = projected_nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .expect("projected welcome node");
        assert_eq!(actual.role.to_string(), expected.role.to_string());
        assert_eq!(actual.frame.x, expected.frame.x);
        assert_eq!(actual.frame.y, expected.frame.y);
        assert_eq!(actual.frame.width, expected.frame.width);
        assert_eq!(actual.frame.height, expected.frame.height);
    }
}

#[test]
fn scene_document_pane_uses_viewport_dimensions_and_enables_toolbar() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let ui_asset_panes = BTreeMap::new();

    let pane = document_pane(&model, &chrome, &ui_asset_panes, &BTreeMap::new(), None);

    assert_eq!(pane.kind, "Scene");
    assert_eq!(pane.subtitle, "1280 x 720");
    assert!(pane.show_toolbar);
}

#[test]
fn scene_document_pane_projects_viewport_toolbar_state() {
    let mut fixture = default_preview_fixture();
    fixture.editor.scene_viewport_settings.tool = SceneViewportTool::Scale;
    fixture.editor.scene_viewport_settings.transform_space = TransformSpace::Global;
    fixture.editor.scene_viewport_settings.projection_mode = ProjectionMode::Orthographic;
    fixture.editor.scene_viewport_settings.view_orientation = ViewOrientation::NegZ;
    fixture.editor.scene_viewport_settings.display_mode = DisplayMode::WireOverlay;
    fixture.editor.scene_viewport_settings.grid_mode = GridMode::VisibleAndSnap;
    fixture.editor.scene_viewport_settings.gizmos_enabled = false;
    fixture.editor.scene_viewport_settings.preview_lighting = false;
    fixture.editor.scene_viewport_settings.preview_skybox = false;
    fixture.editor.scene_viewport_settings.translate_step = 2.5;
    fixture.editor.scene_viewport_settings.rotate_step_deg = 30.0;
    fixture.editor.scene_viewport_settings.scale_step = 0.25;

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let ui_asset_panes = BTreeMap::new();
    let pane = document_pane(&model, &chrome, &ui_asset_panes, &BTreeMap::new(), None);

    assert_eq!(pane.kind, "Scene");
    assert_eq!(pane.viewport.tool, "Scale");
    assert_eq!(pane.viewport.transform_space, "Global");
    assert_eq!(pane.viewport.projection_mode, "Orthographic");
    assert_eq!(pane.viewport.view_orientation, "NegZ");
    assert_eq!(pane.viewport.display_mode, "WireOverlay");
    assert_eq!(pane.viewport.grid_mode, "VisibleAndSnap");
    assert!(!pane.viewport.gizmos_enabled);
    assert!(!pane.viewport.preview_lighting);
    assert!(!pane.viewport.preview_skybox);
    assert_eq!(pane.viewport.translate_snap, 2.5);
    assert_eq!(pane.viewport.rotate_snap_deg, 30.0);
    assert_eq!(pane.viewport.scale_snap, 0.25);
    assert_eq!(pane.viewport.translate_snap_label, "T 2.5");
    assert_eq!(pane.viewport.rotate_snap_label, "R 30");
    assert_eq!(pane.viewport.scale_snap_label, "S 0.25");
}

#[test]
fn floating_windows_project_tabs_and_active_pane_for_host_presentation() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    let game_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.game#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.game"),
        title: "Game".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.play" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.instances.push(game_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![
                scene_instance.instance_id.clone(),
                game_instance.instance_id.clone(),
            ],
            active_tab: Some(game_instance.instance_id.clone()),
        }),
        focused_view: Some(game_instance.instance_id.clone()),
        frame: crate::ui::workbench::autolayout::ShellFrame::default(),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &WorkbenchShellGeometry::default(),
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &WorkbenchShellGeometry::default(),
        &ui_asset_panes,
        &BTreeMap::new(),
        None,
        &floating_window_projection_bundle,
    );

    assert_eq!(floating_windows.len(), 1);
    assert_eq!(floating_windows[0].window_id, "window:preview");
    assert_eq!(floating_windows[0].title, "Preview Popout");
    assert_eq!(
        floating_windows[0]
            .tabs
            .iter()
            .map(|tab| (tab.title.to_string(), tab.active))
            .collect::<Vec<_>>(),
        vec![("Scene".to_string(), false), ("Game".to_string(), true)]
    );
    assert_eq!(floating_windows[0].focus_target_id, "editor.game#float");
    assert_eq!(floating_windows[0].active_pane.title, "Game");
    assert_eq!(floating_windows[0].active_pane.kind, "Game");
}

#[test]
fn floating_windows_ignore_stale_focused_view_when_projecting_focus_target() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    let game_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.game#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.game"),
        title: "Game".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.play" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.instances.push(game_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![
                scene_instance.instance_id.clone(),
                game_instance.instance_id.clone(),
            ],
            active_tab: Some(game_instance.instance_id.clone()),
        }),
        focused_view: Some(ViewInstanceId::new("editor.missing#1")),
        frame: crate::ui::workbench::autolayout::ShellFrame::default(),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &WorkbenchShellGeometry::default(),
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &WorkbenchShellGeometry::default(),
        &ui_asset_panes,
        &BTreeMap::new(),
        None,
        &floating_window_projection_bundle,
    );

    assert_eq!(floating_windows.len(), 1);
    assert_eq!(floating_windows[0].focus_target_id, "editor.game#float");
    assert_eq!(floating_windows[0].active_pane.id, "editor.game#float");
    assert_eq!(floating_windows[0].active_pane.kind, "Game");
}

#[test]
fn floating_window_overlay_snapshot_captures_shared_frame_and_route_keys() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    let game_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.game#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.game"),
        title: "Game".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.play" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.instances.push(game_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![
                scene_instance.instance_id.clone(),
                game_instance.instance_id.clone(),
            ],
            active_tab: Some(game_instance.instance_id.clone()),
        }),
        focused_view: Some(game_instance.instance_id.clone()),
        frame: crate::ui::workbench::autolayout::ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = crate::ui::workbench::autolayout::compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        crate::ui::workbench::autolayout::ShellSizePx::new(1440.0, 900.0),
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        None,
    );
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &geometry,
        &ui_asset_panes,
        &BTreeMap::new(),
        None,
        &floating_window_projection_bundle,
    );

    let snapshot =
        EditorUiCompatibilityHarness::capture_floating_window_overlay_snapshot(&floating_windows);

    assert!(snapshot
        .frame_entries
        .contains(&"floating-window/window:preview=420,180,360,240".to_string()));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.attach=floating-window/window:preview".to_string()
    ));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.left=floating-window-edge/window:preview/left".to_string()
    ));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.right=floating-window-edge/window:preview/right"
            .to_string()
    ));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.top=floating-window-edge/window:preview/top".to_string()
    ));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.bottom=floating-window-edge/window:preview/bottom"
            .to_string()
    ));
    assert!(snapshot
        .attribute_entries
        .contains(&"floating-window/window:preview.title=Preview Popout".to_string()));
    assert!(snapshot
        .attribute_entries
        .contains(&"floating-window/window:preview.focus_target_id=editor.game#float".to_string()));
    assert!(snapshot
        .attribute_entries
        .contains(&"floating-window/window:preview.active_pane.id=editor.game#float".to_string()));
    assert!(snapshot
        .attribute_entries
        .contains(&"floating-window/window:preview.active_pane.kind=Game".to_string()));
}

#[test]
fn floating_window_overlay_route_keys_match_shared_shell_pointer_route_normalization() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![scene_instance.instance_id.clone()],
            active_tab: Some(scene_instance.instance_id.clone()),
        }),
        focused_view: Some(scene_instance.instance_id.clone()),
        frame: crate::ui::workbench::autolayout::ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = crate::ui::workbench::autolayout::compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        crate::ui::workbench::autolayout::ShellSizePx::new(1440.0, 900.0),
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        None,
    );
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &geometry,
        &ui_asset_panes,
        &BTreeMap::new(),
        None,
        &floating_window_projection_bundle,
    );
    let window = &floating_windows[0];

    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindow(
            window_id.clone()
        )),
        Some(window.target_group.to_string())
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: DockEdge::Left,
        }),
        Some(window.left_edge_target_group.to_string())
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: DockEdge::Right,
        }),
        Some(window.right_edge_target_group.to_string())
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: DockEdge::Top,
        }),
        Some(window.top_edge_target_group.to_string())
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindowEdge {
            window_id,
            edge: DockEdge::Bottom,
        }),
        Some(window.bottom_edge_target_group.to_string())
    );
}

#[test]
fn collect_floating_windows_does_not_fall_back_to_legacy_geometry_when_projection_bundle_is_explicitly_provided(
) {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![scene_instance.instance_id.clone()],
            active_tab: Some(scene_instance.instance_id.clone()),
        }),
        focused_view: Some(scene_instance.instance_id.clone()),
        frame: crate::ui::workbench::autolayout::ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let mut geometry = crate::ui::workbench::autolayout::compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        crate::ui::workbench::autolayout::ShellSizePx::new(1440.0, 900.0),
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        None,
    );
    geometry.floating_window_frames.insert(
        window_id.clone(),
        crate::ui::workbench::autolayout::ShellFrame::new(96.0, 72.0, 144.0, 88.0),
    );

    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &geometry,
        &BTreeMap::new(),
        &BTreeMap::new(),
        None,
        &FloatingWindowProjectionBundle::default(),
    );

    assert_eq!(floating_windows.len(), 1);
    assert_eq!(floating_windows[0].frame.x, 0.0);
    assert_eq!(floating_windows[0].frame.y, 0.0);
    assert_eq!(floating_windows[0].frame.width, 0.0);
    assert_eq!(floating_windows[0].frame.height, 0.0);
}
