use super::BridgeImport;
use crate::core::framework::bridge::PluginInterface;

struct StaticInterface;

impl PluginInterface for StaticInterface {
    const INTERFACE_ID: &'static str = "zircon.fixture.static-interface.v1";
}

#[test]
// 约束擦除端克隆仍指向 trait 声明的静态 ID，避免合并时额外分配或更换接口身份。
fn erased_import_clones_share_static_interface_identity() {
    let (_, erased) = BridgeImport::<StaticInterface>::new();
    let cloned = erased.clone();

    assert_eq!(erased.interface_id(), StaticInterface::INTERFACE_ID);
    assert!(std::ptr::eq(
        erased.interface_id().as_ptr(),
        StaticInterface::INTERFACE_ID.as_ptr(),
    ));
    assert!(std::ptr::eq(
        erased.interface_id().as_ptr(),
        cloned.interface_id().as_ptr(),
    ));
}
