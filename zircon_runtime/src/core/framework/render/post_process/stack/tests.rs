mod effect_stack;
mod exposure;
mod screen_space_reflection;
mod temporal_history;
mod terminal_chain;

use crate::core::framework::render::PostProcessGraphResourceNames;

fn expected_uber_effect_stack_outputs() -> Vec<String> {
    [
        PostProcessGraphResourceNames::EFFECT_STACKED,
        PostProcessGraphResourceNames::TONEMAPPED,
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
