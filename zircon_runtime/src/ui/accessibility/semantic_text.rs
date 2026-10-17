//! 为名称/描述提供节点正文：普通文本直接复制，富文本使用 Surface 文本 owner 的语义投影。
//! 已发布视觉命令存在时只接受匹配正文、格式和代际的投影；没有该命令范围时可向文本 owner 编译。
//! 输出是拥有型可见文本，不把 Surface 或节点借用带出本次调用。

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, surface::UiRichTextFormat, tree::UiTreeNode,
};

use crate::{
    text::{resolve_rich_semantic_projection, RichSemanticProjection},
    ui::surface::{editable_text_input_is_secure, resolve_rich_text_format, UiSurface},
};

use super::name;

/// 读取节点自己的可朗读正文，不沿名称关系或子节点递归。
///
/// # 参数与返回
/// `surface` 提供当前渲染命令和文本编译 owner；`node` 提供作者正文与格式，均只在调用内借用。
/// 成功返回拥有型可见文本；正文缺失或富文本没有可接受的当前投影时返回无值。
///
/// # 约束
/// 已发布视觉命令是代际依据；存在命令但不匹配时，不绕过它重新编译作者数据。
pub(super) fn own_text(surface: &UiSurface, node: &UiTreeNode) -> Option<String> {
    if editable_text_input_is_secure(surface, node.node_id) {
        return None;
    }
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
