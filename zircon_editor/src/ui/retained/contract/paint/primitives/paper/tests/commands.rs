use super::*;

#[test]
fn non_finite_paper_origins_do_not_emit_surface_or_shadow_commands() {
    let node = TemplatePaneNodeData {
        component_role: "paper".to_owned(),
        elevation: 4.0,
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 8.0,
        y: f32::NEG_INFINITY,
        width: 48.0,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_paper_primitive_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0,
    ));
    assert!(commands.is_empty());
}

#[test]
fn fractional_paper_elevation_emits_fractional_overlay_alpha() {
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 48.0,
        height: 24.0,
    };
    for (elevation, expected_alpha) in [(0.0, None), (0.5, Some(3_u8)), (1.0, Some(13_u8))] {
        let node = TemplatePaneNodeData {
            component_role: "paper".to_owned(),
            elevation,
            ..TemplatePaneNodeData::default()
        };
        let mut commands = Vec::new();
        assert!(push_paper_primitive_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            10,
            1.0
        ));
        assert!(commands.iter().any(|command| command.z_index == 10));
        let overlays = commands
            .iter()
            .filter(|command| command.z_index == 11)
            .collect::<Vec<_>>();
        if let Some(alpha) = expected_alpha {
            assert_eq!(overlays.len(), 1);
            assert_eq!(overlays[0].background_color, Some([255, 255, 255, alpha]));
            assert_eq!(overlays[0].opacity, 1.0);
        } else {
            assert!(
                overlays.is_empty(),
                "zero elevation must not introduce an overlay"
            );
        }
    }
}
