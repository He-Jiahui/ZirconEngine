use super::*;
use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;
use crate::ui::retained_host::primitives::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

fn values(values: &[&str]) -> ModelRc<SharedString> {
    ModelRc::from(Rc::new(VecModel::from(
        values
            .iter()
            .map(|value| SharedString::from(*value))
            .collect::<Vec<_>>(),
    )))
}

fn rect() -> FrameRect {
    FrameRect {
        x: 4.0,
        y: 6.0,
        width: 260.0,
        height: 180.0,
    }
}

#[test]
fn workflow_roles_are_resolved_from_component_and_host_role() {
    assert!(matches!(
        agent_workflow_kind("mui-x-agent-plan", "Mount"),
        Some(AgentWorkflowKind::Plan)
    ));
    assert!(matches!(
        agent_workflow_kind("", "ToolCalls"),
        Some(AgentWorkflowKind::ToolCalls)
    ));
    assert!(agent_workflow_kind("panel", "Mount").is_none());
}

#[test]
fn agent_plan_projects_progress_steps_as_wrapped_native_text() {
    let node = TemplatePaneNodeData {
        component_role: "mui-x-agent-plan".into(),
        text: "Agent plan · 2/4 complete".into(),
        value_number: 0.5,
        collection_items: values(&[
            "done|Map ReactBits references",
            "active|Build retained ZUI surface",
            "next|Compare native screenshots",
        ]),
        ..TemplatePaneNodeData::default()
    };
    let mut commands = Vec::new();
    push_agent_workflow(
        &mut commands,
        &node,
        &rect(),
        &rect(),
        3,
        1.0,
        AgentWorkflowKind::Plan,
    );

    let texts = commands
        .iter()
        .filter_map(|command| command.text.as_deref())
        .collect::<Vec<_>>();
    assert!(texts
        .iter()
        .any(|text| text.contains("Map ReactBits references")));
    assert!(texts
        .iter()
        .any(|text| text.contains("Build retained ZUI surface")));
    assert!(commands
        .iter()
        .filter(|command| command.text.is_some())
        .all(|command| command.text_layout_policy == HostTextLayoutPolicy::WordWrap));
}

#[test]
fn tool_calls_project_each_status_without_falling_back_to_one_label() {
    let node = TemplatePaneNodeData {
        component_role: "mui-x-tool-calls".into(),
        text: "Tool calls · 3".into(),
        collection_items: values(&[
            "success|read_file|layout.zui",
            "pending|search_web|ReactBits",
            "failure|apply_patch|blocked",
        ]),
        notification_visible_limit: 3,
        ..TemplatePaneNodeData::default()
    };
    let mut commands = Vec::new();
    push_agent_workflow(
        &mut commands,
        &node,
        &rect(),
        &rect(),
        0,
        1.0,
        AgentWorkflowKind::ToolCalls,
    );

    let texts = commands
        .iter()
        .filter_map(|command| command.text.as_deref())
        .collect::<Vec<_>>();
    assert!(texts.iter().any(|text| text.contains("read_file")));
    assert!(texts.iter().any(|text| text.contains("search_web")));
    assert!(texts.iter().any(|text| text.contains("apply_patch")));
}

#[test]
fn approval_and_usage_keep_action_and_progress_geometry_inside_owner() {
    let approval = TemplatePaneNodeData {
        component_role: "mui-x-agent-approval".into(),
        text: "Approval required".into(),
        value_text: "Write layout".into(),
        label_text: "Scope: fixtures".into(),
        component_variant: "pending destructive".into(),
        options: values(&["Deny", "Allow write"]),
        ..TemplatePaneNodeData::default()
    };
    let usage = TemplatePaneNodeData {
        component_role: "mui-x-ai-usage".into(),
        text: "AI usage".into(),
        value_text: "12k used".into(),
        value_number: 0.39,
        ..TemplatePaneNodeData::default()
    };
    let mut approval_commands = Vec::new();
    let mut usage_commands = Vec::new();
    let frame = rect();
    push_agent_workflow(
        &mut approval_commands,
        &approval,
        &frame,
        &frame,
        0,
        1.0,
        AgentWorkflowKind::Approval,
    );
    push_agent_workflow(
        &mut usage_commands,
        &usage,
        &frame,
        &frame,
        0,
        1.0,
        AgentWorkflowKind::Usage,
    );
    assert!(approval_commands.iter().any(|command| {
        command.text.as_deref() == Some("Allow write") && command.frame.right() <= frame.right()
    }));
    assert!(usage_commands.iter().any(|command| {
        command.text.as_deref() == Some("12k used") && command.frame.bottom() <= frame.bottom()
    }));
}
