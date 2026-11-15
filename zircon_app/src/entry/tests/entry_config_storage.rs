//! 检查 EngineEntry 在模块注册前、Foundation 读盘后和全部模块激活后都以可失败写入储存平台、渲染和
//! 窗口配置，禁止静默吞错；并要求 Foundation 先于其余模块激活（官方评审 R01-R1）。
//! 该守卫约束源级接线，仍需结合被调用实现理解运行时契约。

#[test]
fn entry_config_storage_uses_fallible_writes_in_every_bootstrap_phase() {
    let source = include_str!("../engine_entry.rs").replace("\r\n", "\n");
    let store_body = source
        .split("fn store_entry_config")
        .nth(1)
        .and_then(|body| body.split("impl EngineEntry").next())
        .expect("entry config storage helper should remain before EngineEntry bootstrap");

    assert!(source
        .contains("fn store_entry_config(&self, runtime: &CoreRuntime) -> Result<(), CoreError>"));
    assert_eq!(store_body.matches(".store_config(").count(), 3);
    for fallible_write in [
        "runtime_handle.store_config(\n            PLATFORM_CONFIG_KEY,\n            &platform_config_for_entry_config(&self.config),\n        )?;",
        "runtime_handle.store_config(RENDER_PROFILE_CONFIG_KEY, self.config.render_profile())?;",
        "runtime_handle.store_config(\n            PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,\n            self.config.window_descriptor(),\n        )?;",
    ] {
        assert!(store_body.contains(fallible_write));
    }
    assert!(!store_body.contains(".ok()"));
    assert!(!store_body.contains(".unwrap()"));
    assert!(!store_body.contains("let _ = runtime_handle.store_config"));
    assert!(store_body.contains("Ok(())"));
    assert_eq!(
        source
            .match_indices("self.store_entry_config(runtime)?;")
            .count(),
        3,
        "entry config storage must fail closed before registration, after the Foundation disk load, and after module activation"
    );

    let foundation_activation = source
        .find("runtime.activate_module(FOUNDATION_MODULE_NAME)?;")
        .expect("bootstrap should activate Foundation before the remaining modules");
    let overlay_reapplication = source[foundation_activation..]
        .find("self.store_entry_config(runtime)?;")
        .map(|offset| foundation_activation + offset)
        .expect("entry overlay should be reapplied after the Foundation disk load");
    // The trait-default bootstrap above also batch-activates, so search after the Foundation step.
    let batch_activation = source[foundation_activation..]
        .find("runtime.activate_registered_modules()?;")
        .map(|offset| foundation_activation + offset)
        .expect("bootstrap should activate the remaining registered modules");
    assert!(
        overlay_reapplication < batch_activation,
        "entry overlay must be reapplied before the remaining modules activate"
    );
}
