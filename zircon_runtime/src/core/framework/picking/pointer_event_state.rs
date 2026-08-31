use std::collections::{BTreeMap, BTreeSet, HashMap};

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
        let previous_hover = std::mem::take(&mut self.previous_hover);
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
        location_by_pointer: &HashMap<PointerId, PointerLocation>,
        events: &mut Vec<PickingPointerEvent>,
    ) {
        for (pointer, hits) in previous_hover.iter() {
            let Some(location) = location_by_pointer.get(&pointer).copied() else {
                continue;
            };
            let active_buttons = self.active_buttons(pointer);
            for hit in hits
                .iter()
                .filter(|hit| !current_hover.is_hovered(pointer, hit.target))
            {
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

                for button in active_buttons.iter().copied() {
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
    }

    fn dispatch_current_hovers(
        &mut self,
        previous_hover: &PickingHoverMap,
        current_hover: &PickingHoverMap,
        location_by_pointer: &HashMap<PointerId, PointerLocation>,
        events: &mut Vec<PickingPointerEvent>,
    ) {
        for (pointer, hits) in current_hover.iter() {
            let Some(location) = location_by_pointer.get(&pointer).copied() else {
                continue;
            };
            let active_buttons = self.active_buttons(pointer);
            for hit in hits {
                for button in active_buttons.iter().copied() {
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
        let (pressed_targets, dragging_targets, dragging_over) = {
            let state = self.button_state_mut(pointer, button);
            (
                std::mem::take(&mut state.pressing),
                std::mem::take(&mut state.dragging),
                std::mem::take(&mut state.dragging_over),
            )
        };

        for hit in previous_hover.get(pointer) {
            if pressed_targets.contains_key(&hit.target) {
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
) -> HashMap<PointerId, PointerLocation> {
    let mut locations =
        HashMap::with_capacity(pointer_locations.len().saturating_add(inputs.len()));
    locations.extend(
        pointer_locations
            .iter()
            .copied()
            .map(|location| (location.pointer, location)),
    );
    for input in inputs {
        locations.insert(input.pointer(), input.location);
    }
    locations
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::render::RenderViewportHandle;

    use super::*;

    fn pointer_location(pointer: u64, x: f32) -> PointerLocation {
        PointerLocation::new(
            PointerId::new(pointer),
            RenderViewportHandle::new(3),
            Vec2::new(x, x + 1.0),
        )
    }

    #[test]
    fn runtime47_batch_location_map_preserves_input_override() {
        let initial = pointer_location(7, 10.0);
        let other = pointer_location(9, 20.0);
        let override_location = pointer_location(7, 30.0);
        let input = PointerInput::new(
            override_location,
            PointerAction::Move {
                delta: Vec2::new(20.0, 20.0),
            },
        );

        let locations = location_map(&[initial, other], &[input]);

        assert_eq!(locations.len(), 2);
        assert_eq!(locations.get(&PointerId::new(7)), Some(&override_location));
        assert_eq!(locations.get(&PointerId::new(9)), Some(&other));
    }

    #[test]
    fn runtime47_batch_location_map_uses_capacity_hash_index() {
        let source = include_str!("pointer_event_state.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("pointer event production source");
        let location_map = production
            .split("fn location_map")
            .nth(1)
            .expect("location map")
            .split("mod optimization_tests")
            .next()
            .expect("bounded location map");

        assert!(location_map.contains("HashMap<PointerId, PointerLocation>"));
        assert!(location_map.contains("HashMap::with_capacity"));
        assert!(!location_map.contains("collect::<BTreeMap"));
        assert_eq!(
            production
                .matches("&HashMap<PointerId, PointerLocation>")
                .count(),
            2
        );
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn runtime47_batch_pointer_location_hash_map_performance_evidence() {
        fn legacy_location_map(
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

        let pointer_locations = (0..32_768_u64)
            .map(|index| pointer_location(index, index as f32))
            .collect::<Vec<_>>();
        let inputs = (0..16_384_u64)
            .map(|index| {
                let location = pointer_location(index * 2, index as f32 + 0.5);
                PointerInput::new(
                    location,
                    PointerAction::Move {
                        delta: Vec2::new(0.5, 0.5),
                    },
                )
            })
            .collect::<Vec<_>>();
        const SAMPLE_PAIRS: usize = 17;
        let measure_legacy = || {
            let started = Instant::now();
            black_box(legacy_location_map(
                black_box(&pointer_locations),
                black_box(&inputs),
            ));
            started.elapsed().as_nanos().max(1)
        };
        let measure_hash = || {
            let started = Instant::now();
            black_box(location_map(
                black_box(&pointer_locations),
                black_box(&inputs),
            ));
            started.elapsed().as_nanos().max(1)
        };
        for _ in 0..3 {
            black_box(measure_legacy());
            black_box(measure_hash());
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy());
                hash_samples.push(measure_hash());
            } else {
                hash_samples.push(measure_hash());
                legacy_samples.push(measure_legacy());
            }
        }

        legacy_samples.sort_unstable();
        hash_samples.sort_unstable();
        let legacy_p50 = legacy_samples[8];
        let legacy_p95 = legacy_samples[16];
        let hash_p50 = hash_samples[8];
        let hash_p95 = hash_samples[16];
        println!(
            "RUNTIME47_POINTER_LOCATION_HASH_MAP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 hash_first_pairs=8 pointer_locations={} input_overrides={} legacy_p50_ns={} legacy_p95_ns={} hash_p50_ns={} hash_p95_ns={} legacy_tree_writes={} hash_writes={} target_ratio_bp=6000",
            pointer_locations.len(),
            inputs.len(),
            legacy_p50,
            legacy_p95,
            hash_p50,
            hash_p95,
            pointer_locations.len() + inputs.len(),
            pointer_locations.len() + inputs.len(),
        );
        assert!(
            hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "pointer location HashMap P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }
}
