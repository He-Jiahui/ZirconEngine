#[path = "../../../docks/pane/fallback/empty_state.rs"]
mod empty_state;

#[cfg(test)]
#[path = "../../../docks/pane/fallback/tests/cases.rs"]
mod tests;

pub(super) use empty_state::draw_pane_fallback;
