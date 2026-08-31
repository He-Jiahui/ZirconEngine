use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchDisposition, UiDispatchPhase, UiDispatchReplyStepTrace, UiInputDispatchResult,
        UiInputRoutePolicy, UiInputRouteTrace,
    },
    event_ui::UiNodeId,
};

pub(super) fn annotate_result_route_steps(result: &mut UiInputDispatchResult) {
    if !result.diagnostics.route_steps.is_empty() {
        return;
    }

    let trace = &result.diagnostics.route_trace;
    let explicit_handler = result.reply.handler.or(result.diagnostics.blocked_by);
    let steps = match result.diagnostics.route_policy {
        UiInputRoutePolicy::Bubble | UiInputRoutePolicy::FocusPath => routed_path_steps(
            trace,
            result.diagnostics.route_target,
            explicit_handler,
            result.reply.disposition,
            result.reply.effects.len(),
            result.reply.phase,
        ),
        UiInputRoutePolicy::Direct | UiInputRoutePolicy::PointerCapture => direct_route_steps(
            trace
                .direct_target
                .or(trace.capture_target)
                .or(trace.target)
                .or(result.diagnostics.route_target)
                .or(explicit_handler),
            explicit_handler,
            result.reply.disposition,
            result.reply.effects.len(),
        ),
        UiInputRoutePolicy::DefaultAction => default_action_steps(
            trace
                .direct_target
                .or(trace.target)
                .or(result.diagnostics.route_target)
                .or(explicit_handler),
            explicit_handler,
            result.reply.disposition,
            result.reply.effects.len(),
        ),
        UiInputRoutePolicy::PreviewTunnel => preview_only_steps(trace),
        UiInputRoutePolicy::Unrouted => Vec::new(),
    };

    result.diagnostics.route_steps = steps;
}

fn routed_path_steps(
    trace: &UiInputRouteTrace,
    route_target: Option<UiNodeId>,
    explicit_handler: Option<UiNodeId>,
    disposition: UiDispatchDisposition,
    effect_count: usize,
    terminal_phase: Option<UiDispatchPhase>,
) -> Vec<UiDispatchReplyStepTrace> {
    let terminal = explicit_handler.or(route_target).or(trace.target);
    let path = if trace.bubble_path.is_empty() {
        &trace.focus_path
    } else {
        &trace.bubble_path
    };
    let terminal_step_capacity = usize::from(
        terminal.is_some()
            && matches!(
                disposition,
                UiDispatchDisposition::Handled | UiDispatchDisposition::Blocked
            ),
    );
    let step_capacity = trace
        .preview_tunnel
        .len()
        .saturating_add(path.len())
        .saturating_add(terminal_step_capacity);
    let mut steps = if step_capacity == 0 {
        Vec::new()
    } else {
        Vec::with_capacity(step_capacity)
    };

    for node_id in &trace.preview_tunnel {
        steps.push(routed_step(
            UiDispatchPhase::PreviewTunnel,
            *node_id,
            terminal,
            terminal_phase,
            disposition,
            effect_count,
        ));
        if should_stop_at(
            *node_id,
            terminal,
            terminal_phase,
            UiDispatchPhase::PreviewTunnel,
            disposition,
        ) {
            return steps;
        }
    }

    let Some((&target, ancestors)) = path.split_first() else {
        append_out_of_route_terminal_step(&mut steps, terminal, disposition, effect_count);
        return steps;
    };

    steps.push(routed_step(
        UiDispatchPhase::Target,
        target,
        terminal,
        terminal_phase,
        disposition,
        effect_count,
    ));
    if should_stop_at(
        target,
        terminal,
        terminal_phase,
        UiDispatchPhase::Target,
        disposition,
    ) {
        return steps;
    }

    for node_id in ancestors {
        steps.push(routed_step(
            UiDispatchPhase::Bubble,
            *node_id,
            terminal,
            terminal_phase,
            disposition,
            effect_count,
        ));
        if should_stop_at(
            *node_id,
            terminal,
            terminal_phase,
            UiDispatchPhase::Bubble,
            disposition,
        ) {
            break;
        }
    }

    append_out_of_route_terminal_step(&mut steps, terminal, disposition, effect_count);
    steps
}

fn append_out_of_route_terminal_step(
    steps: &mut Vec<UiDispatchReplyStepTrace>,
    terminal: Option<UiNodeId>,
    disposition: UiDispatchDisposition,
    effect_count: usize,
) {
    if steps.iter().any(|step| step.stopped)
        || !matches!(
            disposition,
            UiDispatchDisposition::Handled | UiDispatchDisposition::Blocked
        )
    {
        return;
    }
    let Some(handler) = terminal else {
        return;
    };
    steps.push(terminal_step(
        UiDispatchPhase::DefaultAction,
        handler,
        Some(handler),
        disposition,
        effect_count,
    ));
}

fn direct_route_steps(
    target: Option<UiNodeId>,
    explicit_handler: Option<UiNodeId>,
    disposition: UiDispatchDisposition,
    effect_count: usize,
) -> Vec<UiDispatchReplyStepTrace> {
    target
        .map(|target| {
            terminal_step(
                UiDispatchPhase::Direct,
                target,
                explicit_handler,
                disposition,
                effect_count,
            )
        })
        .into_iter()
        .collect()
}

fn default_action_steps(
    target: Option<UiNodeId>,
    explicit_handler: Option<UiNodeId>,
    disposition: UiDispatchDisposition,
    effect_count: usize,
) -> Vec<UiDispatchReplyStepTrace> {
    target
        .map(|target| {
            terminal_step(
                UiDispatchPhase::DefaultAction,
                target,
                explicit_handler,
                disposition,
                effect_count,
            )
        })
        .into_iter()
        .collect()
}

fn preview_only_steps(trace: &UiInputRouteTrace) -> Vec<UiDispatchReplyStepTrace> {
    trace
        .preview_tunnel
        .iter()
        .map(|node_id| passthrough_step(UiDispatchPhase::PreviewTunnel, Some(*node_id)))
        .collect()
}

fn routed_step(
    phase: UiDispatchPhase,
    node_id: UiNodeId,
    terminal: Option<UiNodeId>,
    terminal_phase: Option<UiDispatchPhase>,
    disposition: UiDispatchDisposition,
    effect_count: usize,
) -> UiDispatchReplyStepTrace {
    if is_terminal_at(node_id, terminal, terminal_phase, phase) {
        terminal_step(phase, node_id, terminal, disposition, effect_count)
    } else {
        passthrough_step(phase, Some(node_id))
    }
}

fn terminal_step(
    phase: UiDispatchPhase,
    target: UiNodeId,
    explicit_handler: Option<UiNodeId>,
    disposition: UiDispatchDisposition,
    effect_count: usize,
) -> UiDispatchReplyStepTrace {
    let (effect_count, ignored_effect_count) = terminal_effect_counts(disposition, effect_count);
    UiDispatchReplyStepTrace {
        phase,
        target: Some(target),
        handler: explicit_handler.or(Some(target)),
        disposition,
        effect_start: 0,
        effect_count,
        ignored_effect_count,
        stopped: matches!(
            disposition,
            UiDispatchDisposition::Handled | UiDispatchDisposition::Blocked
        ),
    }
}

fn terminal_effect_counts(
    disposition: UiDispatchDisposition,
    effect_count: usize,
) -> (usize, usize) {
    if disposition == UiDispatchDisposition::Unhandled {
        (0, effect_count)
    } else {
        (effect_count, 0)
    }
}

fn passthrough_step(phase: UiDispatchPhase, target: Option<UiNodeId>) -> UiDispatchReplyStepTrace {
    UiDispatchReplyStepTrace {
        phase,
        target,
        handler: target,
        disposition: UiDispatchDisposition::Passthrough,
        effect_start: 0,
        effect_count: 0,
        ignored_effect_count: 0,
        stopped: false,
    }
}

fn should_stop_at(
    node_id: UiNodeId,
    terminal: Option<UiNodeId>,
    terminal_phase: Option<UiDispatchPhase>,
    phase: UiDispatchPhase,
    disposition: UiDispatchDisposition,
) -> bool {
    is_terminal_at(node_id, terminal, terminal_phase, phase)
        && matches!(
            disposition,
            UiDispatchDisposition::Handled | UiDispatchDisposition::Blocked
        )
}

fn is_terminal_at(
    node_id: UiNodeId,
    terminal: Option<UiNodeId>,
    terminal_phase: Option<UiDispatchPhase>,
    phase: UiDispatchPhase,
) -> bool {
    Some(node_id) == terminal && terminal_phase.unwrap_or(UiDispatchPhase::Target) == phase
}
