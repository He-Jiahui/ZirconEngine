#[derive(Clone, Copy)]
pub(super) enum ExtensionBindingEventKind {
    Click,
    Change,
    Submit,
}

pub(super) struct ExtensionBindingSpec {
    pub(super) control_id: &'static str,
    pub(super) action_id: &'static str,
    pub(super) event_kind: ExtensionBindingEventKind,
}

pub(super) const fn click(
    control_id: &'static str,
    action_id: &'static str,
) -> ExtensionBindingSpec {
    binding(control_id, action_id, ExtensionBindingEventKind::Click)
}

pub(super) const fn change(
    control_id: &'static str,
    action_id: &'static str,
) -> ExtensionBindingSpec {
    binding(control_id, action_id, ExtensionBindingEventKind::Change)
}

pub(super) const fn submit(
    control_id: &'static str,
    action_id: &'static str,
) -> ExtensionBindingSpec {
    binding(control_id, action_id, ExtensionBindingEventKind::Submit)
}

const fn binding(
    control_id: &'static str,
    action_id: &'static str,
    event_kind: ExtensionBindingEventKind,
) -> ExtensionBindingSpec {
    ExtensionBindingSpec {
        control_id,
        action_id,
        event_kind,
    }
}
