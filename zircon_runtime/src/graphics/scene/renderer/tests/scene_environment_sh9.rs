use super::SceneEnvironmentSh9;

#[test]
fn scene_environment_sh9_matches_gpu_artifact_layout() {
    assert_eq!(SceneEnvironmentSh9::byte_len(), 9 * 4 * 4);
    assert_eq!(
        SceneEnvironmentSh9::default().coefficients(),
        &[[0.0; 4]; 9]
    );
}
