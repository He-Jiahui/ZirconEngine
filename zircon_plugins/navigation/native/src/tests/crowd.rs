use zircon_runtime::core::framework::navigation::NavMeshAgentDescriptor;
use zircon_runtime::core::framework::navigation::{NavMeshAreaCostAsset, NavMeshAsset};

use crate::{RecastCrowd, RecastCrowdConfig};

#[test]
fn crowd_update_round_trips_agent_states() {
    let asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    let mut crowd = RecastCrowd::from_asset(
        &asset,
        RecastCrowdConfig {
            max_agents: 8,
            max_agent_radius: 1.0,
        },
    )
    .expect("create crowd");
    let handle = crowd
        .add_agent([-5.0, 0.0, 0.0], &NavMeshAgentDescriptor::default())
        .expect("add agent");
    crowd
        .set_target(handle, [5.0, 0.0, 0.0])
        .expect("set target");

    for _ in 0..8 {
        crowd.update(0.1).expect("update crowd");
    }

    let states = crowd.read_states().expect("read crowd states");
    let state = states
        .iter()
        .find(|state| state.handle == handle)
        .expect("agent state was returned in the batch");
    assert!(
        state.position[0] > -5.0,
        "agent should advance toward target"
    );
    assert!(state.velocity[0] > 0.0);
    assert!(state.desired_velocity[0] > 0.0);
}

#[test]
fn crowd_rejects_agent_area_mask_that_excludes_the_surface() {
    let asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    let mut crowd =
        RecastCrowd::from_asset(&asset, RecastCrowdConfig::default()).expect("create crowd");
    let error = crowd
        .add_agent(
            [0.0, 0.0, 0.0],
            &NavMeshAgentDescriptor {
                area_mask: 0,
                ..NavMeshAgentDescriptor::default()
            },
        )
        .expect_err("empty area mask must not silently use the default filter");

    assert!(error.to_string().contains("area mask"));
}

#[test]
fn crowd_syncs_controller_owned_position_into_the_corridor() {
    let asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    let mut crowd =
        RecastCrowd::from_asset(&asset, RecastCrowdConfig::default()).expect("create crowd");
    let handle = crowd
        .add_agent([-4.0, 0.0, 0.0], &NavMeshAgentDescriptor::default())
        .expect("add agent");
    crowd
        .set_target(handle, [4.0, 0.0, 0.0])
        .expect("set target");
    crowd
        .sync_agent_position(handle, [-2.0, 0.0, 1.0])
        .expect("sync controller position");

    let state = crowd
        .read_states()
        .expect("read states")
        .into_iter()
        .find(|state| state.handle == handle)
        .expect("agent state");
    assert!((state.position[0] + 2.0).abs() < 0.01);
    assert!((state.position[2] - 1.0).abs() < 0.01);
}

#[test]
fn crowd_recycles_inactive_query_filter_slots() {
    let mut asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    asset.area_costs = (0_u8..18)
        .map(|area| NavMeshAreaCostAsset {
            area,
            cost: 1.0 + f32::from(area) * 0.1,
            walkable: true,
        })
        .collect();
    let mut crowd = RecastCrowd::from_asset(
        &asset,
        RecastCrowdConfig {
            max_agents: 32,
            max_agent_radius: 1.0,
        },
    )
    .expect("create crowd");
    let mut handles = Vec::new();
    for area in 1_u8..=16 {
        handles.push(
            crowd
                .add_agent(
                    [0.0, 0.0, 0.0],
                    &NavMeshAgentDescriptor {
                        area_mask: (1_u64 << 1) | (1_u64 << area),
                        ..NavMeshAgentDescriptor::default()
                    },
                )
                .expect("allocate distinct active filter"),
        );
    }
    crowd
        .remove_agent(handles[0])
        .expect("release first filter");

    crowd
        .add_agent(
            [0.0, 0.0, 0.0],
            &NavMeshAgentDescriptor {
                area_mask: (1_u64 << 1) | (1_u64 << 17),
                ..NavMeshAgentDescriptor::default()
            },
        )
        .expect("inactive filter slot should be reusable");
}
