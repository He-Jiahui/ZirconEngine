use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::core::math::Vec2;

use super::{
    HitData, HitTarget, PickingEventKind, PickingHoverMap, PickingPointerEvent, PointerAction,
    PointerButton, PointerId, PointerInput, PointerLocation,
};

#[derive(Clone, Debug, Default, PartialEq)]
/// 跨帧保存悬停目标与按键拖动状态的派发器，由 picking pipeline 在每帧输入后调用。
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

    /// 按退出、当前悬停变化、输入事件的顺序派发；释放与取消可据上一帧悬停目标完成收尾。
    /// 输入位置覆盖采样位置；取消的指针不会进入新悬停快照，派发结束后其按键状态也会清除。
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
                    let state = self.button_state_mut(pointer, button);
                    state.dragging_over.remove(&hit.target);
                    for dragged in state.dragging.keys().copied() {
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
                    let state = self.button_state_mut(pointer, button);
                    if state.dragging.is_empty()
                        || state
                            .dragging_over
                            .insert(hit.target, hit.hit.clone())
                            .is_some()
                    {
                        continue;
                    }
                    for dragged in state.dragging.keys().copied() {
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
#[path = "tests/pointer_event_state_optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "pointer_event_state/tests/optimization_batch_hq_runtime598_tests.rs"]
mod optimization_batch_hq_runtime598_tests;
