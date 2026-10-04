#[path = "effect_stack.rs"]
mod effect_stack;
#[path = "exposure.rs"]
mod exposure;
#[path = "screen_space_reflection.rs"]
mod screen_space_reflection;
#[path = "temporal_history.rs"]
mod temporal_history;
#[path = "terminal_chain.rs"]
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
