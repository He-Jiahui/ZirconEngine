use super::{BehaviorNodeCategory, BehaviorNodeSemantics, StandardNodeDescriptor};

pub(super) const DESCRIPTORS: [StandardNodeDescriptor; 6] = [
    (
        "blackboard_condition",
        "Blackboard Condition",
        BehaviorNodeCategory::Decorator,
        BehaviorNodeSemantics::BlackboardCondition,
    ),
    (
        "cooldown",
        "Cooldown",
        BehaviorNodeCategory::Decorator,
        BehaviorNodeSemantics::Cooldown,
    ),
    (
        "time_limit",
        "Time Limit",
        BehaviorNodeCategory::Decorator,
        BehaviorNodeSemantics::TimeLimit,
    ),
    (
        "loop",
        "Loop",
        BehaviorNodeCategory::Decorator,
        BehaviorNodeSemantics::Loop,
    ),
    (
        "inverter",
        "Inverter",
        BehaviorNodeCategory::Decorator,
        BehaviorNodeSemantics::Inverter,
    ),
    (
        "force_result",
        "Force Result",
        BehaviorNodeCategory::Decorator,
        BehaviorNodeSemantics::ForceResult,
    ),
];
