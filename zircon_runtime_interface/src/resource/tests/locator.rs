use super::ResourceLocator;

#[test]
fn matches_display_without_formatting() {
    let locator =
        ResourceLocator::parse("package://com.zircon.navigation/nav/agent.znav#mesh").unwrap();

    assert!(locator.matches_display("package://com.zircon.navigation/nav/agent.znav#mesh"));
    assert!(!locator.matches_display("package://com.zircon.navigation/nav/agent.znav#other"));
    assert!(!locator.matches_display("res://com.zircon.navigation/nav/agent.znav#mesh"));
}
