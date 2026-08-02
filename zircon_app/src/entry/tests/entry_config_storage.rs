#[test]
fn entry_config_storage_uses_fallible_writes_in_both_bootstrap_phases() {
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
        "runtime_handle.store_config(RENDER_PROFILE_CONFIG_KEY, &self.config.render_profile)?;",
        "runtime_handle.store_config(\n            PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,\n            &self.config.window_descriptor,\n        )?;",
    ] {
        assert!(store_body.contains(fallible_write));
    }
    assert!(!store_body.contains(".ok()"));
    assert!(!store_body.contains(".unwrap()"));
    assert!(!store_body.contains("let _ = runtime_handle.store_config"));
    assert!(store_body.contains("Ok(())"));
    assert_eq!(
        source
            .match_indices("self.store_entry_config(&runtime)?;")
            .count(),
        2,
        "entry config storage must fail closed before and after module activation"
    );
}
