use crate::asset::{
    module_descriptor, AssetIoDriver, ASSET_IO_DRIVER_NAME, ASSET_MANAGER_NAME,
    PROJECT_ASSET_MANAGER_NAME, RESOURCE_MANAGER_NAME,
};

#[test]
fn asset_module_descriptor_exposes_only_implemented_services() {
    let descriptor = module_descriptor();

    assert!(
        descriptor.drivers.is_empty(),
        "{ASSET_IO_DRIVER_NAME} must stay unregistered until it owns real I/O work"
    );
    assert_eq!(descriptor.managers.len(), 3);
    assert!(descriptor
        .managers
        .iter()
        .any(|manager| manager.name.as_str() == PROJECT_ASSET_MANAGER_NAME));
    assert!(descriptor
        .managers
        .iter()
        .any(|manager| manager.name.as_str() == ASSET_MANAGER_NAME));
    assert!(descriptor
        .managers
        .iter()
        .any(|manager| manager.name.as_str() == RESOURCE_MANAGER_NAME));
    assert_eq!(
        descriptor.description,
        "Project asset pipeline, import workers, and resource indexing"
    );

    let prove_uninhabited = |driver: AssetIoDriver| -> ! { match driver {} };
    let _: fn(AssetIoDriver) -> ! = prove_uninhabited;
}
