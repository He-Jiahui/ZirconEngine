#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PickingScheduleLabel {
    Input,
    RayMap,
    Backend,
    Hover,
    Events,
}
