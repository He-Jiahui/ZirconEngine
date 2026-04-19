use serde_json::Value;

use crate::default_constraints_for_content;
use crate::layout::MainPageId;
use crate::ui::workbench::startup::{WELCOME_DESCRIPTOR_ID, WELCOME_INSTANCE_ID, WELCOME_PAGE_ID};
use crate::view::{
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId,
    ViewKind,
};
use crate::ViewContentKind;

pub(crate) fn welcome_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new(WELCOME_DESCRIPTOR_ID),
        ViewKind::ActivityWindow,
        "Welcome",
    )
    .with_preferred_host(PreferredHost::ExclusiveMainPage)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Welcome))
    .with_icon_key("welcome")
}

pub(super) fn welcome_view_instance() -> ViewInstance {
    ViewInstance {
        instance_id: ViewInstanceId::new(WELCOME_INSTANCE_ID),
        descriptor_id: ViewDescriptorId::new(WELCOME_DESCRIPTOR_ID),
        title: "Welcome".to_string(),
        serializable_payload: Value::Null,
        dirty: false,
        host: ViewHost::ExclusivePage(MainPageId::new(WELCOME_PAGE_ID)),
    }
}
