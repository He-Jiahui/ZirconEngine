use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use zircon_runtime_interface::ui::binding::{UiBindingCall, UiBindingValue};

pub(super) fn material_lab_template_bindings() -> Vec<(String, EditorUiBinding)> {
    let mut bindings = MATERIAL_LAB_BINDING_SPECS
        .iter()
        .map(|spec| material_lab_binding_entry(spec.binding_id, spec.event_kind))
        .collect::<Vec<_>>();
    bindings.extend(
        MATERIAL_LAB_STRUCTURAL_CHILD_BINDING_SPECS
            .iter()
            .map(|spec| {
                material_lab_structural_child_binding_entry(
                    spec.binding_id,
                    spec.control_id,
                    spec.event_kind,
                )
            }),
    );
    bindings
}

#[derive(Clone, Copy)]
struct MaterialLabBindingSpec {
    binding_id: &'static str,
    event_kind: EditorUiEventKind,
}

const MATERIAL_LAB_BINDING_SPECS: &[MaterialLabBindingSpec] = &[
    material_lab_binding_spec("MaterialLab/Accordion/Toggle", EditorUiEventKind::Toggle),
    material_lab_binding_spec("MaterialLab/Autocomplete/Change", EditorUiEventKind::Change),
    material_lab_binding_spec("MaterialLab/Backdrop/Click", EditorUiEventKind::Click),
    material_lab_binding_spec(
        "MaterialLab/BottomNavigation/Change",
        EditorUiEventKind::Change,
    ),
    material_lab_binding_spec("MaterialLab/Breadcrumbs/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Buttons/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Cards/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Checkboxes/Toggle", EditorUiEventKind::Toggle),
    material_lab_binding_spec("MaterialLab/Chips/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Dialogs/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Drawers/Click", EditorUiEventKind::Click),
    material_lab_binding_spec(
        "MaterialLab/FloatingActionButton/Click",
        EditorUiEventKind::Click,
    ),
    material_lab_binding_spec("MaterialLab/ImageList/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Links/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Lists/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Menubar/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Menus/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Modal/Click", EditorUiEventKind::Click),
    material_lab_binding_spec(
        "MaterialLab/MuiXAgentChat/Submit",
        EditorUiEventKind::Submit,
    ),
    material_lab_binding_spec("MaterialLab/MuiXBarChart/Hover", EditorUiEventKind::Hover),
    material_lab_binding_spec("MaterialLab/MuiXCharts/Hover", EditorUiEventKind::Hover),
    material_lab_binding_spec(
        "MaterialLab/MuiXChatComposer/Submit",
        EditorUiEventKind::Submit,
    ),
    material_lab_binding_spec("MaterialLab/MuiXDataGrid/Click", EditorUiEventKind::Click),
    material_lab_binding_spec(
        "MaterialLab/MuiXDateTimePickers/Submit",
        EditorUiEventKind::Submit,
    ),
    material_lab_binding_spec("MaterialLab/MuiXGauge/Hover", EditorUiEventKind::Hover),
    material_lab_binding_spec("MaterialLab/MuiXLineChart/Hover", EditorUiEventKind::Hover),
    material_lab_binding_spec("MaterialLab/MuiXPieChart/Hover", EditorUiEventKind::Hover),
    material_lab_binding_spec("MaterialLab/MuiXSparkline/Hover", EditorUiEventKind::Hover),
    material_lab_binding_spec("MaterialLab/MuiXTreeView/Toggle", EditorUiEventKind::Toggle),
    material_lab_binding_spec("MaterialLab/NumberField/Change", EditorUiEventKind::Change),
    material_lab_binding_spec("MaterialLab/Pagination/Change", EditorUiEventKind::Change),
    material_lab_binding_spec("MaterialLab/Popover/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Popper/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/RadioButtons/Change", EditorUiEventKind::Change),
    material_lab_binding_spec("MaterialLab/Rating/Change", EditorUiEventKind::Change),
    material_lab_binding_spec("MaterialLab/Selects/Change", EditorUiEventKind::Change),
    material_lab_binding_spec(
        "MaterialLab/Slider/DragUpdate",
        EditorUiEventKind::DragUpdate,
    ),
    material_lab_binding_spec("MaterialLab/Snackbars/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/SpeedDial/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Steppers/Change", EditorUiEventKind::Change),
    material_lab_binding_spec("MaterialLab/Switches/Toggle", EditorUiEventKind::Toggle),
    material_lab_binding_spec("MaterialLab/Table/Click", EditorUiEventKind::Click),
    material_lab_binding_spec("MaterialLab/Tabs/Change", EditorUiEventKind::Change),
    material_lab_binding_spec("MaterialLab/TextFields/Change", EditorUiEventKind::Change),
    material_lab_binding_spec(
        "MaterialLab/TextareaAutosize/Change",
        EditorUiEventKind::Change,
    ),
    material_lab_binding_spec("MaterialLab/ToggleButton/Toggle", EditorUiEventKind::Toggle),
    material_lab_binding_spec("MaterialLab/Tooltips/Hover", EditorUiEventKind::Hover),
    material_lab_binding_spec("MaterialLab/TransferList/Change", EditorUiEventKind::Change),
];

#[derive(Clone, Copy)]
struct MaterialLabStructuralChildBindingSpec {
    binding_id: &'static str,
    control_id: &'static str,
    event_kind: EditorUiEventKind,
}

const MATERIAL_LAB_STRUCTURAL_CHILD_BINDING_SPECS: &[MaterialLabStructuralChildBindingSpec] = &[
    material_lab_structural_child_binding_spec(
        "ButtonGroup/First/Click",
        "button-group_SegmentFirst",
        EditorUiEventKind::Click,
    ),
    material_lab_structural_child_binding_spec(
        "ButtonGroup/Middle/Click",
        "button-group_SegmentMiddle",
        EditorUiEventKind::Click,
    ),
    material_lab_structural_child_binding_spec(
        "ButtonGroup/Last/Click",
        "button-group_SegmentLast",
        EditorUiEventKind::Click,
    ),
];

const fn material_lab_binding_spec(
    binding_id: &'static str,
    event_kind: EditorUiEventKind,
) -> MaterialLabBindingSpec {
    MaterialLabBindingSpec {
        binding_id,
        event_kind,
    }
}

const fn material_lab_structural_child_binding_spec(
    binding_id: &'static str,
    control_id: &'static str,
    event_kind: EditorUiEventKind,
) -> MaterialLabStructuralChildBindingSpec {
    MaterialLabStructuralChildBindingSpec {
        binding_id,
        control_id,
        event_kind,
    }
}

fn material_lab_binding_entry(
    binding_id: &str,
    event_kind: EditorUiEventKind,
) -> (String, EditorUiBinding) {
    let action = binding_id.replace('/', ".");
    let control_id = material_lab_control_id(binding_id);
    (
        binding_id.to_string(),
        EditorUiBinding::new(
            "MaterialComponentLab",
            control_id.clone(),
            event_kind,
            EditorUiBindingPayload::Custom(
                UiBindingCall::new("MaterialComponentLab")
                    .with_argument(UiBindingValue::string(action))
                    .with_argument(UiBindingValue::string(control_id)),
            ),
        ),
    )
}

fn material_lab_structural_child_binding_entry(
    binding_id: &str,
    control_id: &str,
    event_kind: EditorUiEventKind,
) -> (String, EditorUiBinding) {
    let action = binding_id.replace('/', ".");
    (
        binding_id.to_string(),
        EditorUiBinding::new(
            "MaterialComponentLab",
            control_id,
            event_kind,
            EditorUiBindingPayload::Custom(
                UiBindingCall::new("MaterialComponentLab")
                    .with_argument(UiBindingValue::string(action))
                    .with_argument(UiBindingValue::string(control_id)),
            ),
        ),
    )
}

fn material_lab_control_id(binding_id: &str) -> String {
    binding_id
        .strip_prefix("MaterialLab/")
        .and_then(|tail| tail.rsplit_once('/').map(|(component, _)| component))
        .map(|component| format!("MaterialLab{component}"))
        .unwrap_or_else(|| "MaterialLabPrototype".to_string())
}
