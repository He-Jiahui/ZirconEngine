use super::*;

trait PoisonBridge: Send + Sync {
    fn sample(&self) -> i32;
}

impl PluginInterface for dyn PoisonBridge {
    const INTERFACE_ID: &'static str = "test.poison.bridge.v1";
}

struct PoisonBridgeProvider {
    value: i32,
}

impl PoisonBridge for PoisonBridgeProvider {
    fn sample(&self) -> i32 {
        self.value
    }
}

#[test]
// 直接验证停用清空 provider、恢复重新启用与热替换保持 trait 类型的一致发布。
fn bridge_entry_publishes_generation_and_provider_as_one_state() {
    let entry = BridgeEntry::new(
        <dyn PoisonBridge as PluginInterface>::INTERFACE_ID.to_string(),
        erased_provider(7),
        PluginModuleId::from_raw(7),
    );

    assert!(entry.provider_installed());
    assert_eq!(entry.status(), BridgeInterfaceStatus::Enabled);
    let (_, provider) = entry
        .provider::<dyn PoisonBridge>()
        .expect("initial provider");
    assert_eq!(provider.sample(), 7);

    entry.deactivate();
    assert!(!entry.provider_installed());
    assert_eq!(entry.status(), BridgeInterfaceStatus::Disabled);

    entry.restore_provider(erased_provider(11));
    let (_, provider) = entry
        .provider::<dyn PoisonBridge>()
        .expect("restored provider");
    assert_eq!(provider.sample(), 11);

    let replacement: Arc<dyn PoisonBridge> = Arc::new(PoisonBridgeProvider { value: 13 });
    entry.replace_provider(replacement);
    let (_, provider) = entry
        .provider::<dyn PoisonBridge>()
        .expect("replaced provider");
    assert_eq!(provider.sample(), 13);
}

fn erased_provider(value: i32) -> Arc<dyn Any + Send + Sync> {
    let provider: Arc<dyn PoisonBridge> = Arc::new(PoisonBridgeProvider { value });
    Arc::new(provider)
}

struct OtherBridgeProvider;

impl PluginInterface for OtherBridgeProvider {
    const INTERFACE_ID: &'static str = "test.other.bridge.v1";
}

#[test]
fn frozen_bridge_rejects_wrong_interface_without_changing_slot() {
    let (table, slot) = frozen_table();
    let entry = table.entry(slot).unwrap();
    let generation = entry.generation();

    let error = table
        .replace_provider(slot, Arc::new(OtherBridgeProvider))
        .unwrap_err();
    assert_eq!(
        error,
        RuntimeExtensionRegistryError::PluginInterfaceMismatch {
            expected: <dyn PoisonBridge as PluginInterface>::INTERFACE_ID.into(),
            actual: OtherBridgeProvider::INTERFACE_ID.into(),
        }
    );
    assert_eq!(entry.generation(), generation);
    assert_eq!(entry.status(), BridgeInterfaceStatus::Enabled);
    assert!(entry.provider_installed());
    assert_eq!(
        table.provider::<dyn PoisonBridge>(slot).unwrap().1.sample(),
        7
    );

    let replacement: Arc<dyn PoisonBridge> = Arc::new(PoisonBridgeProvider { value: 13 });
    table.replace_provider(slot, replacement).unwrap();
    assert_eq!(entry.generation(), generation.wrapping_add(2));
    assert_eq!(
        table.provider::<dyn PoisonBridge>(slot).unwrap().1.sample(),
        13
    );
}

#[test]
fn frozen_bridge_reload_rejects_wrong_interface_while_disabled() {
    let (table, slot) = frozen_table();
    table.set_enabled(slot, false).unwrap();
    let entry = table.entry(slot).unwrap();
    let generation = entry.generation();

    assert!(matches!(
        table.reload_provider(slot, Arc::new(OtherBridgeProvider)),
        Err(RuntimeExtensionRegistryError::PluginInterfaceMismatch { .. })
    ));
    assert_eq!(entry.generation(), generation);
    assert_eq!(entry.status(), BridgeInterfaceStatus::Disabled);
    assert!(entry.provider_installed());

    table.set_enabled(slot, true).unwrap();
    assert_eq!(
        table.provider::<dyn PoisonBridge>(slot).unwrap().1.sample(),
        7
    );
}

fn frozen_table() -> (FrozenBridgeTable, InterfaceSlot) {
    let provider: Arc<dyn PoisonBridge> = Arc::new(PoisonBridgeProvider { value: 7 });
    let interface_id = <dyn PoisonBridge as PluginInterface>::INTERFACE_ID;
    let table = FrozenBridgeTable::from_exports([(
        PluginModuleId::from_raw(7),
        interface_id.to_string(),
        InterfaceExport::new(provider),
    )]);
    let slot = table.resolve_slot(interface_id).unwrap();
    (table, slot)
}
