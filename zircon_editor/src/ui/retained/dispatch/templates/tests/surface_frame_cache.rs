use super::{update_hit_route_key, SurfaceFrameNode, SurfaceFrameSignature};

fn signature_with_hit_controls(values: &[Option<&str>]) -> SurfaceFrameSignature {
    SurfaceFrameSignature {
        width_bits: 1280.0_f32.to_bits(),
        height_bits: 28.0_f32.to_bits(),
        nodes: values
            .iter()
            .enumerate()
            .map(|(index, hit_control_id)| SurfaceFrameNode {
                projection_control_id: format!("projection-{index}"),
                hit_control_id: hit_control_id.map(str::to_owned),
                component: "button".to_owned(),
                frame: Default::default(),
            })
            .collect(),
    }
}

#[test]
fn hit_route_key_updates_reuse_capacity_and_clear_cleanly() {
    let mut cached = None;
    update_hit_route_key(&mut cached, Some(&["first", "second"]));
    let values = cached.as_ref().expect("route key must be stored");
    let first_capacity = values.capacity();
    let first_value_capacity = values[0].capacity();
    let second_value_capacity = values[1].capacity();
    assert!(first_capacity >= 2);

    update_hit_route_key(&mut cached, Some(&["again", "route"]));
    let values = cached.as_ref().expect("route key remains stored");
    assert_eq!(values.len(), 2);
    assert_eq!(values[0], "again");
    assert_eq!(values[1], "route");
    assert_eq!(
        values.capacity(),
        first_capacity,
        "same-shape route updates should reuse the retained allocation"
    );
    assert_eq!(
        values[0].capacity(),
        first_value_capacity,
        "same-shape route updates should reuse each retained string buffer"
    );
    assert_eq!(
        values[1].capacity(),
        second_value_capacity,
        "same-shape route updates should reuse each retained string buffer"
    );

    update_hit_route_key(
        &mut cached,
        Some(&[
            "a-route-value-long-enough-to-grow-the-retained-string",
            "another-route-value-long-enough-to-grow-the-retained-string",
        ]),
    );
    let values = cached.as_ref().expect("grown route strings remain stored");
    let first_long_capacity = values[0].capacity();
    let second_long_capacity = values[1].capacity();
    assert!(first_long_capacity >= values[0].len());
    assert!(second_long_capacity >= values[1].len());

    update_hit_route_key(
        &mut cached,
        Some(&[
            "a-route-value-long-enough-to-grow-the-retained-string-again",
            "another-route-value-long-enough-to-grow-the-retained-string-again",
        ]),
    );
    let values = cached.as_ref().expect("route strings remain stored");
    assert_eq!(
        values[0].capacity(),
        first_long_capacity,
        "long route updates should reuse each retained string buffer"
    );
    assert_eq!(
        values[1].capacity(),
        second_long_capacity,
        "long route updates should reuse each retained string buffer"
    );

    update_hit_route_key(&mut cached, Some(&["again", "route", "third"]));
    let values = cached.as_ref().expect("grown route key remains stored");
    assert_eq!(values.len(), 3);
    assert_eq!(values[2], "third");
    assert!(values.capacity() >= 3);

    update_hit_route_key(&mut cached, Some(&["short"]));
    let values = cached.as_ref().expect("shrunk route key remains stored");
    assert_eq!(values.len(), 1);
    assert_eq!(values[0], "short");

    update_hit_route_key(&mut cached, None);
    assert!(cached.is_none());
}

#[test]
fn hit_control_remap_updates_signature_in_place_and_reports_visits() {
    let mut signature = signature_with_hit_controls(&[Some("old"), None]);
    let (changed, visits) = signature.remap_hit_control_ids(&mut |control_id| {
        Some(match control_id {
            "projection-0" => "new".to_owned(),
            _ => "second".to_owned(),
        })
    });
    assert!(changed);
    assert_eq!(visits, 2);
    assert_eq!(signature.nodes[0].hit_control_id.as_deref(), Some("new"));
    assert_eq!(signature.nodes[1].hit_control_id.as_deref(), Some("second"));

    let (changed_again, visits_again) = signature.remap_hit_control_ids(&mut |control_id| {
        Some(match control_id {
            "projection-0" => "new".to_owned(),
            _ => "second".to_owned(),
        })
    });
    assert!(!changed_again);
    assert_eq!(visits_again, 2);
}
