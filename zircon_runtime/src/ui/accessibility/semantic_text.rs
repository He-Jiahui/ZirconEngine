use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, surface::UiRichTextFormat, tree::UiTreeNode,
};

use crate::{
    text::{RichSemanticProjection, resolve_rich_semantic_projection},
    ui::surface::{UiSurface, resolve_rich_text_format},
};

use super::name;

pub(super) fn own_text(surface: &UiSurface, node: &UiTreeNode) -> Option<String> {
    let source = name::own_text(node.template_metadata.as_ref())?;
    let format = resolve_rich_text_format(node.template_metadata.as_ref());
    if format == UiRichTextFormat::Plain {
        return Some(source);
    }
    current_rich_projection(surface, node.node_id, &source, format)
        .map(|projection| projection.visible_text().to_owned())
}

fn current_rich_projection(
    surface: &UiSurface,
    node_id: UiNodeId,
    source_markup: &str,
    format: UiRichTextFormat,
) -> Option<RichSemanticProjection> {
    let Some(commands) = surface.current_render_commands_for_node(node_id) else {
        return surface.compile_rich_semantic_projection(source_markup, format.into());
    };
    // A published visual command range is authoritative. Never hide a stale visual/text
    // generation mismatch by compiling current metadata behind the render owner's back.
    let mut resolved: Option<RichSemanticProjection> = None;
    for command in commands {
        if command.text.as_deref() != Some(source_markup)
            || command.style.rich_text_format != format
        {
            continue;
        }
        let Some(handle) = command
            .text_layout
            .as_ref()
            .and_then(|layout| layout.rich_text_artifact.as_ref())
        else {
            continue;
        };
        let Some(candidate) =
            resolve_rich_semantic_projection(handle, source_markup, format.into())
        else {
            continue;
        };
        if resolved
            .as_ref()
            .is_some_and(|current| !current.shares_source_generation(&candidate))
        {
            return None;
        }
        resolved = Some(candidate);
    }
    resolved
}
