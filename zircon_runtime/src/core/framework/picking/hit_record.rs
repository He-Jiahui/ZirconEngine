use super::{HitData, HitTarget, Pickable};

#[derive(Clone, Debug, PartialEq)]
/// 将目标、空间命中和穿透策略绑定为一条候选；解析悬停时按策略决定是否继续看更低层目标。
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
