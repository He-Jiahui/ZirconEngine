use zircon_runtime_interface::ui::dispatch::UiInputDispatchResult;

use super::super::super::surface::UiSurface;

pub(super) fn append_focus_input_method_lifecycle(
    surface: &mut UiSurface,
    result: &mut UiInputDispatchResult,
    effect_index: usize,
) {
    surface
        .input
        .append_deferred_focus_input_lifecycle(result, effect_index);
}
