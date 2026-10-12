use super::*;
use std::cell::Cell;

#[test]
fn stale_watch_completion_never_enters_the_live_action() {
    let old = ModulePluginLiveHostProject::new("project".into(), "instance".to_string(), 1);
    let reopened = ModulePluginLiveHostProject::new("project".into(), "instance".to_string(), 2);
    let completion = ModulePluginLiveHostCompletion {
        project: old,
        plugin_id: "native.demo".to_string(),
        result: Ok("candidate ready".to_string()),
    };
    let entered = Cell::new(false);

    let result = execute_matching_watch_completion(&completion, Some(&reopened), |_, _| {
        entered.set(true);
        Ok("reloaded".to_string())
    });

    assert!(result.is_none());
    assert!(!entered.get());
}

#[test]
fn ready_watch_completion_enters_the_live_action_once() {
    let active = ModulePluginLiveHostProject::new("project".into(), "instance".to_string(), 1);
    let completion = ModulePluginLiveHostCompletion {
        project: active.clone(),
        plugin_id: "native.demo".to_string(),
        result: Ok("candidate ready".to_string()),
    };
    let entered = Cell::new(0);

    let result = execute_matching_watch_completion(&completion, Some(&active), |id, project| {
        assert_eq!(id, "native.demo");
        assert_eq!(project.root(), Path::new("project"));
        entered.set(entered.get() + 1);
        Ok("reloaded".to_string())
    });

    assert_eq!(result, Some(Ok("reloaded".to_string())));
    assert_eq!(entered.get(), 1);
}

#[test]
fn failed_watch_preparation_never_enters_the_live_action() {
    let active = ModulePluginLiveHostProject::new("project".into(), "instance".to_string(), 1);
    let completion = ModulePluginLiveHostCompletion {
        project: active.clone(),
        plugin_id: "native.demo".to_string(),
        result: Err("artifact is unavailable".to_string()),
    };
    let entered = Cell::new(false);

    let result = execute_matching_watch_completion(&completion, Some(&active), |_, _| {
        entered.set(true);
        Ok("reloaded".to_string())
    });

    assert_eq!(result, Some(Err("artifact is unavailable".to_string())));
    assert!(!entered.get());
}

#[test]
fn successful_unload_requires_the_live_host_to_be_unloaded() {
    let outcome = reconcile_live_plugin_result(
        ModulePluginLiveHostCommand::Unload,
        "native.demo",
        Ok("unloaded".to_string()),
        &Ok(true),
    );
    assert!(outcome.unwrap_err().contains("still has it loaded"));
}

#[test]
fn successful_hot_reload_requires_a_loaded_plugin_and_verifiable_state() {
    let missing = reconcile_live_plugin_result(
        ModulePluginLiveHostCommand::HotReload,
        "native.demo",
        Ok("hot reloaded".to_string()),
        &Ok(false),
    );
    assert!(missing.unwrap_err().contains("no loaded plugin"));

    let unknown = reconcile_live_plugin_result(
        ModulePluginLiveHostCommand::HotReload,
        "native.demo",
        Ok("hot reloaded".to_string()),
        &Err("host unavailable".to_string()),
    );
    assert!(unknown.unwrap_err().contains("could not verify"));
}

#[test]
fn failed_hot_reload_preserves_the_primary_failure_even_with_last_good_loaded() {
    let outcome = reconcile_live_plugin_result(
        ModulePluginLiveHostCommand::HotReload,
        "native.demo",
        Err("replacement rejected".to_string()),
        &Ok(true),
    );
    assert_eq!(outcome.unwrap_err(), "replacement rejected");
}
