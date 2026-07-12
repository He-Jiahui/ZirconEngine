use super::{BehaviorNodeCategory, BehaviorNodeSemantics, StandardNodeDescriptor};

pub(super) const DESCRIPTORS: [StandardNodeDescriptor; 4] = [
    (
        "selector",
        "Selector",
        BehaviorNodeCategory::Composite,
        BehaviorNodeSemantics::Selector,
    ),
    (
        "sequence",
        "Sequence",
        BehaviorNodeCategory::Composite,
        BehaviorNodeSemantics::Sequence,
    ),
    (
        "parallel",
        "Parallel",
        BehaviorNodeCategory::Composite,
        BehaviorNodeSemantics::Parallel,
    ),
    (
        "random_selector",
        "Random Selector",
        BehaviorNodeCategory::Composite,
        BehaviorNodeSemantics::RandomSelector,
    ),
];
