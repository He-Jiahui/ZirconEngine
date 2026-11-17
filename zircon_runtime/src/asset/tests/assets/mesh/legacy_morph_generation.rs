use super::*;

const O: [f32; 3] = [0.0, 0.0, 0.0];
const X: [f32; 3] = [1.0, 0.0, 0.0];
const Y: [f32; 3] = [0.0, 1.0, 0.0];
const Z: [f32; 3] = [0.0, 0.0, 1.0];
const UV0: [f32; 2] = [0.0, 0.0];
const U: [f32; 2] = [1.0, 0.0];
const V: [f32; 2] = [0.0, 1.0];

fn collapsed_mesh(normals: &[[f32; 3]], uvs: &[[f32; 2]]) -> MeshAsset {
    MeshAsset::new(
        AssetUri::parse("res://meshes/legacy-morph-handoff.zmesh").unwrap(),
        RenderMeshTopology::TriangleList,
        BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![O; normals.len()]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(normals.to_vec()),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(uvs.to_vec()),
            ),
        ]),
        None,
    )
    .unwrap()
}

fn add_target(mesh: &mut MeshAsset, target: &[[f32; 3]]) {
    // The base positions are all zero, so target positions are also deltas.
    mesh.morph_targets.push(MeshMorphTargetAsset {
        name: None,
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(target.to_vec()),
        )]),
    });
}

fn base_tangents(mesh: &MeshAsset) -> &[[f32; 4]] {
    mesh.attributes[MESH_ATTRIBUTE_TANGENT]
        .as_float32x4()
        .unwrap()
}

#[test]
fn legacy_generation_preserves_expanded_direction_with_either_normal_policy() {
    let mut mesh = collapsed_mesh(&[Z; 3], &[UV0, V, U]);
    add_target(&mut mesh, &[O, X, Y]);
    add_target(&mut mesh, &[O, [2.0, 0.0, 0.0], Y]);
    let targets = mesh.morph_targets.clone();
    assert!(mesh.try_generate_missing_tangents().unwrap());
    assert_eq!(
        mesh.morph_targets, targets,
        "the first call only generates the base stream"
    );
    eprintln!("MORPH_LEGACY_GENERATION_W {}", base_tangents(&mesh)[0][3]);

    for flat in [false, true] {
        let mut rebound = mesh.clone();
        let result =
            rebound.try_rebuild_morph_tangent_frames_for_uv(flat, Some(MESH_ATTRIBUTE_UV0));
        eprintln!("MORPH_LEGACY_EXPANDED_ADMISSION flat={flat} result={result:?}");
        assert_eq!(result, Ok(true));
        for target in &rebound.morph_targets {
            let deltas = target.attributes[MESH_ATTRIBUTE_TANGENT]
                .as_float32x3()
                .unwrap();
            for (base, delta) in base_tangents(&rebound).iter().zip(deltas) {
                let tangent =
                    (Vec3::new(base[0], base[1], base[2]) + Vec3::from_array(*delta)).normalize();
                let bitangent = Vec3::Z.cross(tangent) * base[3];
                eprintln!("MORPH_LEGACY_EXPANDED_FRAME flat={flat} tangent={tangent:?} bitangent={bitangent:?}");
                assert!(tangent.abs_diff_eq(Vec3::Y, 1.0e-6));
                assert!(bitangent.abs_diff_eq(Vec3::X, 1.0e-6));
            }
        }
    }
}

#[test]
fn legacy_generation_leaves_disagreeing_normal_policy_unresolved() {
    // All base corners have no donor. At the target, corner 0 of the still-collapsed
    // probe can copy either donor (same P,UV); N decides which can actually weld.
    // Preserved N=+Z selects the mirrored XY donor. Flat N=+Y selects the regular ZX donor.
    let normals = [Z, Z, Z, Y, Y, Y, Z, Z, Z];
    let uvs = [UV0, [3.0, 0.0], [0.0, 3.0], UV0, U, V, UV0, V, U];
    let mut mesh = collapsed_mesh(&normals, &uvs);
    add_target(&mut mesh, &[O, O, O, O, Z, X, O, X, Y]);
    let target = mesh.morph_targets.clone();
    assert!(mesh.try_generate_missing_tangents().unwrap());
    assert_eq!(target, mesh.morph_targets);
    let basis = base_tangents(&mesh);
    let signs = (basis[0][3], basis[3][3], basis[6][3]);
    eprintln!(
        "MORPH_LEGACY_POLICY_SIGNS probe={} regular={} mirrored={}",
        signs.0, signs.1, signs.2
    );
    assert_eq!(
        signs,
        (1.0, 1.0, -1.0),
        "only the real donors have a shared orientation across both policies"
    );

    let mut flat = mesh.clone();
    assert_eq!(
        flat.try_rebuild_morph_tangent_frames_for_uv(true, Some(MESH_ATTRIBUTE_UV0)),
        Ok(true)
    );
    let mut preserved = mesh.clone();
    assert_eq!(
        preserved.try_rebuild_morph_tangent_frames_for_uv(false, Some(MESH_ATTRIBUTE_UV0)),
        Err(MeshValidationError::MorphTangentHandednessMismatch {
            target_index: 0,
            vertex_index: 0
        }),
    );
    assert_eq!(
        preserved, mesh,
        "a real incompatibility must not publish partial deltas"
    );
}
