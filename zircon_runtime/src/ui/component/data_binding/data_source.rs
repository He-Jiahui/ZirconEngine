//! 选中实体检查器的静态字段目录，供绑定适配器和属性编辑器交换目标元数据；subject 由宿主解析为当前选中实体。

use zircon_runtime_interface::ui::component::{
    UiComponentDataSourceDescriptor, UiComponentDataSourceFieldDescriptor,
    UiComponentDataSourceKind, UiValueKind,
};

/// 交付可写字段、分组与数值步长契约；调用方另行负责读取选中实体、校验实际写入和刷新投影。
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
