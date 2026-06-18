use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::workbench::model::StatusBarModel;
use crate::ui::workbench::snapshot::{
    EditorChromeSnapshot, StatusTaskProgressSnapshot, StatusTaskProgressTone,
};

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const STATUS_READY: &str = "WorkbenchStatusReady";
const STATUS_ERRORS: &str = "WorkbenchStatusErrors";
const STATUS_WARNINGS: &str = "WorkbenchStatusWarnings";
const STATUS_MESSAGES: &str = "WorkbenchStatusMessages";
const STATUS_GRID: &str = "WorkbenchStatusGrid";
const STATUS_SNAP: &str = "WorkbenchStatusSnap";
const STATUS_ZOOM: &str = "WorkbenchStatusZoom";
const STATUS_TASK_PROGRESS: &str = "WorkbenchStatusTaskProgress";
const STATUS_TASK_LABEL: &str = "WorkbenchStatusTaskLabel";
const STATUS_TASK_BAR: &str = "WorkbenchStatusTaskBar";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn sync_status_bar(
        &mut self,
        chrome: &EditorChromeSnapshot,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let model = StatusBarModel::from_chrome(chrome);
        self.mutate_control_property(STATUS_READY, "text", UiValue::String(model.primary_text))?;
        self.mutate_control_property(STATUS_ERRORS, "text", UiValue::String(model.error_text))?;
        self.mutate_control_property(STATUS_WARNINGS, "text", UiValue::String(model.warning_text))?;
        self.mutate_control_property(STATUS_MESSAGES, "text", UiValue::String(model.message_text))?;
        self.mutate_control_property(STATUS_GRID, "text", UiValue::String(model.grid_text))?;
        self.mutate_control_property(STATUS_SNAP, "text", UiValue::String(model.snap_text))?;
        self.mutate_control_property(STATUS_ZOOM, "text", UiValue::String(model.zoom_text))?;
        self.sync_status_task_progress(model.task_progress.as_ref())?;
        Ok(())
    }

    fn sync_status_task_progress(
        &mut self,
        task: Option<&StatusTaskProgressSnapshot>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(task) = task else {
            self.set_visible(STATUS_TASK_PROGRESS, false)?;
            self.sync_task_control_values("", "", 0.0, "linear", "#2099a2", "info")?;
            return Ok(());
        };

        let percent = task.percent.unwrap_or(0).min(100);
        let variant = if task.percent.is_some() {
            "linear"
        } else {
            "linear-indeterminate"
        };
        let tone = task_tone(task.tone);
        let fill_color = task_fill_color(task.tone);
        self.set_visible(STATUS_TASK_PROGRESS, true)?;
        self.sync_task_control_values(
            &task_display_text(task),
            &task.detail,
            percent as f64,
            variant,
            fill_color,
            tone,
        )?;
        Ok(())
    }

    fn sync_task_control_values(
        &mut self,
        text: &str,
        detail: &str,
        value: f64,
        variant: &str,
        fill_color: &str,
        tone: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for control_id in [STATUS_TASK_PROGRESS, STATUS_TASK_LABEL] {
            self.mutate_control_property(control_id, "text", UiValue::String(text.to_string()))?;
            self.mutate_control_property(
                control_id,
                "value_text",
                UiValue::String(detail.to_string()),
            )?;
            self.mutate_control_property(
                control_id,
                "text_tone",
                UiValue::String(tone.to_string()),
            )?;
        }
        for control_id in [STATUS_TASK_PROGRESS, STATUS_TASK_BAR] {
            self.mutate_control_property(control_id, "min", UiValue::Float(0.0))?;
            self.mutate_control_property(control_id, "max", UiValue::Float(100.0))?;
            self.mutate_control_property(control_id, "value", UiValue::Float(value))?;
            self.mutate_control_property(
                control_id,
                "variant",
                UiValue::String(variant.to_string()),
            )?;
            self.mutate_control_property(
                control_id,
                "track_fill_color",
                UiValue::String(fill_color.to_string()),
            )?;
            self.mutate_control_property(
                control_id,
                "value_color",
                UiValue::String(fill_color.to_string()),
            )?;
            self.mutate_control_property(
                control_id,
                "text_tone",
                UiValue::String(tone.to_string()),
            )?;
        }
        Ok(())
    }
}

fn task_display_text(task: &StatusTaskProgressSnapshot) -> String {
    match task.percent {
        Some(percent) => format!("{} {}%", task.label, percent.min(100)),
        None => task.label.clone(),
    }
}

fn task_tone(tone: StatusTaskProgressTone) -> &'static str {
    match tone {
        StatusTaskProgressTone::Info => "info",
        StatusTaskProgressTone::Success => "success",
        StatusTaskProgressTone::Warning => "warning",
        StatusTaskProgressTone::Error => "error",
    }
}

fn task_fill_color(tone: StatusTaskProgressTone) -> &'static str {
    match tone {
        StatusTaskProgressTone::Info => "#2099a2",
        StatusTaskProgressTone::Success => "#58b866",
        StatusTaskProgressTone::Warning => "#f2c356",
        StatusTaskProgressTone::Error => "#e35d5b",
    }
}
