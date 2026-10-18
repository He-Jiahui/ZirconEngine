use zircon_runtime_interface::ui::component::UiValue;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const GRAPHICS_BANNER: &str = "WorkbenchGraphicsDegradation";
const MAX_BANNER_CHARACTERS: usize = 240;

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn prepare_graphics_degradation(
        &mut self,
        diagnostic: Option<&str>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if !self.has_control(GRAPHICS_BANNER) {
            return Ok(());
        }
        let diagnostic = diagnostic.map(str::trim).filter(|text| !text.is_empty());
        self.set_visible(GRAPHICS_BANNER, diagnostic.is_some())?;
        let full_text = diagnostic.unwrap_or_default();
        let text = full_text.chars().take(MAX_BANNER_CHARACTERS).collect();
        self.mutate_control_property(GRAPHICS_BANNER, "text", UiValue::String(text))?;
        self.mutate_control_property(GRAPHICS_BANNER, "tooltip", UiValue::String(full_text.to_owned()))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/graphics_degradation.rs"]
mod tests;
