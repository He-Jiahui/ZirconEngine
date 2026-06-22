use zircon_runtime_interface::ui::component::UiComponentDescriptor;

pub(super) fn accepted_drag_payloads(
    component_descriptor: Option<&UiComponentDescriptor>,
) -> String {
    component_descriptor
        .map(|descriptor| {
            descriptor
                .drop_policy
                .accepts
                .iter()
                .map(|kind| kind.as_str())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default()
}
