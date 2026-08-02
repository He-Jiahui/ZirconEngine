use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchPhase, UiInputDispatchResult,
        UiPointerInputEvent,
    },
    surface::{UiPointerActivationPhase, UiPointerButton, UiPointerEventKind, UiPointerRoute},
};

use crate::ui::text::link_at_layout_point;

use super::super::surface::UiSurface;
use super::effect::append_dispatch_effect_to_result;

pub(super) fn dispatch_pointer_rich_link_activation(
    surface: &mut UiSurface,
    pointer: &UiPointerInputEvent,
    route: &UiPointerRoute,
    result: &mut UiInputDispatchResult,
) {
    if !matches!(route.kind, UiPointerEventKind::Up)
        || !matches!(route.button, Some(UiPointerButton::Primary))
        || !matches!(
            route.activation_phase,
            UiPointerActivationPhase::PrimaryRelease
        )
        || !route.release_inside_pressed
        || matches!(result.reply.disposition, UiDispatchDisposition::Blocked)
    {
        return;
    }
    let Some(target) = route.click_target else {
        return;
    };
    let Some(hit) = surface
        .render_extract
        .list
        .commands
        .iter()
        .rev()
        .filter(|command| {
            command.node_id == target
                && command.frame.contains_point(route.point)
                && command
                    .clip_frame
                    .map(|clip| clip.contains_point(route.point))
                    .unwrap_or(true)
        })
        .find_map(|command| link_at_layout_point(command.text_layout.as_ref()?, route.point))
    else {
        return;
    };

    let applied_before = result.applied_effects.len();
    append_dispatch_effect_to_result(
        surface,
        result,
        UiDispatchEffect::RequestLinkActivation {
            target,
            href: hit.href.clone(),
        },
    );
    if result.applied_effects.len() == applied_before {
        return;
    }
    result.reply.disposition = UiDispatchDisposition::Handled;
    result.reply.handler = Some(target);
    result.reply.phase = Some(UiDispatchPhase::DefaultAction);
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some("pointer.rich_link_activation".to_string());
    result.diagnostics.notes.push(format!(
        "rich_link_range={}..{}:{:?}:click_count={}",
        hit.source_range.start, hit.source_range.end, hit.affinity, pointer.event.click_count
    ));
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        dispatch::{
            UiDispatchEffect, UiDispatchHostRequestKind, UiDispatchReply, UiInputDispatchResult,
            UiInputEvent, UiInputEventMetadata, UiPointerEvent, UiPointerInputEvent,
        },
        event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
        layout::{UiFrame, UiPoint},
        surface::{
            UiPointerActivationPhase, UiPointerButton, UiPointerEventKind, UiPointerRoute,
            UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiRichTextFormat,
            UiTextOverflow, UiTextWrap,
        },
        tree::UiTreeNode,
    };

    use crate::ui::{surface::UiSurface, text::layout_text};

    use super::dispatch_pointer_rich_link_activation;

    #[test]
    fn primary_release_on_rich_link_emits_host_activation_request() {
        let target = UiNodeId::new(7);
        let mut surface = UiSurface::new(UiTreeId::new("runtime.rich-link"));
        surface.tree.insert_root(
            UiTreeNode::new(target, UiNodePath::new("root/link"))
                .with_frame(UiFrame::new(0.0, 0.0, 320.0, 40.0)),
        );
        let mut style = UiResolvedStyle::default();
        style.rich_text_format = UiRichTextFormat::Html;
        style.wrap = UiTextWrap::None;
        style.text_overflow = UiTextOverflow::Clip;
        let markup = "before <a href=\"res://docs/help.md\">help</a> after";
        let layout = layout_text(markup, &style, UiFrame::new(0.0, 0.0, 320.0, 40.0), None);
        let point = UiPoint::new(
            layout.lines[0].frame.x + layout.lines[0].glyph_advances[..7].iter().sum::<f32>() + 1.0,
            layout.lines[0].frame.y + 4.0,
        );
        surface.render_extract.list.commands.push(UiRenderCommand {
            node_id: target,
            kind: UiRenderCommandKind::Text,
            frame: UiFrame::new(0.0, 0.0, 320.0, 40.0),
            clip_frame: None,
            z_index: 0,
            style,
            text_layout: Some(layout),
            text: Some(markup.to_string()),
            image: None,
            opacity: 1.0,
        });
        let pointer = UiPointerInputEvent {
            metadata: UiInputEventMetadata::default(),
            event: UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
            precise_scroll: None,
        };
        let route = UiPointerRoute {
            kind: UiPointerEventKind::Up,
            button: Some(UiPointerButton::Primary),
            modifiers: Default::default(),
            activation_phase: UiPointerActivationPhase::PrimaryRelease,
            point,
            scroll_delta: 0.0,
            target: Some(target),
            hit_path: Default::default(),
            bubbled: vec![target],
            stacked: vec![target],
            entered: Vec::new(),
            left: Vec::new(),
            captured: None,
            pressed: Some(target),
            click_target: Some(target),
            release_inside_pressed: true,
            focused: None,
            fallback_to_root: false,
            root_targets: vec![target],
        };
        let mut result = UiInputDispatchResult::new(
            UiInputEvent::Pointer(pointer.clone()),
            UiDispatchReply::unhandled(),
        );

        dispatch_pointer_rich_link_activation(&mut surface, &pointer, &route, &mut result);

        assert!(matches!(
            result.reply.effects.as_slice(),
            [UiDispatchEffect::RequestLinkActivation { target: effect_target, href }]
                if *effect_target == target && href == "res://docs/help.md"
        ));
        assert!(matches!(
            result.host_requests.as_slice(),
            [request]
                if matches!(
                    &request.request,
                    UiDispatchHostRequestKind::ActivateLink { target: request_target, href }
                        if *request_target == target && href == "res://docs/help.md"
                )
        ));
    }

    #[test]
    fn primary_release_on_second_table_cell_link_emits_host_activation_request() {
        let target = UiNodeId::new(17);
        let frame = UiFrame::new(0.0, 0.0, 360.0, 100.0);
        let mut surface = UiSurface::new(UiTreeId::new("runtime.rich-table-link"));
        surface.tree.insert_root(
            UiTreeNode::new(target, UiNodePath::new("root/table-link")).with_frame(frame),
        );
        let mut style = UiResolvedStyle::default();
        style.rich_text_format = UiRichTextFormat::BbCode;
        style.wrap = UiTextWrap::None;
        style.text_overflow = UiTextOverflow::Clip;
        let markup = "[table=2][cell]first[/cell][cell padding=18,12,16,10][url=res://docs/table-link.md]second link[/url][/cell][/table]";
        let layout = layout_text(markup, &style, frame, None);
        let link_line = layout
            .lines
            .iter()
            .find(|line| line.text.contains("second link"))
            .expect("second table cell link line");
        let point = UiPoint::new(link_line.frame.x + 2.0, link_line.frame.y + 2.0);
        surface.render_extract.list.commands.push(UiRenderCommand {
            node_id: target,
            kind: UiRenderCommandKind::Text,
            frame,
            clip_frame: None,
            z_index: 0,
            style,
            text_layout: Some(layout),
            text: Some(markup.to_string()),
            image: None,
            opacity: 1.0,
        });
        let pointer = UiPointerInputEvent {
            metadata: UiInputEventMetadata::default(),
            event: UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
            precise_scroll: None,
        };
        let route = UiPointerRoute {
            kind: UiPointerEventKind::Up,
            button: Some(UiPointerButton::Primary),
            modifiers: Default::default(),
            activation_phase: UiPointerActivationPhase::PrimaryRelease,
            point,
            scroll_delta: 0.0,
            target: Some(target),
            hit_path: Default::default(),
            bubbled: vec![target],
            stacked: vec![target],
            entered: Vec::new(),
            left: Vec::new(),
            captured: None,
            pressed: Some(target),
            click_target: Some(target),
            release_inside_pressed: true,
            focused: None,
            fallback_to_root: false,
            root_targets: vec![target],
        };
        let mut result = UiInputDispatchResult::new(
            UiInputEvent::Pointer(pointer.clone()),
            UiDispatchReply::unhandled(),
        );

        dispatch_pointer_rich_link_activation(&mut surface, &pointer, &route, &mut result);

        assert!(matches!(
            result.host_requests.as_slice(),
            [request]
                if matches!(
                    &request.request,
                    UiDispatchHostRequestKind::ActivateLink { target: request_target, href }
                        if *request_target == target && href == "res://docs/table-link.md"
                )
        ));
    }

    #[test]
    fn pointer_dispatch_routes_rich_link_through_default_action() {
        use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
        use zircon_runtime_interface::ui::tree::UiInputPolicy;

        let target = UiNodeId::new(8);
        let mut surface = UiSurface::new(UiTreeId::new("runtime.rich-link-dispatch"));
        surface.tree.insert_root(
            UiTreeNode::new(target, UiNodePath::new("root/link"))
                .with_frame(UiFrame::new(0.0, 0.0, 320.0, 40.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    ..UiStateFlags::default()
                })
                .with_input_policy(UiInputPolicy::Receive),
        );
        surface.rebuild();
        let mut style = UiResolvedStyle::default();
        style.rich_text_format = UiRichTextFormat::Html;
        style.wrap = UiTextWrap::None;
        style.text_overflow = UiTextOverflow::Clip;
        let markup = "before <a href=\"res://docs/help.md\">help</a> after";
        let layout = layout_text(markup, &style, UiFrame::new(0.0, 0.0, 320.0, 40.0), None);
        let point = UiPoint::new(
            layout.lines[0].frame.x + layout.lines[0].glyph_advances[..7].iter().sum::<f32>() + 1.0,
            layout.lines[0].frame.y + 4.0,
        );
        surface.render_extract.list.commands.push(UiRenderCommand {
            node_id: target,
            kind: UiRenderCommandKind::Text,
            frame: UiFrame::new(0.0, 0.0, 320.0, 40.0),
            clip_frame: None,
            z_index: 0,
            style,
            text_layout: Some(layout),
            text: Some(markup.to_string()),
            image: None,
            opacity: 1.0,
        });
        let mut pointer = UiPointerInputEvent {
            metadata: UiInputEventMetadata::default(),
            event: UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
            precise_scroll: None,
        };
        pointer.metadata.pointer_id =
            Some(zircon_runtime_interface::ui::dispatch::UiPointerId::new(1));
        let pointer_dispatcher = UiPointerDispatcher::default();
        let navigation_dispatcher = UiNavigationDispatcher::default();
        surface
            .dispatch_input_event(
                &pointer_dispatcher,
                &navigation_dispatcher,
                UiInputEvent::Pointer(UiPointerInputEvent {
                    metadata: pointer.metadata.clone(),
                    event: UiPointerEvent::new(UiPointerEventKind::Down, point)
                        .with_button(UiPointerButton::Primary),
                    precise_scroll: None,
                }),
            )
            .expect("rich-link pointer press should establish the shared click route");

        let result = surface
            .dispatch_input_event(
                &pointer_dispatcher,
                &navigation_dispatcher,
                UiInputEvent::Pointer(pointer),
            )
            .expect("rich-link pointer dispatch should remain routed");
        assert_eq!(
            result.diagnostics.handled_phase.as_deref(),
            Some("pointer.rich_link_activation")
        );
        assert!(matches!(
            result.host_requests.as_slice(),
            [request]
                if matches!(
                    &request.request,
                    UiDispatchHostRequestKind::ActivateLink { target: request_target, href }
                        if *request_target == target && href == "res://docs/help.md"
                )
        ));
    }
}
