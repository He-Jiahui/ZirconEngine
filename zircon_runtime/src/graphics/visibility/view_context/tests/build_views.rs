#[test]
fn extra_views_share_one_frustum_candidate_projection() {
    let source = include_str!("../build_views.rs");
    let builder = concat!("build_frustum_candidates(", "&frame_visibility)");

    assert_eq!(source.matches(builder).count(), 1);
}
