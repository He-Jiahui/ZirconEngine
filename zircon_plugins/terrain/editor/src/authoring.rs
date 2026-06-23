#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrainHeightfieldImportRequest {
    pub width: u32,
    pub height: u32,
    pub sample_count: Option<usize>,
    pub source_extension: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainImportKind {
    Heightfield,
    LayerStack,
}

impl TerrainImportKind {
    pub fn output_kind(self) -> &'static str {
        match self {
            Self::Heightfield => "terrain.heightfield",
            Self::LayerStack => "terrain.layer_stack",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrainImportPlan {
    pub import_kind: TerrainImportKind,
    pub normalized_extension: String,
    pub output_kind: &'static str,
    pub expected_sample_count: usize,
}

pub fn validate_heightfield_import(request: &TerrainHeightfieldImportRequest) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if request.width == 0 || request.height == 0 {
        diagnostics.push("terrain heightfield dimensions must be greater than zero".to_string());
    }
    let extension = normalized_terrain_import_extension(&request.source_extension);
    if !terrain_import_extension_supported(&extension) {
        diagnostics.push(format!(
            "terrain heightfield extension `{}` is not supported",
            request.source_extension
        ));
    }
    if let Some(sample_count) = request.sample_count {
        let expected = request.width as usize * request.height as usize;
        if expected != sample_count {
            diagnostics.push(format!(
                "terrain heightfield import expected {} samples for {}x{} source but found {}",
                expected, request.width, request.height, sample_count
            ));
        }
    }
    diagnostics
}

pub fn plan_terrain_import(
    import_kind: TerrainImportKind,
    request: &TerrainHeightfieldImportRequest,
) -> Result<TerrainImportPlan, Vec<String>> {
    let diagnostics = validate_heightfield_import(request);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let normalized_extension = normalized_terrain_import_extension(&request.source_extension);
    Ok(TerrainImportPlan {
        import_kind,
        normalized_extension,
        output_kind: import_kind.output_kind(),
        expected_sample_count: request.width as usize * request.height as usize,
    })
}

pub fn terrain_import_output_kind(source_extension: &str) -> Option<&'static str> {
    match normalized_terrain_import_extension(source_extension).as_str() {
        "raw" | "r16" | "png" => Some("terrain.heightfield"),
        _ => None,
    }
}

fn normalized_terrain_import_extension(source_extension: &str) -> String {
    source_extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

fn terrain_import_extension_supported(extension: &str) -> bool {
    matches!(extension, "raw" | "r16" | "png")
}
