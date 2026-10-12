use std::time::{Duration, Instant};

use super::ModulePluginDevelopmentWatchPoll;

#[test]
fn watch_completion_rejects_a_reopened_generation_at_the_same_project_root() {
    let old =
        super::ModulePluginLiveHostProject::new("project".into(), "instance-a".to_string(), 1);
    let reopened =
        super::ModulePluginLiveHostProject::new("project".into(), "instance-a".to_string(), 2);
    let other = super::ModulePluginLiveHostProject::new(
        "other-project".into(),
        "instance-a".to_string(),
        1,
    );
    let completion = super::ModulePluginLiveHostCompletion {
        project: old.clone(),
        plugin_id: "native.demo".to_string(),
        result: Ok("hot reloaded".to_string()),
    };

    assert!(completion.matches_project(Some(&old)));
    assert!(!completion.matches_project(Some(&reopened)));
    assert!(!completion.matches_project(Some(&other)));
    assert!(!completion.matches_project(None));
}

#[test]
fn development_watch_poll_keeps_the_earliest_host_wake_deadline() {
    let now = Instant::now();
    let mut poll = ModulePluginDevelopmentWatchPoll::default();

    poll.include_deadline(Some(now + Duration::from_secs(2)));
    poll.include_deadline(None);
    poll.include_deadline(Some(now + Duration::from_secs(1)));

    let (diagnostics, completions, deadline) = poll.into_parts();
    assert!(diagnostics.is_empty());
    assert!(completions.is_empty());
    assert_eq!(deadline, Some(now + Duration::from_secs(1)));
}
