use super::support::selection;
use super::*;

#[test]
fn executed_cluster_selection_pass_filters_deduplicates_and_sorts_cluster_selections() {
    let entity_a = 42_u64;
    let entity_b = 43_u64;
    let mut executed_submission_keys = HashSet::new();
    executed_submission_keys.insert((entity_a, 7));
    executed_submission_keys.insert((entity_b, 3));

    let selections = collect_execution_cluster_selections_from_submission_keys(
        Some(&[
            selection(
                Some(1),
                entity_a,
                7,
                30,
                1,
                300,
                0,
                VirtualGeometryPrepareClusterState::Resident,
            ),
            selection(
                Some(1),
                entity_a,
                9,
                40,
                2,
                400,
                0,
                VirtualGeometryPrepareClusterState::Resident,
            ),
            selection(
                None,
                entity_b,
                3,
                50,
                0,
                500,
                0,
                VirtualGeometryPrepareClusterState::PendingUpload,
            ),
            selection(
                Some(1),
                entity_a,
                7,
                20,
                0,
                200,
                0,
                VirtualGeometryPrepareClusterState::Resident,
            ),
            selection(
                Some(1),
                entity_a,
                7,
                20,
                0,
                200,
                0,
                VirtualGeometryPrepareClusterState::Resident,
            ),
        ]),
        &executed_submission_keys,
    );

    assert_eq!(
        selections,
        vec![
            selection(
                Some(1),
                entity_a,
                7,
                20,
                0,
                200,
                0,
                VirtualGeometryPrepareClusterState::Resident,
            ),
            selection(
                Some(1),
                entity_a,
                7,
                30,
                1,
                300,
                0,
                VirtualGeometryPrepareClusterState::Resident,
            ),
            selection(
                None,
                entity_b,
                3,
                50,
                0,
                500,
                0,
                VirtualGeometryPrepareClusterState::PendingUpload,
            ),
        ],
        "expected the shared compat executed-cluster seam to drop non-executed submissions, deduplicate repeated clusters, and emit the exact stable ordering that both VisBuffer64 and HardwareRasterization consume"
    );
}
