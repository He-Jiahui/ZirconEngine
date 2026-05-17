use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonInputState<T>
where
    T: Clone + Ord,
{
    pressed: BTreeSet<T>,
    just_pressed: BTreeSet<T>,
    just_released: BTreeSet<T>,
}

impl<T> Default for ButtonInputState<T>
where
    T: Clone + Ord,
{
    fn default() -> Self {
        Self {
            pressed: BTreeSet::default(),
            just_pressed: BTreeSet::default(),
            just_released: BTreeSet::default(),
        }
    }
}

impl<T> ButtonInputState<T>
where
    T: Clone + Ord,
{
    pub fn from_pressed(inputs: impl IntoIterator<Item = T>) -> Self {
        Self {
            pressed: inputs.into_iter().collect(),
            just_pressed: BTreeSet::default(),
            just_released: BTreeSet::default(),
        }
    }

    pub fn press(&mut self, input: T) -> bool {
        if self.pressed.insert(input.clone()) {
            self.just_pressed.insert(input);
            true
        } else {
            false
        }
    }

    pub fn release(&mut self, input: &T) -> bool {
        if self.pressed.remove(input) {
            self.just_released.insert(input.clone());
            true
        } else {
            false
        }
    }

    pub fn release_where(&mut self, mut predicate: impl FnMut(&T) -> bool) -> Vec<T> {
        let released = self
            .pressed
            .iter()
            .filter(|input| predicate(input))
            .cloned()
            .collect::<Vec<_>>();
        for input in &released {
            self.release(input);
        }
        released
    }

    pub fn release_all(&mut self) -> Vec<T> {
        let released = self.pressed.iter().cloned().collect::<Vec<_>>();
        for input in &released {
            self.release(input);
        }
        released
    }

    pub fn clear_transitions(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn reset_all(&mut self) {
        self.pressed.clear();
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn pressed(&self, input: &T) -> bool {
        self.pressed.contains(input)
    }

    pub fn just_pressed(&self, input: &T) -> bool {
        self.just_pressed.contains(input)
    }

    pub fn just_released(&self, input: &T) -> bool {
        self.just_released.contains(input)
    }

    pub fn pressed_inputs(&self) -> Vec<T> {
        self.pressed.iter().cloned().collect()
    }

    pub fn just_pressed_inputs(&self) -> Vec<T> {
        self.just_pressed.iter().cloned().collect()
    }

    pub fn just_released_inputs(&self) -> Vec<T> {
        self.just_released.iter().cloned().collect()
    }
}
