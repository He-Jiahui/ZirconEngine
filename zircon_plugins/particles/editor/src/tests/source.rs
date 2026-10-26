use super::{
    parse_particle_seed, ParticleCurveChannel, ParticleOverviewConnectionEligibility,
    ParticleOverviewNodeId, ParticleSourceCurveAdapter, ParticleSourceCurveEdit,
    ParticleSourceCurveId, ParticleSourceParameterEdit, ParticleSourceScalarRange,
    ParticleSystemSource,
};
use zircon_plugin_particles_runtime::{ParticleShape, ParticleSystemComponent, ParticlesManager};

#[path = "source_burst.rs"]
mod bursts;
#[path = "source_curve.rs"]
mod curves;

const TEMPLATE: &str = include_str!("../../../templates/cpu_sprite_system.toml");

#[test]
fn existing_cpu_template_decodes_lowers_and_round_trips() {
    let source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let asset = source.to_cpu_asset().unwrap();
    assert_eq!(asset.id, "cpu_sprite_system");
    assert_eq!(asset.seed, 12648430);
    assert_eq!(asset.emitters[0].max_particles, 256);
    assert_eq!(asset.emitters[0].spawn_rate_per_second, 32.0);
    assert_eq!(asset.emitters[0].lifetime.max, 1.5);
    assert_eq!(
        asset.emitters[0].shape,
        ParticleShape::Sphere { radius: 0.1 }
    );
    assert_eq!(asset.emitters[0].color_over_lifetime[1].value.w, 0.0);
    assert_eq!(asset.emitters[0].size_over_lifetime[1].value, 0.0);
    let reopened = ParticleSystemSource::from_toml(&source.to_toml().unwrap()).unwrap();
    assert_eq!(reopened, source);
    assert_eq!(reopened.to_cpu_asset().unwrap(), asset);
}

#[test]
fn exact_u64_seed_survives_a_text_round_trip() {
    let input = TEMPLATE.replace("seed = 12648430", "seed = \"18446744073709551615\"");
    let source = ParticleSystemSource::from_toml(&input).unwrap();
    assert_eq!(source.system.seed, u64::MAX);
    let output = source.to_toml().unwrap();
    assert!(output.contains("seed = \"18446744073709551615\""));
    assert_eq!(
        ParticleSystemSource::from_toml(&output)
            .unwrap()
            .system
            .seed,
        u64::MAX
    );
    for invalid in ["-1", "1.5", "\"+1\"", "\" 1\"", "\"18446744073709551616\""] {
        assert!(ParticleSystemSource::from_toml(&TEMPLATE.replace("12648430", invalid)).is_err());
    }
}

#[test]
fn identity_survives_rename_reorder_and_key_time_edits() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let mut second = source.emitters[0].clone();
    second.id = "second".into();
    second.key = None;
    for burst in &mut second.bursts {
        burst.key = None;
    }
    for key in &mut second.color_over_lifetime {
        key.key = None;
    }
    for key in &mut second.size_over_lifetime {
        key.key = None;
    }
    source.emitters.push(second);
    source.assign_missing_keys().unwrap();
    let first_key = source.emitters[0].key.clone();
    let color_key = source.emitters[0].color_over_lifetime[0].key.clone();
    source.emitters[0].display_name = Some("长名称／发射器 · emitter with a long label".into());
    source.emitters[0].color_over_lifetime[0].t = 0.1;
    source.emitters.swap(0, 1);
    let reopened = ParticleSystemSource::from_toml(&source.to_toml().unwrap()).unwrap();
    assert_eq!(reopened.emitters[1].key, first_key);
    assert_eq!(reopened.emitters[1].color_over_lifetime[0].key, color_key);
    assert_eq!(
        reopened.emitters[1].display_name,
        source.emitters[1].display_name
    );
    assert_eq!(reopened.emitters[1].id, "sprite_emitter");
}

#[test]
fn duplicate_or_missing_keys_cannot_be_committed() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters[0].size_over_lifetime[0].key = source.emitters[0].key.clone();
    let before = source.clone();
    assert!(source.assign_missing_keys().is_err());
    assert_eq!(source, before);
    assert!(source.to_toml().is_err());
    source.emitters[0].size_over_lifetime[0].key = None;
    assert!(source.to_toml().is_err());
    source.assign_missing_keys().unwrap();
    assert!(source.to_toml().is_ok());
}

#[test]
fn legacy_key_adoption_is_reported_until_encoded_source_is_decoded() {
    let adopted = ParticleSystemSource::decode_toml(TEMPLATE).unwrap();
    assert!(adopted.requires_identity_persistence());
    assert_eq!(adopted.assigned_keys, 5);
    let encoded = adopted.source.to_toml().unwrap();
    // Encoding is an in-memory operation; it does not clear the live document's adoption debt.
    assert!(adopted.requires_identity_persistence());
    let separately_decoded_legacy = ParticleSystemSource::decode_toml(TEMPLATE).unwrap();
    assert!(separately_decoded_legacy.requires_identity_persistence());
    assert_ne!(
        separately_decoded_legacy.source.emitters[0].key,
        adopted.source.emitters[0].key
    );
    // This only proves codec identity round-trip, not a file write or durable save receipt.
    let reopened = ParticleSystemSource::decode_toml(&encoded).unwrap();
    assert!(!reopened.requires_identity_persistence());
    assert_eq!(reopened.assigned_keys, 0);
    assert_eq!(reopened.source, adopted.source);
}

#[test]
fn a_new_curve_key_does_not_reuse_a_deleted_items_identity() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let deleted = source.emitters[0].size_over_lifetime.remove(0);
    let mut replacement = deleted.clone();
    replacement.key = None;
    source.emitters[0].size_over_lifetime.insert(0, replacement);
    assert_eq!(source.assign_missing_keys().unwrap(), 1);
    assert_ne!(source.emitters[0].size_over_lifetime[0].key, deleted.key);
    let reopened = ParticleSystemSource::decode_toml(&source.to_toml().unwrap()).unwrap();
    assert_eq!(reopened.assigned_keys, 0);
    assert_eq!(reopened.source, source);
}

#[test]
fn invalid_source_never_uses_the_zero_capacity_validation_bypass() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters[0].max_particles = 0;
    source.emitters[0].drag = zircon_runtime::core::math::Real::NAN;
    assert!(source.to_toml().is_err());
    assert!(source.to_cpu_asset().is_err());
    source.emitters[0].drag = 0.0;
    source.emitters[0].initial_velocity.min[0] = -10.0;
    source.emitters[0].gravity[0] = -9.8;
    source.emitters[0].initial_rotation.min = -2.0;
    assert!(source.to_cpu_asset().is_ok());
    source.emitters[0].color_over_lifetime[1].t = source.emitters[0].color_over_lifetime[0].t;
    assert!(source.validate().is_err());
}

#[test]
fn empty_authoring_and_unknown_fields_are_distinct() {
    let empty = ParticleSystemSource::from_toml("[system]\nid = \"empty\"\n").unwrap();
    assert!(empty.emitters.is_empty());
    assert_eq!(
        ParticleSystemSource::from_toml(&empty.to_toml().unwrap()).unwrap(),
        empty
    );
    assert!(ParticleSystemSource::from_toml(&format!("unknown = true\n{TEMPLATE}")).is_err());
    assert!(ParticleSystemSource::from_toml(&format!("format_version = 99\n{TEMPLATE}")).is_err());
}

#[test]
fn resource_locators_are_preserved_and_unready_resources_are_rejected() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters[0].texture = Some("res://textures/spark.png".into());
    let reopened = ParticleSystemSource::from_toml(&source.to_toml().unwrap()).unwrap();
    assert_eq!(reopened.emitters[0].texture, source.emitters[0].texture);
    assert!(reopened
        .to_cpu_asset()
        .unwrap_err()
        .to_string()
        .contains("resource is not ready"));
}

#[test]
fn decoded_parameters_drive_real_cpu_simulation_pause_and_reset() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters[0].spawn_rate_per_second = 20.0;
    let manager = ParticlesManager::default();
    let handle = manager
        .instantiate(ParticleSystemComponent::new(
            1,
            source.to_cpu_asset().unwrap(),
        ))
        .unwrap();
    manager.tick(0.1).unwrap();
    assert_eq!(manager.snapshot().emitters[0].live_particles, 2);
    manager.pause(handle).unwrap();
    let paused = manager.snapshot().sprites;
    manager.tick(0.1).unwrap();
    assert_eq!(manager.snapshot().sprites, paused);
    manager.stop(handle).unwrap();
    assert!(manager.snapshot().sprites.is_empty());
    source.emitters[0].spawn_rate_per_second = 40.0;
    let updated = ParticlesManager::default();
    updated
        .instantiate(ParticleSystemComponent::new(
            1,
            source.to_cpu_asset().unwrap(),
        ))
        .unwrap();
    updated.tick(0.1).unwrap();
    assert_eq!(updated.snapshot().emitters[0].live_particles, 4);
}

#[test]
fn seed_field_input_is_exact_decimal_text_and_parameter_inverse_restores_it() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    assert_eq!(
        parse_particle_seed("9007199254740993").unwrap(),
        9007199254740993
    );
    let edit = ParticleSourceParameterEdit::Seed {
        expected: source.system.seed,
        replacement: parse_particle_seed("18446744073709551615").unwrap(),
    };
    let inverse = source.apply_parameter_edit(edit.clone()).unwrap();
    assert_eq!(source.system.seed, u64::MAX);
    let encoded = source.to_toml().unwrap();
    assert!(encoded.contains("seed = \"18446744073709551615\""));
    for input in [
        "",
        "-1",
        "+1",
        " 1",
        "1 ",
        "1.0",
        "1e3",
        "１２",
        "18446744073709551616",
    ] {
        assert!(parse_particle_seed(input).is_err());
        assert_eq!(source.system.seed, u64::MAX);
    }
    assert_eq!(source.apply_parameter_edit(inverse).unwrap(), edit);
    assert_eq!(source, original);
}

#[test]
fn typed_spawn_and_capacity_edits_change_real_cpu_output_and_restore_source() {
    fn live_particles(source: &ParticleSystemSource) -> usize {
        let manager = ParticlesManager::default();
        manager
            .instantiate(ParticleSystemComponent::new(
                1,
                source.to_cpu_asset().unwrap(),
            ))
            .unwrap();
        manager.tick(0.1).unwrap();
        manager.snapshot().emitters[0].live_particles
    }

    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters[0].spawn_rate_per_second = 20.0;
    let original = source.clone();
    let selected = source.emitters[0].key.clone().unwrap();
    assert_eq!(live_particles(&source), 2);
    let undo_rate = source
        .apply_parameter_edit(ParticleSourceParameterEdit::SpawnRate {
            emitter_key: selected.clone(),
            expected: 20.0,
            replacement: 40.0,
        })
        .unwrap();
    assert_eq!(live_particles(&source), 4);
    let undo_large_capacity = source
        .apply_parameter_edit(ParticleSourceParameterEdit::Capacity {
            emitter_key: selected.clone(),
            expected: 256,
            replacement: 4096,
        })
        .unwrap();
    assert_eq!(
        source.to_cpu_asset().unwrap().emitters[0].max_particles,
        4096
    );
    let reopened = ParticleSystemSource::from_toml(&source.to_toml().unwrap()).unwrap();
    assert_eq!(reopened.emitters[0].max_particles, 4096);
    source.apply_parameter_edit(undo_large_capacity).unwrap();
    let undo_capacity = source
        .apply_parameter_edit(ParticleSourceParameterEdit::Capacity {
            emitter_key: selected,
            expected: 256,
            replacement: 1,
        })
        .unwrap();
    assert_eq!(live_particles(&source), 1);
    source.apply_parameter_edit(undo_capacity).unwrap();
    assert_eq!(live_particles(&source), 4);
    source.apply_parameter_edit(undo_rate).unwrap();
    assert_eq!(live_particles(&source), 2);
    assert_eq!(source, original);
}

#[test]
fn lifetime_range_changes_both_endpoints_atomically_and_inverse_restores_them() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let expected = source.emitters[0].lifetime;
    let edit = ParticleSourceParameterEdit::Lifetime {
        emitter_key: source.emitters[0].key.clone().unwrap(),
        expected,
        replacement: ParticleSourceScalarRange { min: 2.0, max: 3.0 },
    };
    let inverse = source.apply_parameter_edit(edit.clone()).unwrap();
    let runtime = source.to_cpu_asset().unwrap();
    assert_eq!(runtime.emitters[0].lifetime.min, 2.0);
    assert_eq!(runtime.emitters[0].lifetime.max, 3.0);
    assert_eq!(source.apply_parameter_edit(inverse).unwrap(), edit);
    assert_eq!(source, original);
}

#[test]
fn invalid_stale_or_missing_parameter_targets_leave_the_source_unchanged() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let selected = source.emitters[0].key.clone().unwrap();
    for value in [-1.0, f32::NAN, f32::INFINITY] {
        let edit = ParticleSourceParameterEdit::SpawnRate {
            emitter_key: selected.clone(),
            expected: source.emitters[0].spawn_rate_per_second,
            replacement: value,
        };
        assert!(source.apply_parameter_edit(edit).is_err());
        assert_eq!(source, original);
    }
    for replacement in [
        ParticleSourceScalarRange { min: 0.0, max: 1.0 },
        ParticleSourceScalarRange { min: 3.0, max: 2.0 },
        ParticleSourceScalarRange {
            min: f32::NAN,
            max: 2.0,
        },
        ParticleSourceScalarRange {
            min: 1.0,
            max: f32::INFINITY,
        },
    ] {
        let edit = ParticleSourceParameterEdit::Lifetime {
            emitter_key: selected.clone(),
            expected: source.emitters[0].lifetime,
            replacement,
        };
        assert!(source.apply_parameter_edit(edit).is_err());
        assert_eq!(source, original);
    }
    for edit in [
        ParticleSourceParameterEdit::Seed {
            expected: 0,
            replacement: 1,
        },
        ParticleSourceParameterEdit::SpawnRate {
            emitter_key: selected.clone(),
            expected: 0.0,
            replacement: 1.0,
        },
        ParticleSourceParameterEdit::Capacity {
            emitter_key: selected.clone(),
            expected: 0,
            replacement: 1,
        },
        ParticleSourceParameterEdit::Lifetime {
            emitter_key: selected,
            expected: ParticleSourceScalarRange { min: 2.0, max: 2.0 },
            replacement: ParticleSourceScalarRange { min: 1.0, max: 1.0 },
        },
        ParticleSourceParameterEdit::Capacity {
            emitter_key: "deleted emitter".into(),
            expected: 256,
            replacement: 1,
        },
    ] {
        assert!(source.apply_parameter_edit(edit).is_err());
        assert_eq!(source, original);
    }
}

#[test]
fn parameter_noop_and_zero_capacity_preserve_validation_and_identity() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters[0].max_particles = 0;
    let original = source.clone();
    let edit = ParticleSourceParameterEdit::Capacity {
        emitter_key: source.emitters[0].key.clone().unwrap(),
        expected: 0,
        replacement: 0,
    };
    assert!(edit.is_noop());
    assert!(source.apply_parameter_edit(edit).unwrap().is_noop());
    assert_eq!(source, original);
    source.emitters[0].drag = -1.0;
    let invalid = source.clone();
    let seed_edit = ParticleSourceParameterEdit::Seed {
        expected: source.system.seed,
        replacement: 0,
    };
    assert!(source.apply_parameter_edit(seed_edit).is_err());
    assert_eq!(source, invalid);
}

#[test]
fn parameter_preflight_is_read_only_and_execution_rechecks_the_live_source() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let key = source.emitters[0].key.clone().unwrap();
    let first = ParticleSourceParameterEdit::SpawnRate {
        emitter_key: key.clone(),
        expected: 32.0,
        replacement: 48.0,
    };
    source.validate_parameter_edit(&first).unwrap();
    assert_eq!(source, original);
    let intervening = ParticleSourceParameterEdit::SpawnRate {
        emitter_key: key,
        expected: 32.0,
        replacement: 64.0,
    };
    source.apply_parameter_edit(intervening).unwrap();
    let newer = source.clone();
    assert!(source.validate_parameter_edit(&first).is_err());
    assert!(source.apply_parameter_edit(first).is_err());
    assert_eq!(source, newer);

    let seed = ParticleSourceParameterEdit::seed_from_decimal("12648430", "0").unwrap();
    source.emitters[0].drag = -1.0;
    let invalid = source.clone();
    assert!(source.validate_parameter_edit(&seed).is_err());
    assert_eq!(source, invalid);
}

#[test]
fn seed_input_factory_parses_both_strings_without_losing_u64_precision() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    for replacement in ["9007199254740993", "18446744073709551615"] {
        let edit = ParticleSourceParameterEdit::seed_from_decimal("12648430", replacement).unwrap();
        source.validate_parameter_edit(&edit).unwrap();
        let inverse = source.apply_parameter_edit(edit).unwrap();
        assert_eq!(
            source.system.seed,
            parse_particle_seed(replacement).unwrap()
        );
        source.apply_parameter_edit(inverse).unwrap();
        assert_eq!(source.system.seed, 12648430);
    }
    for invalid in ["18446744073709551616", " 1", "+1", "1.0", "１", ""] {
        assert!(ParticleSourceParameterEdit::seed_from_decimal(invalid, "0").is_err());
        assert!(ParticleSourceParameterEdit::seed_from_decimal("12648430", invalid).is_err());
    }
}

fn overview_layout(
    source: &ParticleSystemSource,
) -> std::collections::BTreeMap<ParticleOverviewNodeId, zircon_editor::ui::graph::GraphNodeBounds> {
    use zircon_editor::ui::graph::{GraphNodeBounds, GraphPoint};
    std::iter::once(ParticleOverviewNodeId::System)
        .chain(
            source
                .emitters
                .iter()
                .map(|emitter| ParticleOverviewNodeId::Emitter(emitter.key.clone().unwrap())),
        )
        .enumerate()
        .map(|(index, id)| {
            (
                id,
                GraphNodeBounds::new(
                    GraphPoint::new(index as f32 * 280.0, 0.0),
                    GraphPoint::new(240.0, 80.0),
                ),
            )
        })
        .collect()
}

#[test]
fn particle_overview_real_source_consumes_shared_presenter_without_ports_or_edges() {
    use zircon_editor::ui::graph::{
        default_connection_verdict, required_input_diagnostics, ConnectVerdict,
        GraphConnectRejection, GraphEdgeView, GraphNodePresentation, GraphPortRef, GraphSelection,
        StructureConstraint,
    };
    let source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let graph = source.project_overview(&overview_layout(&source)).unwrap();
    assert_eq!(graph.nodes().len(), 2);
    assert_eq!(graph.nodes()[0].display_name, source.system.id);
    assert_eq!(graph.nodes()[1].display_name, source.emitters[0].id);
    assert!(graph.edges().is_empty());
    for node in graph.nodes() {
        let view = GraphNodePresentation::from_view(node, &GraphSelection::default());
        assert_eq!((view.input_count, view.output_count), (0, 0));
    }
    assert!(required_input_diagnostics(graph.nodes(), graph.edges()).is_empty());
    assert_eq!(
        graph.connection_eligibility(),
        ParticleOverviewConnectionEligibility::UnavailableNoSourcePortsOrEdges
    );
    assert!(!graph.connection_eligibility().reason().is_empty());
    let forged = GraphEdgeView::new(
        GraphPortRef::output(graph.nodes()[0].id.clone(), "invented"),
        GraphPortRef::input(graph.nodes()[1].id.clone(), "invented"),
    );
    assert_eq!(
        default_connection_verdict(
            graph.nodes(),
            graph.edges(),
            &forged,
            StructureConstraint::FreeGraph
        ),
        ConnectVerdict::Rejected(GraphConnectRejection::MissingPin)
    );
    assert_eq!(source, original);
}

#[test]
fn particle_overview_ready_empty_system_does_not_invent_an_emitter() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters.clear();
    let graph = source.project_overview(&overview_layout(&source)).unwrap();
    assert_eq!(graph.nodes().len(), 1);
    assert_eq!(graph.nodes()[0].id, ParticleOverviewNodeId::System);
    assert!(graph.nodes()[0].inputs.is_empty());
    assert!(graph.nodes()[0].outputs.is_empty());
    assert!(graph.edges().is_empty());
}

#[test]
fn particle_overview_identity_and_measured_layout_survive_rename_and_reorder() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let mut second = source.emitters[0].clone();
    second.key = Some("second-stable-key".to_owned());
    second.id = "second-emitter".to_owned();
    source.emitters.push(second);
    let layout = overview_layout(&source);
    let before = source.project_overview(&layout).unwrap();
    source.emitters.reverse();
    source.system.id = "renamed-system".to_owned();
    source.emitters[0].id = "renamed-emitter".to_owned();
    let after = source.project_overview(&layout).unwrap();
    assert_eq!(after.nodes()[0].id, before.nodes()[0].id);
    assert_eq!(after.nodes()[1].id, before.nodes()[2].id);
    assert_eq!(after.nodes()[2].id, before.nodes()[1].id);
    for node in after.nodes() {
        assert_eq!(node.bounds, layout[&node.id]);
    }
    assert_eq!(after.nodes()[0].display_name, "renamed-system");
    assert_eq!(after.nodes()[1].display_name, "renamed-emitter");
}

#[test]
fn particle_overview_rejects_missing_identity_and_invalid_layout_without_source_mutation() {
    let source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let layout = overview_layout(&source);
    let mut missing = layout.clone();
    missing.remove(&ParticleOverviewNodeId::System);
    assert!(source.project_overview(&missing).is_err());
    for invalid in [f32::NAN, f32::INFINITY, 0.0, -1.0] {
        let mut malformed = layout.clone();
        malformed
            .get_mut(&ParticleOverviewNodeId::System)
            .unwrap()
            .size
            .x = invalid;
        assert!(source.project_overview(&malformed).is_err());
    }
    let mut overflow = layout.clone();
    {
        let bounds = overflow.get_mut(&ParticleOverviewNodeId::System).unwrap();
        bounds.origin.x = f32::MAX;
        bounds.size.x = f32::MAX;
    }
    assert!(source.project_overview(&overflow).is_err());
    overflow
        .get_mut(&ParticleOverviewNodeId::System)
        .unwrap()
        .size
        .x = 1.0;
    assert!(source.project_overview(&overflow).is_err());
    assert_eq!(source, original);
    let mut unkeyed = source.clone();
    unkeyed.emitters[0].key = None;
    assert!(unkeyed.project_overview(&layout).is_err());
    assert!(unkeyed.emitters[0].key.is_none());
    let mut duplicate = source.clone();
    duplicate.emitters.push(duplicate.emitters[0].clone());
    assert!(duplicate.project_overview(&layout).is_err());
    assert_eq!(duplicate.emitters.len(), 2);
}
