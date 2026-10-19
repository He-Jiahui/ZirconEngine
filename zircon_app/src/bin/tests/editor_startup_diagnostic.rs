use super::*;

#[test]
fn interactive_failures_are_visible_but_headless_and_smoke_failures_never_block() {
    let args = |items: &[&str]| items.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert!(should_show_dialog(&[], None, false));
    assert!(should_show_dialog(&args(&["--project", "project.zrproj"]), None, false));
    for headless in [vec!["--run", "plugin-list"], vec!["--run=plugin-list"], vec!["--help"], vec!["-h"]] {
        assert!(!should_show_dialog(&args(&headless), None, false));
    }
    assert!(!should_show_dialog(&[], None, true));
    for disabled in ["0", "false", "FALSE", "off", "no"] {
        assert!(!should_show_dialog(&[], Some(disabled), false));
    }
}

#[test]
fn dialog_preserves_the_actual_diagnostic_recovery_and_log_path() {
    let diagnostic = "runtime manifest missing; recovery=stage the same BuildSet";
    let path = Path::new("logs/editor.log");
    let text = failure_message(diagnostic, Some(path));
    assert!(text.starts_with(diagnostic));
    assert!(text.contains(&format!("Log: {}", path.display())));
    assert!(failure_message(diagnostic, None).ends_with("Log file unavailable."));
}
