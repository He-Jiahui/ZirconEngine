use zircon_runtime::plugin::ExportPipelineStage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportStageProgressKind {
    Pending,
    Running,
    Passed,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardStageProgressSnapshot {
    pub stage: ExportPipelineStage,
    pub kind: ExportStageProgressKind,
    pub profile: Option<String>,
    pub report_path: Option<String>,
    pub artifact_paths: Vec<ExportWizardStageArtifactPath>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardStageArtifactPath {
    pub key: String,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardStreamEvent {
    pub stage: ExportPipelineStage,
    pub kind: ExportStageProgressKind,
    pub line: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardProgressState {
    stages: Vec<ExportWizardStageProgressSnapshot>,
    current_stage: Option<ExportPipelineStage>,
}

impl ExportWizardStageProgressSnapshot {
    fn pending(stage: ExportPipelineStage) -> Self {
        Self {
            stage,
            kind: ExportStageProgressKind::Pending,
            profile: None,
            report_path: None,
            artifact_paths: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn record_artifact_path(&mut self, key: &str, path: &str) {
        if key == "report" || key == "pipeline_report" {
            self.report_path = Some(path.to_string());
        }
        if let Some(existing) = self
            .artifact_paths
            .iter_mut()
            .find(|artifact| artifact.key == key)
        {
            existing.path = path.to_string();
            return;
        }
        self.artifact_paths.push(ExportWizardStageArtifactPath {
            key: key.to_string(),
            path: path.to_string(),
        });
    }
}

impl ExportWizardProgressState {
    pub fn new() -> Self {
        Self {
            stages: export_pipeline_stages()
                .iter()
                .copied()
                .map(ExportWizardStageProgressSnapshot::pending)
                .collect(),
            current_stage: None,
        }
    }

    pub fn snapshots(&self) -> &[ExportWizardStageProgressSnapshot] {
        &self.stages
    }

    pub fn current_stage(&self) -> Option<ExportPipelineStage> {
        self.current_stage
    }

    pub fn snapshot(
        &self,
        stage: ExportPipelineStage,
    ) -> Option<&ExportWizardStageProgressSnapshot> {
        self.stages.iter().find(|snapshot| snapshot.stage == stage)
    }

    pub fn push_stdout_line(&mut self, line: &str) -> Option<ExportWizardStreamEvent> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some((stage, profile)) = parse_stage_banner(trimmed) {
            self.current_stage = Some(stage);
            let snapshot = self.stage_mut(stage);
            snapshot.kind = ExportStageProgressKind::Running;
            snapshot.profile = profile;
            return Some(ExportWizardStreamEvent {
                stage,
                kind: snapshot.kind,
                line: trimmed.to_string(),
            });
        }

        let Some(stage) = self.current_stage else {
            return None;
        };

        if let Some((key, value)) = trimmed.split_once('=') {
            if is_artifact_key(key) {
                let snapshot = self.stage_mut(stage);
                snapshot.record_artifact_path(key.trim(), value.trim());
                return Some(ExportWizardStreamEvent {
                    stage,
                    kind: snapshot.kind,
                    line: trimmed.to_string(),
                });
            }
        }

        if let Some(fatal) = parse_json_fatal_field(line) {
            let snapshot = self.stage_mut(stage);
            snapshot.kind = if fatal {
                ExportStageProgressKind::Fatal
            } else {
                ExportStageProgressKind::Passed
            };
            return Some(ExportWizardStreamEvent {
                stage,
                kind: snapshot.kind,
                line: trimmed.to_string(),
            });
        }

        if looks_like_diagnostic(trimmed) {
            let snapshot = self.stage_mut(stage);
            snapshot.diagnostics.push(trimmed.to_string());
            return Some(ExportWizardStreamEvent {
                stage,
                kind: snapshot.kind,
                line: trimmed.to_string(),
            });
        }

        None
    }

    fn stage_mut(&mut self, stage: ExportPipelineStage) -> &mut ExportWizardStageProgressSnapshot {
        self.stages
            .iter_mut()
            .find(|snapshot| snapshot.stage == stage)
            .expect("export progress state is initialized with every pipeline stage")
    }
}

impl Default for ExportWizardProgressState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn export_pipeline_stages() -> [ExportPipelineStage; 7] {
    [
        ExportPipelineStage::Validate,
        ExportPipelineStage::CompileHost,
        ExportPipelineStage::SourceTemplate,
        ExportPipelineStage::CookAssets,
        ExportPipelineStage::Pack,
        ExportPipelineStage::PlatformBundle,
        ExportPipelineStage::Report,
    ]
}

pub fn parse_export_pipeline_stage(value: &str) -> Option<ExportPipelineStage> {
    match normalize_stage_name(value).as_str() {
        "validate" => Some(ExportPipelineStage::Validate),
        "compilehost" => Some(ExportPipelineStage::CompileHost),
        "sourcetemplate" => Some(ExportPipelineStage::SourceTemplate),
        "cookassets" => Some(ExportPipelineStage::CookAssets),
        "pack" => Some(ExportPipelineStage::Pack),
        "platformbundle" => Some(ExportPipelineStage::PlatformBundle),
        "report" => Some(ExportPipelineStage::Report),
        _ => None,
    }
}

fn parse_stage_banner(line: &str) -> Option<(ExportPipelineStage, Option<String>)> {
    let rest = line.strip_prefix("zircon_export ")?;
    let mut stage = None;
    let mut profile = None;
    for token in rest.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };
        match key {
            "stage" => stage = parse_export_pipeline_stage(value),
            "profile" => profile = Some(value.to_string()),
            _ => {}
        }
    }
    stage.map(|stage| (stage, profile))
}

fn normalize_stage_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| *character != '_' && *character != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_artifact_key(key: &str) -> bool {
    matches!(
        key.trim(),
        "asset_manifest"
            | "bundle"
            | "cooked_asset_manifest"
            | "delta_pack"
            | "host"
            | "pack"
            | "pipeline_report"
            | "previous_pack"
            | "project"
            | "report"
            | "source_asset_manifest"
            | "template"
            | "validate_report"
    )
}

fn parse_json_fatal_field(line: &str) -> Option<bool> {
    let leading_whitespace = line.len().saturating_sub(line.trim_start().len());
    let trimmed = line.trim_start();
    if leading_whitespace > 2 || (!trimmed.starts_with('{') && !trimmed.starts_with("\"fatal\"")) {
        return None;
    }
    let fatal_position = trimmed.find("\"fatal\"")?;
    let after_fatal = &trimmed[fatal_position + "\"fatal\"".len()..];
    let (_, value) = after_fatal.split_once(':')?;
    let value = value.trim_start();
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn looks_like_diagnostic(line: &str) -> bool {
    if line.starts_with("\"diagnostics\"")
        || line.starts_with("\"fatal\"")
        || line.starts_with("\"fatal_stages\"")
    {
        return false;
    }
    line.contains("diagnostic")
        || line.contains("Diagnostic")
        || line.contains("error")
        || line.contains("failed")
        || line.contains("fatal")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_wizard_progress_parses_cli_stream_into_stage_snapshots() {
        let mut progress = ExportWizardProgressState::new();

        let event = progress
            .push_stdout_line("zircon_export stage=CookAssets profile=windows-release")
            .expect("stage banner should produce a progress event");
        assert_eq!(event.stage, ExportPipelineStage::CookAssets);
        assert_eq!(event.kind, ExportStageProgressKind::Running);

        progress
            .push_stdout_line("cooked_asset_manifest=D:\\export\\stages\\cook_assets\\assets.json");
        progress.push_stdout_line("report=D:\\export\\stages\\cook_assets\\report.json");
        progress.push_stdout_line(r#""fatal": false,"#);

        let cook_assets = progress
            .snapshot(ExportPipelineStage::CookAssets)
            .expect("CookAssets snapshot should exist");
        assert_eq!(cook_assets.kind, ExportStageProgressKind::Passed);
        assert_eq!(cook_assets.profile.as_deref(), Some("windows-release"));
        assert_eq!(
            cook_assets.report_path.as_deref(),
            Some("D:\\export\\stages\\cook_assets\\report.json")
        );
        assert!(cook_assets.artifact_paths.iter().any(|artifact| {
            artifact.key == "cooked_asset_manifest"
                && artifact.path == "D:\\export\\stages\\cook_assets\\assets.json"
        }));
    }

    #[test]
    fn export_wizard_progress_marks_fatal_stage_reports() {
        let mut progress = ExportWizardProgressState::new();

        progress.push_stdout_line("zircon_export stage=PlatformBundle profile=windows-release");
        progress.push_stdout_line("bundle=D:\\export\\bundle\\windows-release");
        progress.push_stdout_line(r#""fatal": true,"#);

        let platform_bundle = progress
            .snapshot(ExportPipelineStage::PlatformBundle)
            .expect("PlatformBundle snapshot should exist");
        assert_eq!(platform_bundle.kind, ExportStageProgressKind::Fatal);
        assert!(platform_bundle
            .artifact_paths
            .iter()
            .any(|artifact| artifact.key == "bundle"));
    }

    #[test]
    fn export_pipeline_stage_parser_accepts_cli_and_report_stage_names() {
        assert_eq!(
            parse_export_pipeline_stage("source_template"),
            Some(ExportPipelineStage::SourceTemplate)
        );
        assert_eq!(
            parse_export_pipeline_stage("SourceTemplate"),
            Some(ExportPipelineStage::SourceTemplate)
        );
        assert_eq!(
            parse_export_pipeline_stage("platform_bundle"),
            Some(ExportPipelineStage::PlatformBundle)
        );
        assert_eq!(
            export_pipeline_stages(),
            [
                ExportPipelineStage::Validate,
                ExportPipelineStage::CompileHost,
                ExportPipelineStage::SourceTemplate,
                ExportPipelineStage::CookAssets,
                ExportPipelineStage::Pack,
                ExportPipelineStage::PlatformBundle,
                ExportPipelineStage::Report,
            ]
        );
    }
}
