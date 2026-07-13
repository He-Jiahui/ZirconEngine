use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime_interface::export::ExportStage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportStageProgressKind {
    Pending,
    Running,
    Passed,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardStageProgressSnapshot {
    pub stage: ExportStage,
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
    pub stage: ExportStage,
    pub kind: ExportStageProgressKind,
    pub line: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardProgressState {
    stages: Vec<ExportWizardStageProgressSnapshot>,
    current_stage: Option<ExportStage>,
    json_diagnostics_depth: usize,
}

impl ExportWizardStageProgressSnapshot {
    fn pending(stage: ExportStage) -> Self {
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
        Self::for_stages(ExportStage::ALL)
    }

    pub fn for_stages(stages: impl IntoIterator<Item = ExportStage>) -> Self {
        Self {
            stages: stages
                .into_iter()
                .map(ExportWizardStageProgressSnapshot::pending)
                .collect(),
            current_stage: None,
            json_diagnostics_depth: 0,
        }
    }

    pub fn snapshots(&self) -> &[ExportWizardStageProgressSnapshot] {
        &self.stages
    }

    pub fn current_stage(&self) -> Option<ExportStage> {
        self.current_stage
    }

    pub fn snapshot(&self, stage: ExportStage) -> Option<&ExportWizardStageProgressSnapshot> {
        self.stages.iter().find(|snapshot| snapshot.stage == stage)
    }

    pub fn push_stdout_line(&mut self, line: &str) -> Option<ExportWizardStreamEvent> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some((stage, profile)) = parse_stage_banner(trimmed) {
            self.current_stage = Some(stage);
            self.json_diagnostics_depth = 0;
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

        if self.json_diagnostics_depth > 0 {
            let diagnostic = json_string_line_value(trimmed);
            self.json_diagnostics_depth =
                json_array_depth_after_line(trimmed, self.json_diagnostics_depth);
            if let Some(diagnostic) = diagnostic {
                let snapshot = self.stage_mut(stage);
                snapshot.diagnostics.push(diagnostic);
                return Some(ExportWizardStreamEvent {
                    stage,
                    kind: snapshot.kind,
                    line: trimmed.to_string(),
                });
            }
            return None;
        }

        if starts_json_diagnostics_array(trimmed) {
            self.json_diagnostics_depth = json_array_depth_after_line(trimmed, 0);
            return None;
        }

        if looks_like_report_json_line(trimmed) {
            return None;
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

    fn stage_mut(&mut self, stage: ExportStage) -> &mut ExportWizardStageProgressSnapshot {
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

pub fn export_pipeline_stages_for_strategies(
    strategies: &[ExportPackagingStrategy],
) -> Vec<ExportStage> {
    let mut stages = Vec::new();
    stages.push(ExportStage::Validate);
    if strategies.contains(&ExportPackagingStrategy::SourceTemplate) {
        push_stage_once(&mut stages, ExportStage::SourceTemplate);
    }
    if strategies.contains(&ExportPackagingStrategy::NativeDynamic) {
        push_stage_once(&mut stages, ExportStage::NativeDynamic);
        push_stage_once(&mut stages, ExportStage::CompileHost);
        push_stage_once(&mut stages, ExportStage::CookAssets);
        push_stage_once(&mut stages, ExportStage::Pack);
        push_stage_once(&mut stages, ExportStage::PlatformBundle);
    }
    if strategies.contains(&ExportPackagingStrategy::LibraryEmbed) {
        push_stage_once(&mut stages, ExportStage::CompileHost);
        push_stage_once(&mut stages, ExportStage::CookAssets);
        push_stage_once(&mut stages, ExportStage::Pack);
        push_stage_once(&mut stages, ExportStage::PlatformBundle);
    }
    stages.push(ExportStage::Report);
    stages
}

fn push_stage_once(stages: &mut Vec<ExportStage>, stage: ExportStage) {
    if !stages.contains(&stage) {
        stages.push(stage);
    }
}

fn parse_stage_banner(line: &str) -> Option<(ExportStage, Option<String>)> {
    let rest = line.strip_prefix("zircon_export ")?;
    let mut stage = None;
    let mut profile = None;
    for token in rest.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };
        match key {
            "stage" => stage = value.parse().ok(),
            "profile" => profile = Some(value.to_string()),
            _ => {}
        }
    }
    stage.map(|stage| (stage, profile))
}

fn is_artifact_key(key: &str) -> bool {
    matches!(
        key.trim(),
        "asset_manifest"
            | "bundle"
            | "cooked_asset_manifest"
            | "delta_pack"
            | "host"
            | "loader_manifest"
            | "native_plugin_root"
            | "native_plugins"
            | "pack"
            | "pipeline_report"
            | "previous_pack"
            | "project"
            | "plugins_dir"
            | "report"
            | "source_asset_manifest"
            | "stage_output"
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
    line.contains("diagnostic")
        || line.contains("Diagnostic")
        || line.contains("error")
        || line.contains("failed")
        || line.contains("fatal")
}

fn starts_json_diagnostics_array(line: &str) -> bool {
    line.starts_with("\"diagnostics\"") && line.contains('[')
}

fn json_array_depth_after_line(line: &str, current_depth: usize) -> usize {
    let mut depth = current_depth;
    for character in line.chars() {
        match character {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth
}

fn looks_like_report_json_line(line: &str) -> bool {
    matches!(
        line.chars().next(),
        Some('{') | Some('}') | Some('[') | Some(']')
    ) || (line.starts_with('"') && (line.contains("\":") || json_string_line_value(line).is_some()))
}

fn json_string_line_value(line: &str) -> Option<String> {
    let value = line.trim_end_matches(',');
    if !value.starts_with('"') || !value.ends_with('"') || value.contains("\":") {
        return None;
    }
    Some(
        value
            .trim_matches('"')
            .replace("\\\"", "\"")
            .replace("\\\\", "\\"),
    )
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
        assert_eq!(event.stage, ExportStage::CookAssets);
        assert_eq!(event.kind, ExportStageProgressKind::Running);

        progress
            .push_stdout_line("cooked_asset_manifest=D:\\export\\stages\\cook_assets\\assets.json");
        progress.push_stdout_line("report=D:\\export\\stages\\cook_assets\\report.json");
        progress.push_stdout_line(r#""fatal": false,"#);

        let cook_assets = progress
            .snapshot(ExportStage::CookAssets)
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
            .snapshot(ExportStage::PlatformBundle)
            .expect("PlatformBundle snapshot should exist");
        assert_eq!(platform_bundle.kind, ExportStageProgressKind::Fatal);
        assert!(platform_bundle
            .artifact_paths
            .iter()
            .any(|artifact| artifact.key == "bundle"));
    }

    #[test]
    fn export_wizard_progress_ignores_pipeline_summary_json_lines() {
        let mut progress = ExportWizardProgressState::new();

        progress.push_stdout_line("zircon_export stage=Report profile=windows-release");
        progress.push_stdout_line(r#"  "export_plan": {"#);
        progress.push_stdout_line(r#"    "unsupported_strategies": ["#);
        progress.push_stdout_line(r#"      "future_error_path""#);
        progress.push_stdout_line(r#"    ]"#);
        progress.push_stdout_line(r#"  },"#);
        progress.push_stdout_line(r#""fatal": false,"#);

        let report = progress
            .snapshot(ExportStage::Report)
            .expect("Report snapshot should exist");
        assert_eq!(report.kind, ExportStageProgressKind::Passed);
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    }

    #[test]
    fn export_wizard_progress_records_report_diagnostics_array_lines() {
        let mut progress = ExportWizardProgressState::new();

        progress.push_stdout_line("zircon_export stage=Report profile=windows-release");
        progress.push_stdout_line(r#""diagnostics": ["#);
        progress.push_stdout_line(r#"  "validate failed","#);
        progress.push_stdout_line(r#"],"#);
        progress.push_stdout_line(r#""fatal": true,"#);

        let report = progress
            .snapshot(ExportStage::Report)
            .expect("Report snapshot should exist");
        assert_eq!(report.kind, ExportStageProgressKind::Fatal);
        assert_eq!(report.diagnostics, vec!["validate failed"]);
    }

    #[test]
    fn export_pipeline_stage_parser_accepts_cli_and_report_stage_names() {
        assert_eq!(
            "source_template".parse::<ExportStage>().ok(),
            Some(ExportStage::SourceTemplate)
        );
        assert_eq!(
            "SourceTemplate".parse::<ExportStage>().ok(),
            Some(ExportStage::SourceTemplate)
        );
        assert_eq!(
            "platform_bundle".parse::<ExportStage>().ok(),
            Some(ExportStage::PlatformBundle)
        );
        assert_eq!(
            "native_dynamic".parse::<ExportStage>().ok(),
            Some(ExportStage::NativeDynamic)
        );
        assert_eq!(
            "NativeDynamic".parse::<ExportStage>().ok(),
            Some(ExportStage::NativeDynamic)
        );
        assert_eq!(
            ExportStage::ALL,
            [
                ExportStage::Validate,
                ExportStage::SourceTemplate,
                ExportStage::NativeDynamic,
                ExportStage::CompileHost,
                ExportStage::CookAssets,
                ExportStage::Pack,
                ExportStage::PlatformBundle,
                ExportStage::Report,
            ]
        );
    }
}
