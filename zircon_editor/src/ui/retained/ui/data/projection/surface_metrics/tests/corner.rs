use super::*;
use toml::Value;

#[test]
fn panel_radius_projects_into_the_generic_host_corner_radius() {
    let attributes = BTreeMap::from([("panel_radius".to_string(), Value::Float(12.0))]);

    assert_eq!(
        projected_corner_radius(&attributes, "notification-center"),
        12.0
    );
}
