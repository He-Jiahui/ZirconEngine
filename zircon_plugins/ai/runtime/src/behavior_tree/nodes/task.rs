use super::{BehaviorNodeCategory, BehaviorNodeSemantics, StandardNodeDescriptor};

pub(super) const DESCRIPTORS: [StandardNodeDescriptor; 7] = [
    (
        "wait",
        "Wait",
        BehaviorNodeCategory::Task,
        BehaviorNodeSemantics::Wait,
    ),
    (
        "move_to",
        "Move To",
        BehaviorNodeCategory::Task,
        BehaviorNodeSemantics::MoveTo,
    ),
    (
        "play_animation",
        "Play Animation",
        BehaviorNodeCategory::Task,
        BehaviorNodeSemantics::PlayAnimation,
    ),
    (
        "set_blackboard",
        "Set Blackboard",
        BehaviorNodeCategory::Task,
        BehaviorNodeSemantics::SetBlackboard,
    ),
    (
        "emit_event",
        "Emit Event",
        BehaviorNodeCategory::Task,
        BehaviorNodeSemantics::EmitEvent,
    ),
    (
        "run_subtree",
        "Run Subtree",
        BehaviorNodeCategory::Task,
        BehaviorNodeSemantics::RunSubtree,
    ),
    (
        "script_task",
        "Script Task",
        BehaviorNodeCategory::Task,
        BehaviorNodeSemantics::ScriptTask,
    ),
];
