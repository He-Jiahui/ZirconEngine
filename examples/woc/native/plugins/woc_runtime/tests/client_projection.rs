use std::f32::consts::PI;

use woc_protocol::EntityRef;
use woc_runtime::{
    ActorAnimationInput, ActorAppearance, ActorPresentation, ActorTransform,
    BulkPresentationProjection, PresentationProjectionError, PresentationVec3,
};

fn entity(id: u64, generation: u32) -> EntityRef {
    EntityRef { id, generation }
}

fn actor(id: u64, generation: u32, x: f32, facing_radians: f32) -> ActorPresentation {
    ActorPresentation {
        entity: entity(id, generation),
        template_id: format!("actor_{id}"),
        transform: ActorTransform {
            translation: PresentationVec3 { x, y: 0.0, z: 0.0 },
            facing_radians,
        },
        animation: ActorAnimationInput::default(),
        appearance: ActorAppearance::default(),
    }
}

#[test]
fn projection_requires_canonical_actor_order_and_a_present_viewer() {
    let duplicate = BulkPresentationProjection {
        viewer: entity(1, 4),
        actors: vec![actor(1, 4, 0.0, 0.0), actor(1, 4, 1.0, 0.0)],
    };
    assert_eq!(
        duplicate.validate().expect_err("duplicate identity"),
        PresentationProjectionError::ActorsNotStrictlySorted { index: 1 }
    );

    let missing_viewer = BulkPresentationProjection {
        viewer: entity(9, 1),
        actors: vec![actor(1, 4, 0.0, 0.0), actor(2, 4, 1.0, 0.0)],
    };
    assert_eq!(
        missing_viewer
            .validate()
            .expect_err("viewer must be rendered"),
        PresentationProjectionError::ViewerMissing {
            viewer: entity(9, 1),
        }
    );
}

#[test]
fn projection_rejects_non_finite_or_negative_motion_inputs() {
    let mut projection = BulkPresentationProjection {
        viewer: entity(1, 4),
        actors: vec![actor(1, 4, f32::NAN, 0.0)],
    };
    assert_eq!(
        projection.validate().expect_err("position must be finite"),
        PresentationProjectionError::NonFiniteActorField {
            entity: entity(1, 4),
            field: "transform.translation.x",
        }
    );

    projection.actors[0].transform.translation.x = 0.0;
    projection.actors[0].animation.horizontal_speed = -0.01;
    assert_eq!(
        projection.validate().expect_err("speed cannot be negative"),
        PresentationProjectionError::NegativeHorizontalSpeed {
            entity: entity(1, 4),
        }
    );
}

#[test]
fn interpolation_visits_current_actors_and_uses_the_shortest_facing_arc() {
    let previous = BulkPresentationProjection {
        viewer: entity(1, 4),
        actors: vec![
            actor(1, 4, 0.0, 170.0_f32.to_radians()),
            actor(2, 4, 4.0, 0.0),
        ],
    };
    let current = BulkPresentationProjection {
        viewer: entity(1, 4),
        actors: vec![
            actor(1, 4, 6.0, (-170.0_f32).to_radians()),
            actor(3, 4, 9.0, 0.5),
        ],
    };

    previous.validate().expect("previous projection");
    current.validate().expect("current projection");
    let mut samples = Vec::new();
    current
        .visit_interpolated_from(&previous, 0.5, |actor, transform| {
            samples.push((actor.entity, transform));
        })
        .expect("valid alpha");

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].0, entity(1, 4));
    assert_eq!(samples[0].1.translation.x, 3.0);
    assert!((samples[0].1.facing_radians.abs() - PI).abs() < 0.000_01);
    assert_eq!(samples[1].0, entity(3, 4));
    assert_eq!(samples[1].1, current.actors[1].transform);
}

#[test]
fn a_reused_numeric_id_with_a_new_generation_never_interpolates_old_pose() {
    let previous = BulkPresentationProjection {
        viewer: entity(7, 1),
        actors: vec![actor(7, 1, -100.0, -2.0)],
    };
    let current = BulkPresentationProjection {
        viewer: entity(7, 2),
        actors: vec![actor(7, 2, 12.0, 1.0)],
    };

    let mut sampled = None;
    current
        .visit_interpolated_from(&previous, 0.25, |_, transform| sampled = Some(transform))
        .expect("valid alpha");
    assert_eq!(sampled, Some(current.actors[0].transform));
}

#[test]
fn interpolation_rejects_non_finite_alpha_and_clamps_finite_overshoot() {
    let previous = BulkPresentationProjection {
        viewer: entity(1, 0),
        actors: vec![actor(1, 0, 0.0, 0.0)],
    };
    let current = BulkPresentationProjection {
        viewer: entity(1, 0),
        actors: vec![actor(1, 0, 10.0, 1.0)],
    };

    assert_eq!(
        current
            .visit_interpolated_from(&previous, f32::NAN, |_, _| {})
            .expect_err("NaN alpha"),
        PresentationProjectionError::NonFiniteAlpha,
    );

    let mut transform = None;
    current
        .visit_interpolated_from(&previous, 2.0, |_, sampled| transform = Some(sampled))
        .expect("finite alpha is clamped");
    assert_eq!(transform, Some(current.actors[0].transform));
}
