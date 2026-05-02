use zircon_runtime_interface::ui::component::{
    UiComponentDataSourceDescriptor, UiComponentDataSourceFieldDescriptor,
    UiComponentDataSourceKind, UiValueKind,
};

pub fn inspector_selected_entity_data_source() -> UiComponentDataSourceDescriptor {
    UiComponentDataSourceDescriptor::new(
        "inspector",
        "subject",
        "Selected Entity Inspector",
        UiComponentDataSourceKind::Inspector,
    )
    .with_subject("entity://selected")
    .writable(true)
    .with_value_kinds([UiValueKind::String, UiValueKind::Int, UiValueKind::Float])
    .with_fields([
        UiComponentDataSourceFieldDescriptor::new("name", "Name", UiValueKind::String)
            .writable(true)
            .group("Entity"),
        UiComponentDataSourceFieldDescriptor::new("parent", "Parent", UiValueKind::String)
            .writable(true)
            .group("Entity")
            .reference_kind("scene-entity"),
        UiComponentDataSourceFieldDescriptor::new(
            "transform.translation.x",
            "Translation X",
            UiValueKind::Float,
        )
        .writable(true)
        .group("Transform")
        .range(-100000.0, 100000.0)
        .step(0.1),
        UiComponentDataSourceFieldDescriptor::new(
            "transform.translation.y",
            "Translation Y",
            UiValueKind::Float,
        )
        .writable(true)
        .group("Transform")
        .range(-100000.0, 100000.0)
        .step(0.1),
        UiComponentDataSourceFieldDescriptor::new(
            "transform.translation.z",
            "Translation Z",
            UiValueKind::Float,
        )
        .writable(true)
        .group("Transform")
        .range(-100000.0, 100000.0)
        .step(0.1),
    ])
}
