use std::collections::HashMap;

use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewInstance, ViewInstanceId};

use super::{
    descriptor_content_kind::descriptor_content_kind, placeholder_view::placeholder_view,
    ViewTabSnapshot,
};

pub(crate) fn resolve_view_tab(
    instance_id: &ViewInstanceId,
    instances: &HashMap<ViewInstanceId, ViewInstance>,
    descriptors: &HashMap<ViewDescriptorId, ViewDescriptor>,
) -> ViewTabSnapshot {
    let Some(instance) = instances.get(instance_id) else {
        return placeholder_view(
            instance_id.clone(),
            ViewDescriptorId::new("missing.instance"),
            format!("Missing View {}", instance_id.0),
        );
    };

    let Some(descriptor) = descriptors.get(&instance.descriptor_id) else {
        return placeholder_view(
            instance.instance_id.clone(),
            instance.descriptor_id.clone(),
            format!("Missing Descriptor {}", instance.title),
        );
    };

    ViewTabSnapshot {
        instance_id: instance.instance_id.clone(),
        descriptor_id: descriptor.descriptor_id.clone(),
        title: instance.title.clone(),
        icon_key: descriptor.icon_key.clone(),
        kind: descriptor.kind,
        host: instance.host.clone(),
        serializable_payload: instance.serializable_payload.clone(),
        dirty: instance.dirty,
        content_kind: descriptor_content_kind(&descriptor.descriptor_id),
        placeholder: false,
    }
}
