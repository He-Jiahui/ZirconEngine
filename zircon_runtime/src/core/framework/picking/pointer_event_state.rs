use std::collections::{BTreeMap, BTreeSet};

use crate::core::math::Vec2;

use super::{
    HitData, HitTarget, PickingEventKind, PickingHoverMap, PickingPointerEvent, PointerAction,
    PointerButton, PointerId, PointerInput, PointerLocation,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PickingEventState {
    previous_hover: PickingHoverMap,
    button_states: BTreeMap<(PointerId, PointerButton), PointerButtonEventState>,
}

impl PickingEventState {
    pub fn previous_hover(&self) -> &PickingHoverMap {
        &self.previous_hover
    }

    pub fn clear(&mut self) {
        self.previous_hover = PickingHoverMap::default();
        self.button_states.clear();
    }

    pub fn clear_pointer(&mut self, pointer: PointerId) {
        self.previous_hover.remove_pointer(pointer);
        self.button_states
            .retain(|(state_pointer, _), _| *state_pointer != pointer);
    }

    pub fn dispatch_frame(
        &mut self,
        mut current_hover: PickingHoverMap,
        pointer_locations: &[PointerLocation],
        inputs: &[PointerInput],
    ) -> Vec<PickingPointerEvent> {
        let previous_hover = self.previous_hover.clone();
        let canceled_pointers = canceled_pointers(inputs);
        for pointer in &canceled_pointers {
            current_hover.remove_pointer(*pointer);
        }
        let location_by_pointer = location_map(pointer_locations, inputs);
        let mut events = Vec::new();

        self.dispatch_exits(
            &previous_hover,
            &current_hover,
            &location_by_pointer,
            &mut events,
        );
        self.dispatch_current_hovers(
            &previous_hover,
            &current_hover,
            &location_by_pointer,
            &mut events,
        );

        let mut processed_cancels = BTreeSet::new();
        for input in inputs.iter().copied() {
            if processed_cancels.contains(&input.pointer()) {
                continue;
            }
            self.dispatch_input(input, &previous_hover, &current_hover, &mut events);
            if matches!(input.action, PointerAction::Cancel) {
                processed_cancels.insert(input.pointer());
            }
        }

        self.previous_hover = current_hover;
        for pointer in canceled_pointers {
            self.clear_pointer(pointer);
        }
        events
    }

    fn dispatch_exits(
        &mut self,
        previous_hover: &PickingHoverMap,
        current_hover: &PickingHoverMap,
        location_by_pointer: &BTreeMap<PointerId, PointerLocation>,
        events: &mut Vec<PickingPointerEvent>,
    ) {
        let exiting_hits = previous_hover
            .iter()
            .flat_map(|(pointer, hits)| {
                hits.iter()
                    .filter(move |hit| !current_hover.is_hovered(pointer, hit.target))
                    .cloned()
                    .map(move |hit| (pointer, hit))
            })
            .collect::<Vec<_>>();

        for (pointer, hit) in exiting_hits {
            let Some(location) = location_by_pointer.get(&pointer).copied() else {
                continue;
            };
            events.push(PickingPointerEvent::new(
                pointer,
                location,
                hit.target,
                PickingEventKind::Out {
                    hit: hit.hit.clone(),
                },
            ));
            events.push(PickingPointerEvent::new_without_propagate(
                pointer,
                location,
                hit.target,
                PickingEventKind::Leave {
                    hit: hit.hit.clone(),
                    was_direct: true,
                },
            ));

            for button in self.active_buttons(pointer) {
                let dragged_targets = {
                    let state = self.button_state_mut(pointer, button);
                    state.dragging_over.remove(&hit.target);
                    state.dragging.keys().copied().collect::<Vec<_>>()
                };
                for dragged in dragged_targets {
                    events.push(PickingPointerEvent::new(
                        pointer,
                        location,
                        hit.target,
                        PickingEventKind::DragLeave {
                            button,
                            dragged,
                            hit: hit.hit.clone(),
                        },
                    ));
                }
            }
        }
    }

    fn dispatch_current_hovers(
        &mut self,
        previous_hover: &PickingHoverMap,
        current_hover: &PickingHoverMap,
        location_by_pointer: &BTreeMap<PointerId, PointerLocation>,
        events: &mut Vec<PickingPointerEvent>,
    ) {
        let current_hits = current_hover
            .iter()
            .flat_map(|(pointer, hits)| hits.iter().cloned().map(move |hit| (pointer, hit)))
            .collect::<Vec<_>>();

        for (pointer, hit) in current_hits {
            let Some(location) = location_by_pointer.get(&pointer).copied() else {
                continue;
            };

            for button in self.active_buttons(pointer) {
                let dragged_targets = {
                    let state = self.button_state_mut(pointer, button);
                    if state.dragging.is_empty()
                        || state
                            .dragging_over
                            .insert(hit.target, hit.hit.clone())
                            .is_some()
                    {
                        Vec::new()
                    } else {
                        state.dragging.keys().copied().collect::<Vec<_>>()
                    }
                };
                for dragged in dragged_targets {
                    events.push(PickingPointerEvent::new(
                        pointer,
                        location,
                        hit.target,
                        PickingEventKind::DragEnter {
                            button,
                            dragged,
                            hit: hit.hit.clone(),
                        },
                    ));
                }
            }

            if !previous_hover.is_hovered(pointer, hit.target) {
                events.push(PickingPointerEvent::new_without_propagate(
                    pointer,
                    location,
                    hit.target,
                    PickingEventKind::Enter {
                        hit: hit.hit.clone(),
                        is_direct: true,
                    },
                ));
                events.push(PickingPointerEvent::new(
                    pointer,
                    location,
                    hit.target,
                    PickingEventKind::Over {
                        hit: hit.hit.clone(),
                    },
                ));
            }
        }
    }

    fn dispatch_input(
        &mut self,
        input: PointerInput,
        previous_hover: &PickingHoverMap,
        current_hover: &PickingHoverMap,
        events: &mut Vec<PickingPointerEvent>,
    ) {
        match input.action {
            PointerAction::Press(button) => {
                self.dispatch_press(input.location, button, current_hover, events)
            }
            PointerAction::Release(button) => {
                self.dispatch_release(input.location, button, previous_hover, events)
            }
            PointerAction::Move { delta } => {
                self.dispatch_move(input.location, delta, current_hover, events)
            }
            PointerAction::Scroll { unit, delta } => {
                for hit in current_hover.get(input.pointer()) {
                    events.push(PickingPointerEvent::new(
                        input.pointer(),
                        input.location,
                        hit.target,
                        PickingEventKind::Scroll {
                            unit,
                            delta,
                            hit: hit.hit.clone(),
                        },
                    ));
                }
            }
            PointerAction::Cancel => {
                for hit in previous_hover.get(input.pointer()) {
                    events.push(PickingPointerEvent::new(
                        input.pointer(),
                        input.location,
                        hit.target,
                        PickingEventKind::Cancel {
                            hit: hit.hit.clone(),
                        },
                    ));
                }
                self.clear_pointer(input.pointer());
            }
        }
    }

    fn dispatch_press(
        &mut self,
        location: PointerLocation,
        button: PointerButton,
        current_hover: &PickingHoverMap,
        events: &mut Vec<PickingPointerEvent>,
    ) {
        let pointer = location.pointer;
        for hit in current_hover.get(pointer) {
            events.push(PickingPointerEvent::new(
                pointer,
                location,
                hit.target,
                PickingEventKind::Press {
                    button,
                    hit: hit.hit.clone(),
                },
            ));
            self.button_state_mut(pointer, button).pressing.insert(
                hit.target,
                PressState {
                    location,
                    hit: hit.hit.clone(),
                },
            );
        }
    }

    fn dispatch_release(
        &mut self,
        location: PointerLocation,
        button: PointerButton,
        previous_hover: &PickingHoverMap,
        events: &mut Vec<PickingPointerEvent>,
    ) {
        let pointer = location.pointer;
        let previous_hits = previous_hover.get(pointer).to_vec();
        let (pressed_targets, dragging_targets, dragging_over) = {
            let state = self.button_state_mut(pointer, button);
            let pressed_targets = state.pressing.keys().copied().collect::<BTreeSet<_>>();
            let dragging_targets = state
                .dragging
                .iter()
                .map(|(target, drag)| (*target, drag.clone()))
                .collect::<Vec<_>>();
            let dragging_over = state
                .dragging_over
                .iter()
                .map(|(target, hit)| (*target, hit.clone()))
                .collect::<Vec<_>>();
            state.clear();
            (pressed_targets, dragging_targets, dragging_over)
        };

        for hit in previous_hits {
            if pressed_targets.contains(&hit.target) {
                events.push(PickingPointerEvent::new(
                    pointer,
                    location,
                    hit.target,
                    PickingEventKind::Click {
                        button,
                        hit: hit.hit.clone(),
                    },
                ));
            }
            events.push(PickingPointerEvent::new(
                pointer,
                location,
                hit.target,
                PickingEventKind::Release {
                    button,
                    hit: hit.hit.clone(),
                },
            ));
        }

        for (dragged, drag) in dragging_targets {
            for (drop_target, hit) in &dragging_over {
                events.push(PickingPointerEvent::new(
                    pointer,
                    location,
                    *drop_target,
                    PickingEventKind::DragDrop {
                        button,
                        dropped: dragged,
                        hit: hit.clone(),
                    },
                ));
            }
            events.push(PickingPointerEvent::new(
                pointer,
                location,
                dragged,
                PickingEventKind::DragEnd {
                    button,
                    distance: drag.latest_position - drag.start_position,
                },
            ));
            for (drop_target, hit) in &dragging_over {
                events.push(PickingPointerEvent::new(
                    pointer,
                    location,
                    *drop_target,
                    PickingEventKind::DragLeave {
                        button,
                        dragged,
                        hit: hit.clone(),
                    },
                ));
            }
        }
    }

    fn dispatch_move(
        &mut self,
        location: PointerLocation,
        delta: Vec2,
        current_hover: &PickingHoverMap,
        events: &mut Vec<PickingPointerEvent>,
    ) {
        if delta == Vec2::ZERO {
            return;
        }

        let pointer = location.pointer;
        for button in self.active_buttons(pointer) {
            self.start_drags(pointer, location, button, current_hover, events);
            self.update_drags(pointer, location, button, current_hover, events);
        }

        for hit in current_hover.get(pointer) {
            events.push(PickingPointerEvent::new(
                pointer,
                location,
                hit.target,
                PickingEventKind::Move {
                    hit: hit.hit.clone(),
                    delta,
                },
            ));
        }
    }

    fn start_drags(
        &mut self,
        pointer: PointerId,
        location: PointerLocation,
        button: PointerButton,
        current_hover: &PickingHoverMap,
        events: &mut Vec<PickingPointerEvent>,
    ) {
        let press_targets = {
            let state = self.button_state_mut(pointer, button);
            state
                .pressing
                .iter()
                .filter(|(target, _)| !state.dragging.contains_key(target))
                .map(|(target, press)| (*target, press.clone()))
                .collect::<Vec<_>>()
        };

        for (target, press) in press_targets {
            self.button_state_mut(pointer, button).dragging.insert(
                target,
                DragState {
                    start_position: press.location.position,
                    latest_position: press.location.position,
                },
            );
            events.push(PickingPointerEvent::new(
                pointer,
                press.location,
                target,
                PickingEventKind::DragStart {
                    button,
                    hit: press.hit.clone(),
                },
            ));

            for hovered in current_hover
                .get(pointer)
                .iter()
                .filter(|hovered| hovered.target != target)
            {
                self.button_state_mut(pointer, button)
                    .dragging_over
                    .insert(hovered.target, hovered.hit.clone());
                events.push(PickingPointerEvent::new(
                    pointer,
                    location,
                    hovered.target,
                    PickingEventKind::DragEnter {
                        button,
                        dragged: target,
                        hit: hovered.hit.clone(),
                    },
                ));
            }
        }
    }

    fn update_drags(
        &mut self,
        pointer: PointerId,
        location: PointerLocation,
        button: PointerButton,
        current_hover: &PickingHoverMap,
        events: &mut Vec<PickingPointerEvent>,
    ) {
        let drag_targets = self
            .button_state_mut(pointer, button)
            .dragging
            .keys()
            .copied()
            .collect::<Vec<_>>();

        for target in drag_targets {
            let Some((distance, delta)) =
                self.update_drag_position(pointer, button, target, location)
            else {
                continue;
            };
            events.push(PickingPointerEvent::new(
                pointer,
                location,
                target,
                PickingEventKind::Drag {
                    button,
                    distance,
                    delta,
                },
            ));

            for hovered in current_hover
                .get(pointer)
                .iter()
                .filter(|hovered| hovered.target != target)
            {
                events.push(PickingPointerEvent::new(
                    pointer,
                    location,
                    hovered.target,
                    PickingEventKind::DragOver {
                        button,
                        dragged: target,
                        hit: hovered.hit.clone(),
                    },
                ));
            }
        }
    }

    fn update_drag_position(
        &mut self,
        pointer: PointerId,
        button: PointerButton,
        target: HitTarget,
        location: PointerLocation,
    ) -> Option<(Vec2, Vec2)> {
        let drag = self
            .button_state_mut(pointer, button)
            .dragging
            .get_mut(&target)?;
        let delta = location.position - drag.latest_position;
        if delta == Vec2::ZERO {
            return None;
        }
        let distance = location.position - drag.start_position;
        drag.latest_position = location.position;
        Some((distance, delta))
    }

    fn active_buttons(&self, pointer: PointerId) -> Vec<PointerButton> {
        self.button_states
            .keys()
            .filter_map(|(state_pointer, button)| (*state_pointer == pointer).then_some(*button))
            .collect()
    }

    fn button_state_mut(
        &mut self,
        pointer: PointerId,
        button: PointerButton,
    ) -> &mut PointerButtonEventState {
        self.button_states.entry((pointer, button)).or_default()
    }
}

fn canceled_pointers(inputs: &[PointerInput]) -> BTreeSet<PointerId> {
    inputs
        .iter()
        .filter_map(|input| {
            matches!(input.action, PointerAction::Cancel).then_some(input.pointer())
        })
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq)]
struct PointerButtonEventState {
    pressing: BTreeMap<HitTarget, PressState>,
    dragging: BTreeMap<HitTarget, DragState>,
    dragging_over: BTreeMap<HitTarget, HitData>,
}

impl PointerButtonEventState {
    fn clear(&mut self) {
        self.pressing.clear();
        self.dragging.clear();
        self.dragging_over.clear();
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PressState {
    location: PointerLocation,
    hit: HitData,
}

#[derive(Clone, Debug, PartialEq)]
struct DragState {
    start_position: Vec2,
    latest_position: Vec2,
}

fn location_map(
    pointer_locations: &[PointerLocation],
    inputs: &[PointerInput],
) -> BTreeMap<PointerId, PointerLocation> {
    let mut locations = pointer_locations
        .iter()
        .copied()
        .map(|location| (location.pointer, location))
        .collect::<BTreeMap<_, _>>();
    for input in inputs {
        locations.insert(input.pointer(), input.location);
    }
    locations
}
