#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrainHeightfieldImportRequest {
    pub width: u32,
    pub height: u32,
    pub sample_count: Option<usize>,
    pub source_extension: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainHeightfieldSourceFormat {
    Raw,
    R16,
    Png,
}

impl TerrainHeightfieldSourceFormat {
    pub const fn canonical_extension(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::R16 => "r16",
            Self::Png => "png",
        }
    }

    fn parse(source_extension: &str) -> Option<Self> {
        let extension = source_extension.trim().trim_start_matches('.');
        if extension.eq_ignore_ascii_case("raw") {
            Some(Self::Raw)
        } else if extension.eq_ignore_ascii_case("r16") {
            Some(Self::R16)
        } else if extension.eq_ignore_ascii_case("png") {
            Some(Self::Png)
        } else {
            None
        }
    }
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
    pub source_format: TerrainHeightfieldSourceFormat,
    pub normalized_extension: String,
    pub output_kind: &'static str,
    pub expected_sample_count: usize,
}

pub fn validate_heightfield_import(request: &TerrainHeightfieldImportRequest) -> Vec<String> {
    inspect_heightfield_import(request).diagnostics
}

struct TerrainHeightfieldImportInspection {
    diagnostics: Vec<String>,
    source_format: Option<TerrainHeightfieldSourceFormat>,
    expected_sample_count: Option<usize>,
}

fn inspect_heightfield_import(
    request: &TerrainHeightfieldImportRequest,
) -> TerrainHeightfieldImportInspection {
    let mut diagnostics = Vec::new();
    if request.width == 0 || request.height == 0 {
        diagnostics.push("terrain heightfield dimensions must be greater than zero".to_string());
    }
    let source_format = TerrainHeightfieldSourceFormat::parse(&request.source_extension);
    if source_format.is_none() {
        diagnostics.push(format!(
            "terrain heightfield extension `{}` is not supported",
            request.source_extension
        ));
    }
    let expected_sample_count = u64::from(request.width)
        .checked_mul(u64::from(request.height))
        .and_then(|count| usize::try_from(count).ok());
    if expected_sample_count.is_none() {
        diagnostics.push(
            "terrain heightfield sample count exceeds the platform import budget".to_string(),
        );
    }
    if let Some(sample_count) = request.sample_count {
        if let Some(expected) = expected_sample_count.filter(|expected| *expected != sample_count) {
            diagnostics.push(format!(
                "terrain heightfield import expected {} samples for {}x{} source but found {}",
                expected, request.width, request.height, sample_count
            ));
        }
    }
    TerrainHeightfieldImportInspection {
        diagnostics,
        source_format,
        expected_sample_count,
    }
}

pub fn plan_terrain_import(
    import_kind: TerrainImportKind,
    request: &TerrainHeightfieldImportRequest,
) -> Result<TerrainImportPlan, Vec<String>> {
    if import_kind == TerrainImportKind::LayerStack {
        return Err(vec![
            "terrain layer stack import requires a layer-aware request with channel and format semantics"
                .to_string(),
        ]);
    }
    let inspection = inspect_heightfield_import(request);
    if !inspection.diagnostics.is_empty() {
        return Err(inspection.diagnostics);
    }
    let source_format = inspection
        .source_format
        .expect("validated terrain heightfield source format");
    Ok(TerrainImportPlan {
        import_kind,
        source_format,
        normalized_extension: source_format.canonical_extension().to_string(),
        output_kind: import_kind.output_kind(),
        expected_sample_count: inspection
            .expected_sample_count
            .expect("validated terrain heightfield sample count"),
    })
}

pub fn terrain_import_output_kind(source_extension: &str) -> Option<&'static str> {
    TerrainHeightfieldSourceFormat::parse(source_extension).map(|_| "terrain.heightfield")
}
