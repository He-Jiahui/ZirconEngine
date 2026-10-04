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
