#![cfg(feature = "runtime")]

use std::path::PathBuf;

use zircon_plugin_gltf_importer_runtime::{asset_importer_descriptors, import_gltf};
use zircon_runtime::asset::{AssetImportContext, AssetUri, ImportedAsset, MESH_ATTRIBUTE_TANGENT};

// Same triangle and effective UV set as the existing geometry convergence
// case, with in-memory companion data so the production entry needs no files.
// Positions (0,+X,+Y), UV0 all zero, UV1 (0,+U,+V), indices (0,1,2).
const UV1_TRIANGLE: &[u8] = br#"{
  "asset": { "version": "2.0" },
  "extensionsUsed": ["KHR_texture_transform"],
  "buffers": [{ "uri": "data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIA/AAAAAAAAAAAAAIA/AAABAAIA", "byteLength": 90 }],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 60, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 84, "byteLength": 6, "target": 34963 }
  ],
  "accessors": [
    { "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0,0,0], "max": [1,1,0] },
    { "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC2" },
    { "bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2" },
    { "bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR" }
  ],
  "images": [{ "uri": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg==" }],
  "textures": [{ "source": 0 }],
  "materials": [{ "normalTexture": {
    "index": 0, "texCoord": 0,
    "extensions": { "KHR_texture_transform": { "texCoord": 1 } }
  } }],
  "meshes": [{ "primitives": [{
    "attributes": { "POSITION": 0, "TEXCOORD_0": 1, "TEXCOORD_1": 2 },
    "indices": 3, "material": 0
  }] }],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0
}"#;

#[test]
fn stable_gltf_import_preserves_the_generated_effective_uv1_basis() {
    let uri = AssetUri::parse("res://meshes/normal-uv1.gltf").unwrap();
    let imported = import_gltf(&AssetImportContext::new(
        PathBuf::from("normal-uv1.gltf"),
        uri,
        UV1_TRIANGLE.to_vec(),
        toml::Table::new(),
    ))
    .unwrap();
    let mesh = imported
        .entries
        .iter()
        .find_map(|entry| match &entry.asset {
            ImportedAsset::Mesh(mesh) if entry.locator.label() == Some("Mesh0/Primitive0") => {
                Some(mesh)
            }
            _ => None,
        })
        .expect("the labelled mesh subasset owns the tangent stream");
    let tangents = mesh.attributes[MESH_ATTRIBUTE_TANGENT]
        .as_float32x4()
        .unwrap();
    assert_eq!(
        tangents,
        &[[1.0, 0.0, 0.0, 1.0]; 3],
        "effective UV1 must reconstruct the +V direction under the flat +Z normal"
    );
}

#[test]
fn stable_gltf_output_version_matches_the_manifest_projection() {
    let descriptors = asset_importer_descriptors();
    let [descriptor] = descriptors.as_slice() else {
        panic!("expected the one glTF import descriptor")
    };
    assert_eq!(descriptor.id, "gltf_importer.gltf");
    assert_eq!(
        descriptor.importer_version, 5,
        "generated tangent output changed"
    );
    let manifest: toml::Table = include_str!("../../plugin.toml").parse().unwrap();
    let row = &manifest["asset_importers"].as_array().unwrap()[0];
    assert_eq!(row["id"].as_str(), Some(descriptor.id.as_str()));
    assert_eq!(
        row["importer_version"].as_integer(),
        Some(i64::from(descriptor.importer_version))
    );
}

mod mirrored_morph_endpoints {
    use std::path::PathBuf;

    use zircon_plugin_gltf_importer_runtime::import_gltf;
    use zircon_runtime::asset::{
        AssetImportContext, AssetImportError, AssetImportOutcome, AssetUri, ImportedAsset,
        MeshAsset, MeshValidationError, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION,
        MESH_ATTRIBUTE_UV0,
    };

    const EXPAND: &[u8] = include_bytes!(
        "../../../../zircon_runtime/src/asset/tests/fixtures/gltf/mirrored_morph_endpoints/expand.gltf"
    );
    const COLLAPSE: &[u8] = include_bytes!(
        "../../../../zircon_runtime/src/asset/tests/fixtures/gltf/mirrored_morph_endpoints/collapse.gltf"
    );
    const CONFLICT: &[u8] = include_bytes!(
        "../../../../zircon_runtime/src/asset/tests/fixtures/gltf/mirrored_morph_endpoints/conflicting_targets.gltf"
    );
    const SAME_HANDED: &[u8] = include_bytes!(
        "../../../../zircon_runtime/src/asset/tests/fixtures/gltf/mirrored_morph_endpoints/same_handed_targets.gltf"
    );
    const ZERO: [f32; 3] = [0.0, 0.0, 0.0];
    const X: [f32; 3] = [1.0, 0.0, 0.0];
    const Y: [f32; 3] = [0.0, 1.0, 0.0];
    const Z: [f32; 3] = [0.0, 0.0, 1.0];
    const OPEN: [[f32; 3]; 3] = [ZERO, X, Y];
    const MIRRORED_UVS: [[f32; 2]; 3] = [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]];

    #[test]
    fn stable_import_expands_coincident_base_into_the_actual_mirrored_basis() {
        assert_endpoint("expand.gltf", EXPAND, [ZERO; 3], OPEN, &[1.0], &[]);
    }

    #[test]
    fn stable_import_preserves_the_healthy_mirrored_base_when_the_target_collapses() {
        assert_endpoint(
            "collapse.gltf",
            COLLAPSE,
            OPEN,
            [ZERO, [-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]],
            &[],
            &[1.0],
        );
    }

    #[test]
    fn stable_import_reports_the_second_valid_opposite_handedness_at_shared_vertex_zero() {
        let source = read_source("conflicting_targets.gltf", CONFLICT);
        assert_shared_targets(&source);
        assert_eq!(
            source.uvs,
            [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 1.0], [1.0, 0.0]]
        );
        assert_eq!(target_frame(&source, 0, [0, 1, 2]), (X, Y, 1.0));
        assert_eq!(target_frame(&source, 1, [0, 3, 4]), (Y, X, -1.0));

        // Only one triangle is healthy in each target. At shared vertex 0 the
        // degenerate corner can borrow that target's healthy result (Mikk 1.0.0
        // degenerate epilogue, mikktspace.rs:843+); it is not a second constraint.
        // The two genuinely valid constraints at vertex 0 nevertheless disagree.
        // Flipping a position axis alone would NOT do this: Mikk's orientation
        // flag is selected by the fixed UV determinant (mikktspace.rs:345-355).
        let error = import_fixture("conflicting_targets.gltf", CONFLICT)
            .expect_err("opposite valid target signs cannot share one glTF tangent w");
        let AssetImportError::Parse(message) = error else {
            panic!("expected the morph-frame diagnostic, got {error:?}")
        };
        let actual_conflict = MeshValidationError::MorphTangentHandednessMismatch {
            target_index: 1,
            vertex_index: 0,
        };
        assert!(
            message.contains("Mesh0/Primitive0")
                && message.ends_with(actual_conflict.to_string().as_str()),
            "expected the second target's genuine conflict, not a fallback or other early error: {message}"
        );
    }

    #[test]
    fn stable_import_accepts_two_independent_same_handed_targets_and_their_healthy_frames() {
        let source = read_source("same_handed_targets.gltf", SAME_HANDED);
        assert_shared_targets(&source);
        assert_eq!(
            source.uvs,
            [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [0.0, 1.0], [1.0, 0.0]]
        );
        let outcome = import_fixture("same_handed_targets.gltf", SAME_HANDED).unwrap();
        let mesh = imported_mesh(&outcome);
        assert_asset_matches_source(mesh, &source);
        assert_imported_normal_texel(&outcome);
        for (target_index, triangle) in [[0, 1, 2], [0, 3, 4]].into_iter().enumerate() {
            let expected = target_frame(&source, target_index, triangle);
            assert_eq!(expected, (Y, X, -1.0));
            let mut weights = [0.0; 2];
            weights[target_index] = 1.0;
            let primitive = mesh.to_morphed_model_primitive(&weights).unwrap();
            assert_eq!(primitive.indices, source.indices);
            for (vertex_index, vertex) in primitive.vertices.iter().enumerate() {
                assert_eq!(vertex.position, source.deltas[target_index][vertex_index]);
            }
            for vertex_index in triangle {
                let vertex = &primitive.vertices[vertex_index];
                assert_basis(vertex.normal, vertex.tangent, expected);
            }
        }
    }

    fn assert_endpoint(
        name: &str,
        bytes: &[u8],
        base: [[f32; 3]; 3],
        delta: [[f32; 3]; 3],
        healthy_weights: &[f32],
        collapsed_weights: &[f32],
    ) {
        let source = read_source(name, bytes);
        assert_eq!(source.positions, base);
        assert_eq!(source.uvs, MIRRORED_UVS);
        assert_eq!(source.indices, [0, 1, 2]);
        assert_eq!(source.deltas, [delta.to_vec()]);

        let outcome = import_fixture(name, bytes).unwrap();
        let mesh = imported_mesh(&outcome);
        assert_asset_matches_source(mesh, &source);
        assert_imported_normal_texel(&outcome);
        let healthy = mesh.to_morphed_model_primitive(healthy_weights).unwrap();
        assert_eq!(healthy.vertices.len(), 3);
        assert_eq!(healthy.indices, [0, 1, 2]);
        let expected = triangle_frame(OPEN, MIRRORED_UVS);
        // Solve dP/dU and dP/dV from the source, not a tangent default or another
        // importer. det(UV)=-1 gives T=+Y, B=+X, and cross(+Z,+Y)*w=+X => w=-1.
        assert_eq!(expected, (Y, X, -1.0));
        for (vertex, position) in healthy.vertices.iter().zip(OPEN) {
            assert_eq!(vertex.position, position);
            assert_basis(vertex.normal, vertex.tangent, expected);
        }
        let collapsed = mesh.to_morphed_model_primitive(collapsed_weights).unwrap();
        assert_eq!(collapsed.vertices.len(), 3);
        assert!(collapsed
            .vertices
            .iter()
            .all(|vertex| vertex.position == ZERO));
    }

    struct SourceGeometry {
        positions: Vec<[f32; 3]>,
        uvs: Vec<[f32; 2]>,
        indices: Vec<u32>,
        deltas: Vec<Vec<[f32; 3]>>,
    }

    fn read_source(name: &str, bytes: &[u8]) -> SourceGeometry {
        // This independent glTF decode validates the actual checked-in accessors
        // and PNG, but never substitutes for the stable import_gltf call above.
        // The locked glTF library's slice importer rejects URI images, even data URIs.
        let path = fixture_path(name);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let json: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        assert!(
            json["materials"][0].get("extensions").is_none(),
            "must stay lit"
        );
        let (document, buffers, images) = gltf::import(path).unwrap();
        assert_eq!(document.meshes().count(), 1);
        assert_eq!(document.materials().count(), 1);
        assert_eq!(document.textures().count(), 1);
        for buffer in document.buffers() {
            assert!(matches!(buffer.source(), gltf::buffer::Source::Uri(uri)
                if uri.starts_with("data:application/octet-stream;base64,")));
        }
        assert!(matches!(document.images().next().unwrap().source(),
            gltf::image::Source::Uri { uri, .. } if uri.starts_with("data:image/png;base64,")));
        let [image] = images.as_slice() else {
            panic!("the fixture must embed exactly one real normal-map PNG")
        };
        assert_eq!((image.width, image.height), (1, 1));
        assert_eq!(image.format, gltf::image::Format::R8G8B8A8);
        assert_eq!(image.pixels, [128, 204, 255, 255]);

        let mesh = document.meshes().next().unwrap();
        assert_eq!(mesh.primitives().count(), 1);
        let primitive = mesh.primitives().next().unwrap();
        assert_eq!(primitive.mode(), gltf::mesh::Mode::Triangles);
        assert_eq!(primitive.attributes().count(), 3);
        let material = primitive.material();
        assert_eq!(material.index(), Some(0));
        let pbr = material.pbr_metallic_roughness();
        assert_eq!(pbr.base_color_factor(), [1.0; 4]);
        assert_eq!(pbr.metallic_factor(), 0.0);
        let normal_map = material
            .normal_texture()
            .expect("activate tangent generation");
        assert_eq!(normal_map.texture().index(), 0);
        assert_eq!(
            normal_map.texture().source().map(|image| image.index()),
            Some(0)
        );
        assert_eq!(normal_map.tex_coord(), 0);
        assert_eq!(normal_map.scale(), 1.0);

        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));
        let positions: Vec<_> = reader.read_positions().unwrap().collect();
        let normals: Vec<_> = reader
            .read_normals()
            .expect("explicit +Z normals")
            .collect();
        assert_eq!(normals, vec![Z; positions.len()]);
        assert!(
            reader.read_tangents().is_none(),
            "tangents must be generated"
        );
        let uvs = reader.read_tex_coords(0).unwrap().into_f32().collect();
        let indices = reader.read_indices().map_or_else(
            || (0..positions.len() as u32).collect(),
            |indices| indices.into_u32().collect(),
        );
        let deltas = reader
            .read_morph_targets()
            .map(|(positions, normals, tangents)| {
                assert!(normals.is_none(), "source targets contain only POSITION");
                assert!(tangents.is_none(), "source targets contain only POSITION");
                positions.expect("POSITION morph delta").collect()
            })
            .collect();
        SourceGeometry {
            positions,
            uvs,
            indices,
            deltas,
        }
    }

    fn assert_shared_targets(source: &SourceGeometry) {
        assert_eq!(source.positions, [ZERO; 5]);
        assert_eq!(source.indices, [0, 1, 2, 0, 3, 4]);
        assert_eq!(
            source.deltas,
            [vec![ZERO, X, Y, ZERO, ZERO], vec![ZERO, ZERO, ZERO, X, Y]]
        );
    }

    fn assert_asset_matches_source(mesh: &MeshAsset, source: &SourceGeometry) {
        assert_eq!(mesh.vertex_count().unwrap(), source.positions.len());
        assert_eq!(
            mesh.attributes[MESH_ATTRIBUTE_POSITION]
                .as_float32x3()
                .unwrap(),
            source.positions
        );
        assert_eq!(
            mesh.attributes[MESH_ATTRIBUTE_UV0].as_float32x2().unwrap(),
            source.uvs
        );
        assert_eq!(
            mesh.attributes[MESH_ATTRIBUTE_NORMAL]
                .as_float32x3()
                .unwrap(),
            vec![Z; source.positions.len()]
        );
        assert_eq!(mesh.indices.as_ref().unwrap().to_u32_vec(), source.indices);
        assert_eq!(mesh.morph_targets.len(), source.deltas.len());
        for (target, expected) in mesh.morph_targets.iter().zip(&source.deltas) {
            assert_eq!(
                target.attributes[MESH_ATTRIBUTE_POSITION]
                    .as_float32x3()
                    .unwrap(),
                expected
            );
            assert!(!target.attributes.contains_key(MESH_ATTRIBUTE_NORMAL));
        }
    }

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../zircon_runtime/src/asset/tests/fixtures/gltf/mirrored_morph_endpoints")
            .join(name)
    }

    fn import_fixture(name: &str, bytes: &[u8]) -> Result<AssetImportOutcome, AssetImportError> {
        import_gltf(&AssetImportContext::new(
            fixture_path(name),
            AssetUri::parse(&format!("res://fixtures/mirrored_morph_endpoints/{name}")).unwrap(),
            bytes.to_vec(),
            toml::Table::new(),
        ))
    }

    fn imported_mesh(outcome: &AssetImportOutcome) -> &MeshAsset {
        outcome
            .entries
            .iter()
            .find_map(|entry| match &entry.asset {
                ImportedAsset::Mesh(mesh) if entry.locator.label() == Some("Mesh0/Primitive0") => {
                    Some(mesh)
                }
                _ => None,
            })
            .expect("stable import must publish the labelled Mesh0/Primitive0 owner")
    }

    fn assert_imported_normal_texel(outcome: &AssetImportOutcome) {
        let mut textures = outcome
            .entries
            .iter()
            .filter_map(|entry| match &entry.asset {
                ImportedAsset::Texture(texture) => Some(texture),
                _ => None,
            });
        let texture = textures
            .next()
            .expect("the material's real normal texture must import");
        assert_eq!((texture.width, texture.height), (1, 1));
        assert_eq!(texture.rgba, [128, 204, 255, 255]);
        assert!(textures.next().is_none());
    }

    fn target_frame(
        source: &SourceGeometry,
        target: usize,
        triangle: [usize; 3],
    ) -> ([f32; 3], [f32; 3], f32) {
        let positions = triangle.map(|vertex| {
            std::array::from_fn(|axis| {
                source.positions[vertex][axis] + source.deltas[target][vertex][axis]
            })
        });
        triangle_frame(positions, triangle.map(|vertex| source.uvs[vertex]))
    }

    fn triangle_frame(positions: [[f32; 3]; 3], uvs: [[f32; 2]; 3]) -> ([f32; 3], [f32; 3], f32) {
        let [e1, e2] = [1, 2].map(|i| {
            std::array::from_fn::<_, 3, _>(|axis| positions[i][axis] - positions[0][axis])
        });
        let [d1, d2] = [1, 2].map(|i| [uvs[i][0] - uvs[0][0], uvs[i][1] - uvs[0][1]]);
        let determinant = d1[0] * d2[1] - d1[1] * d2[0];
        assert!(determinant.abs() > 0.5);
        assert!(
            dot(cross(e1, e2), cross(e1, e2)) > 0.5,
            "only healthy triangles constrain w"
        );
        let tangent = normalize(std::array::from_fn(|axis| {
            (e1[axis] * d2[1] - e2[axis] * d1[1]) / determinant
        }));
        let bitangent = normalize(std::array::from_fn(|axis| {
            (e2[axis] * d1[0] - e1[axis] * d2[0]) / determinant
        }));
        (
            tangent,
            bitangent,
            dot(cross(Z, tangent), bitangent).signum(),
        )
    }

    fn assert_basis(normal: [f32; 3], tangent: [f32; 4], expected: ([f32; 3], [f32; 3], f32)) {
        assert_eq!(normal, Z);
        assert!(tangent.iter().all(|component| component.is_finite()));
        let actual_tangent = [tangent[0], tangent[1], tangent[2]];
        let actual_bitangent =
            cross(normal, actual_tangent).map(|component| component * tangent[3]);
        assert_eq!(
            tangent[3], expected.2,
            "healthy endpoint N={normal:?} T={actual_tangent:?} B={actual_bitangent:?}; expected B={:?}",
            expected.1,
        );
        for axis in 0..3 {
            assert!((actual_tangent[axis] - expected.0[axis]).abs() < 1.0e-6);
            assert!(
                (actual_bitangent[axis] - expected.1[axis]).abs() < 1.0e-6,
                "wrong reconstructed +V direction"
            );
        }
    }

    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    fn normalize(vector: [f32; 3]) -> [f32; 3] {
        let length = dot(vector, vector).sqrt();
        assert!(length > 0.0);
        vector.map(|component| component / length)
    }
}
