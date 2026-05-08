use super::{HitData, HitTarget, Pickable};

#[derive(Clone, Debug, PartialEq)]
pub struct HitRecord {
    pub target: HitTarget,
    pub hit: HitData,
    pub pickable: Pickable,
}

impl HitRecord {
    pub fn new(target: HitTarget, hit: HitData) -> Self {
        Self {
            target,
            hit,
            pickable: Pickable::default(),
        }
    }

    pub fn with_pickable(mut self, pickable: Pickable) -> Self {
        self.pickable = pickable;
        self
    }
}
