use super::super::super::data::HostResizeLayerData;
use super::super::UiProfileNamedFrame;
use super::frame_math::push_named_frame;

pub(in crate::ui::retained_host::host_contract) fn collect_resize_splitters(
    resize_layer: &HostResizeLayerData,
) -> Vec<UiProfileNamedFrame> {
    let mut resize_splitters = Vec::with_capacity(3);
    push_named_frame(
        &mut resize_splitters,
        "resize.left_splitter",
        "resize_splitter",
        "left",
        resize_layer.left_splitter_frame.clone(),
        None,
    );
    push_named_frame(
        &mut resize_splitters,
        "resize.right_splitter",
        "resize_splitter",
        "right",
        resize_layer.right_splitter_frame.clone(),
        None,
    );
    push_named_frame(
        &mut resize_splitters,
        "resize.bottom_splitter",
        "resize_splitter",
        "bottom",
        resize_layer.bottom_splitter_frame.clone(),
        None,
    );
    resize_splitters
}

#[cfg(test)]
#[path = "tests/resize_splitters_optimization_batch_20260830bu_editor_tests.rs"]
mod optimization_batch_20260830bu_editor_tests;
