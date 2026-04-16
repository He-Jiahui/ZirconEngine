use std::fs;
use std::path::Path;

use image::GenericImageView;
use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use zircon_math::{Vec2, Vec3};

use crate::assets::{
    ImportedAsset, MaterialAsset, ModelAsset, ModelPrimitiveAsset, SceneAsset, ShaderAsset,
    TextureAsset, UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
};
use crate::{AssetImportError, AssetUri, MeshVertex};

#[derive(Clone, Debug, Default)]
pub struct AssetImporter;

impl AssetImporter {
    pub fn import_from_source(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let lower_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        if lower_name.ends_with(".material.toml") {
            return self.import_material(source_path);
        }
        if lower_name.ends_with(".scene.toml") {
            return self.import_scene(source_path);
        }
        if lower_name.ends_with(".ui.toml") {
            return self.import_ui_asset(source_path);
        }

        let extension = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        match extension.as_str() {
            "png" | "jpg" | "jpeg" => self.import_texture(source_path, uri),
            "wgsl" => self.import_shader(source_path, uri),
            "obj" => self.import_obj(source_path, uri),
            "gltf" | "glb" => self.import_gltf(source_path, uri),
            "fbx" => Err(AssetImportError::UnsupportedFormat(format!(
                "fbx importer is not implemented for this milestone: {}",
                source_path.display()
            ))),
            other => Err(AssetImportError::UnsupportedFormat(format!(
                "{other} from {}",
                source_path.display()
            ))),
        }
    }

    fn import_texture(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let image = image::open(source_path).map_err(|error| {
            AssetImportError::Parse(format!("decode image {}: {error}", source_path.display()))
        })?;
        let rgba = image.to_rgba8();
        let (width, height) = image.dimensions();
        Ok(ImportedAsset::Texture(TextureAsset {
            uri: uri.clone(),
            width,
            height,
            rgba: rgba.into_raw(),
        }))
    }

    fn import_shader(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let source = fs::read_to_string(source_path)?;
        validate_wgsl(uri, &source)?;
        Ok(ImportedAsset::Shader(ShaderAsset {
            uri: uri.clone(),
            source,
        }))
    }

    fn import_material(&self, source_path: &Path) -> Result<ImportedAsset, AssetImportError> {
        let document = fs::read_to_string(source_path)?;
        let material = MaterialAsset::from_toml_str(&document)
            .map_err(|error| AssetImportError::Parse(format!("parse material toml: {error}")))?;
        Ok(ImportedAsset::Material(material))
    }

    fn import_scene(&self, source_path: &Path) -> Result<ImportedAsset, AssetImportError> {
        let document = fs::read_to_string(source_path)?;
        let scene = SceneAsset::from_toml_str(&document)
            .map_err(|error| AssetImportError::Parse(format!("parse scene toml: {error}")))?;
        Ok(ImportedAsset::Scene(scene))
    }

    fn import_ui_asset(&self, source_path: &Path) -> Result<ImportedAsset, AssetImportError> {
        let document = fs::read_to_string(source_path)?;
        if let Ok(asset) = UiLayoutAsset::from_toml_str(&document) {
            return Ok(ImportedAsset::UiLayout(asset));
        }
        if let Ok(asset) = UiWidgetAsset::from_toml_str(&document) {
            return Ok(ImportedAsset::UiWidget(asset));
        }
        if let Ok(asset) = UiStyleAsset::from_toml_str(&document) {
            return Ok(ImportedAsset::UiStyle(asset));
        }
        Err(AssetImportError::Parse(format!(
            "parse ui asset toml {}: unsupported or mismatched [asset.kind]",
            source_path.display()
        )))
    }

    fn import_obj(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let (models, _) = tobj::load_obj(
            source_path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .map_err(|error| AssetImportError::Parse(format!("parse obj: {error}")))?;

        let primitives = models
            .into_iter()
            .map(|model| {
                primitive_from_indexed_mesh(
                    &model.mesh.positions,
                    &model.mesh.normals,
                    &model.mesh.texcoords,
                    &model.mesh.indices,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ImportedAsset::Model(ModelAsset {
            uri: uri.clone(),
            primitives,
        }))
    }

    fn import_gltf(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let (document, buffers, _) = gltf::import(source_path)
            .map_err(|error| AssetImportError::Parse(format!("parse gltf: {error}")))?;
        let mut primitives = Vec::new();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));
                let positions = reader
                    .read_positions()
                    .ok_or_else(|| {
                        AssetImportError::Parse("gltf primitive missing positions".to_string())
                    })?
                    .flat_map(|position| position.into_iter())
                    .collect::<Vec<_>>();
                let normals = reader
                    .read_normals()
                    .map(|iter| {
                        iter.flat_map(|normal| normal.into_iter())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let texcoords = reader
                    .read_tex_coords(0)
                    .map(|set| {
                        set.into_f32()
                            .flat_map(|uv| uv.into_iter())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let indices = reader
                    .read_indices()
                    .map(|indices| indices.into_u32().collect::<Vec<_>>())
                    .unwrap_or_else(|| {
                        let vertex_count = positions.len() / 3;
                        (0..vertex_count as u32).collect()
                    });

                primitives.push(primitive_from_indexed_mesh(
                    &positions, &normals, &texcoords, &indices,
                )?);
            }
        }

        Ok(ImportedAsset::Model(ModelAsset {
            uri: uri.clone(),
            primitives,
        }))
    }
}

fn validate_wgsl(uri: &AssetUri, source: &str) -> Result<(), AssetImportError> {
    let module = wgsl::parse_str(source).map_err(|error| {
        AssetImportError::ShaderValidation(format!("{uri}: {}", error.emit_to_string(source)))
    })?;
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    validator
        .validate(&module)
        .map_err(|error| AssetImportError::ShaderValidation(format!("{uri}: {error}")))?;
    Ok(())
}

fn primitive_from_indexed_mesh(
    positions: &[f32],
    normals: &[f32],
    texcoords: &[f32],
    indices: &[u32],
) -> Result<ModelPrimitiveAsset, AssetImportError> {
    if positions.len() % 3 != 0 {
        return Err(AssetImportError::Parse(
            "vertex positions were not a multiple of 3".to_string(),
        ));
    }
    let vertex_count = positions.len() / 3;
    let mut computed_normals = if normals.is_empty() {
        generate_normals(positions, indices)
    } else {
        normals.to_vec()
    };
    if computed_normals.len() < vertex_count * 3 {
        computed_normals.resize(vertex_count * 3, 0.0);
    }

    let vertices = (0..vertex_count)
        .map(|index| {
            let position = Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            );
            let normal = Vec3::new(
                computed_normals[index * 3],
                computed_normals[index * 3 + 1],
                computed_normals[index * 3 + 2],
            );
            let uv = if texcoords.len() >= (index + 1) * 2 {
                Vec2::new(texcoords[index * 2], texcoords[index * 2 + 1])
            } else {
                Vec2::ZERO
            };
            MeshVertex::new(
                position,
                if normal.length_squared() <= f32::EPSILON {
                    Vec3::Y
                } else {
                    normal.normalize_or_zero()
                },
                uv,
            )
        })
        .collect();

    Ok(ModelPrimitiveAsset {
        vertices,
        indices: indices.to_vec(),
    })
}

fn generate_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
    let vertex_count = positions.len() / 3;
    let mut normals = vec![0.0_f32; vertex_count * 3];

    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;
        let position = |index: usize| -> Vec3 {
            Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            )
        };
        let face_normal = (position(b) - position(a))
            .cross(position(c) - position(a))
            .normalize_or_zero();
        for index in [a, b, c] {
            normals[index * 3] += face_normal.x;
            normals[index * 3 + 1] += face_normal.y;
            normals[index * 3 + 2] += face_normal.z;
        }
    }

    normals
}
