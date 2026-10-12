use super::scene_view_descriptor;

#[test]
fn scene_descriptor_admits_split_clone_instances() {
    assert!(
        scene_view_descriptor().multi_instance,
        "split Scene leaves require independent descriptor instances"
    );
}
