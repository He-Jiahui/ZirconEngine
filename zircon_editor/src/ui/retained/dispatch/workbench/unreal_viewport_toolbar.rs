use zircon_runtime_interface::ui::component::UiValue;

use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::{
    DisplayMode, PivotMode, SceneViewportChromeSettings, TransformHandleKind, TransformSpace,
    ViewOrientation,
};

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    // Paint, hit testing and Penpot publication read this actual mounted source
    // tree. Updating a separate legacy toolbar paint cache would leave it stale.
    pub(super) fn sync_unreal_viewport_toolbar_state(
        &mut self,
        settings: &SceneViewportChromeSettings,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for (control_id, mode) in [
            ("SelectMode", SceneModeActivation::Select),
            ("TranslateMode", SceneModeActivation::Transform(TransformHandleKind::Move)),
            ("RotateMode", SceneModeActivation::Transform(TransformHandleKind::Rotate)),
            ("ScaleMode", SceneModeActivation::Transform(TransformHandleKind::Scale)),
        ] {
            self.set_control_active(control_id, settings.mode == mode)?;
        }
        self.set_unreal_viewport_dropdown("SetProjectionMode", format!("{:?}", settings.projection_mode),
            format!("{:?}", settings.projection_mode))?;
        self.set_unreal_viewport_dropdown("SetDisplayMode", format!("{:?}", settings.display_mode),
            match settings.display_mode {
                DisplayMode::Shaded => "Lit",
                DisplayMode::WireOverlay => "Wireframe Overlay",
                DisplayMode::WireOnly => "Wireframe",
            })?;
        self.set_unreal_viewport_dropdown("SetGridMode", if settings.grid_mode == crate::scene::viewport::GridMode::Hidden { "Hidden" } else { "VisibleNoSnap" }, "Show")?;
        self.set_unreal_viewport_dropdown("SetTransformSpace", format!("{:?}", settings.transform_space),
            match settings.transform_space { TransformSpace::Local => "Local", TransformSpace::Global => "World" })?;
        self.set_unreal_viewport_dropdown("SetPivotMode", format!("{:?}", settings.pivot_mode),
            match settings.pivot_mode { PivotMode::Primary => "Pivot", PivotMode::Centroid => "Centroid" })?;
        self.set_unreal_viewport_dropdown("AlignView", format!("{:?}", settings.view_orientation),
            match settings.view_orientation {
                ViewOrientation::User => "Align", ViewOrientation::PosX => "+X", ViewOrientation::NegX => "-X",
                ViewOrientation::PosY => "+Y", ViewOrientation::NegY => "-Y", ViewOrientation::PosZ => "+Z", ViewOrientation::NegZ => "-Z",
            })?;
        for (control_id, step) in [
            ("SetTranslateSnap", settings.translate_step),
            ("SetRotateSnapDegrees", settings.rotate_step_deg),
            ("SetScaleSnap", settings.scale_step),
        ] {
            // Debug preserves the decimal in an integer option ID ("2.0");
            // the caption remains the compact numeric value ("2").
            self.set_unreal_viewport_dropdown(control_id, format!("{step:?}"), step.to_string())?;
        }
        for (control_id, enabled) in [
            ("ToggleTranslateSnap", settings.translate_snap_enabled),
            ("ToggleRotateSnap", settings.rotate_snap_enabled),
            ("ToggleScaleSnap", settings.scale_snap_enabled),
        ] {
            self.set_control_active(control_id, enabled)?;
        }
        self.refresh_unreal_viewport_toolbar_overflow()?;
        Ok(())
    }

    fn set_unreal_viewport_dropdown(
        &mut self,
        control_id: &str,
        value: impl Into<String>,
        caption: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let value = value.into();
        self.mutate_control_property(control_id, "value", UiValue::String(value.clone()))?;
        self.mutate_control_property(control_id, "value_text", UiValue::String(caption.into()))?;
        self.mutate_control_property(control_id, "special_options", UiValue::Array(vec![UiValue::String(value)]))
    }
}

#[cfg(test)]
#[path = "../tests/unreal_viewport_toolbar.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/unreal_snap_toolbar.rs"]
mod unreal_snap_toolbar;
