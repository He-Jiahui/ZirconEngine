use super::*;

#[test]
fn dense_avoidance_rotation_is_deterministic_and_agent_distinct_in_one_frame() {
    let agents = (0..=MAX_NAVIGATION_AVOIDANCE_NEIGHBORS)
        .map(|index| (index as u64, Vec3::ZERO, 0.05))
        .collect::<Vec<_>>();
    let index = NavigationAvoidanceIndex::new(&agents, &[]);
    let epoch = 41;

    let rows_for = |entity| {
        let mut rows = Vec::new();
        index.for_each_nearby_agent(Vec3::ZERO, 0.05, avoidance_rotation(epoch, entity), |row| {
            rows.push(row);
            true
        });
        rows
    };

    let first = rows_for(1);
    let second = rows_for(2);

    assert_eq!(first.len(), MAX_NAVIGATION_AVOIDANCE_NEIGHBORS);
    assert_ne!(
        first, second,
        "agents sharing one dense cell must receive distinct fair candidate rotations"
    );
    assert_eq!(first, rows_for(1));
}

#[test]
fn dense_avoidance_rotation_reaches_neighboring_cells_across_epochs() {
    let mut agents = (0..=MAX_NAVIGATION_AVOIDANCE_NEIGHBORS)
        .map(|index| (index as u64, Vec3::new(-0.2, 0.0, -0.2), 0.05))
        .collect::<Vec<_>>();
    let neighboring_entity = 10_000;
    agents.push((neighboring_entity, Vec3::new(0.2, 0.0, 0.2), 0.05));
    let index = NavigationAvoidanceIndex::new(&agents, &[]);

    let visited_neighboring_cell = (0..9).any(|epoch| {
        let mut rows = Vec::new();
        index.for_each_nearby_agent(Vec3::ZERO, 0.05, avoidance_rotation(epoch, 1), |row| {
            rows.push(agents[row].0);
            true
        });
        rows.contains(&neighboring_entity)
    });

    assert!(
        visited_neighboring_cell,
        "a dense earlier cell must not starve a neighboring-cell candidate across rotations"
    );
}

#[test]
fn dense_avoidance_rotation_remains_bounded_after_epoch_wraparound() {
    let agents = (0..=MAX_NAVIGATION_AVOIDANCE_NEIGHBORS)
        .map(|index| (index as u64, Vec3::ZERO, 0.05))
        .collect::<Vec<_>>();
    let index = NavigationAvoidanceIndex::new(&agents, &[]);
    let mut rows = Vec::new();

    index.for_each_nearby_agent(Vec3::ZERO, 0.05, usize::MAX, |row| {
        rows.push(row);
        true
    });

    assert_eq!(rows.len(), MAX_NAVIGATION_AVOIDANCE_NEIGHBORS);
}
