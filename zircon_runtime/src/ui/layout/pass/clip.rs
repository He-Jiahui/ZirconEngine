use zircon_runtime_interface::ui::layout::UiFrame;

/// 安排阶段计算继承裁剪；返回值还会作为下一层的裁剪输入，空交集必须与无裁剪区分。
///
/// 当节点启用 `clip_to_bounds` 时，裁剪区取祖先裁剪与自身矩形的交集。
/// 若祖先裁剪与自身矩形无交集，节点完全不可见，返回一个面积为零的空帧而非回退到自身矩形，
/// 避免重新开放已被祖先裁掉的区域。
pub(crate) fn resolve_clip_frame(
    inherited_clip: Option<UiFrame>,
    frame: UiFrame,
    clip_to_bounds: bool,
) -> Option<UiFrame> {
    if clip_to_bounds {
        match inherited_clip {
            // 无祖先裁剪：以自身矩形作为裁剪区。
            None => Some(frame),
            // 有祖先裁剪：取交集；无交集时返回零面积帧，保持完全不可见。
            Some(clip) => Some(clip.intersection(frame).unwrap_or(UiFrame::ZERO)),
        }
    } else {
        // 节点自身不裁剪，透传祖先裁剪。
        inherited_clip.and_then(|clip| clip.intersection(frame))
    }
}
