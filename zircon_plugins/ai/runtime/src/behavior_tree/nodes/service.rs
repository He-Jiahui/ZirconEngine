use super::{BehaviorNodeCategory, BehaviorNodeSemantics, StandardNodeDescriptor};

pub(super) const DESCRIPTORS: [StandardNodeDescriptor; 1] = [(
    "update_blackboard_distance",
    "Update Blackboard Distance",
    BehaviorNodeCategory::Service,
    BehaviorNodeSemantics::UpdateBlackboardDistance,
)];
