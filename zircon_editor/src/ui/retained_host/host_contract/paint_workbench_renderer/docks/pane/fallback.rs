mod empty_state;

#[cfg(test)]
#[path = "fallback/tests/cases.rs"]
mod tests;

pub(super) use empty_state::draw_pane_fallback;
