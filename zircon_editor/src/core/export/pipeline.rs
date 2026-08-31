use std::error::Error;
use std::fmt;

use zircon_runtime_interface::export::{
    ExportArtifactRef, ExportDigest, ExportPipelineReport, ExportStage, ExportStageIo,
    ExportStageRecord, ExportStageStatus,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportStageNode {
    pub stage: ExportStage,
    pub dependencies: Vec<ExportStage>,
}

impl ExportStageNode {
    pub fn new(stage: ExportStage, dependencies: impl IntoIterator<Item = ExportStage>) -> Self {
        Self {
            stage,
            dependencies: dependencies.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportPipelinePlan {
    ordered_nodes: Vec<ExportStageNode>,
}

impl ExportPipelinePlan {
    pub fn new(
        nodes: impl IntoIterator<Item = ExportStageNode>,
    ) -> Result<Self, ExportPipelinePlanError> {
        let nodes = nodes.into_iter().collect::<Vec<_>>();
        reject_duplicate_stages(&nodes)?;
        reject_missing_dependencies(&nodes)?;
        let ordered_nodes = topological_order(nodes)?;
        Ok(Self { ordered_nodes })
    }

    pub fn ordered_nodes(&self) -> &[ExportStageNode] {
        &self.ordered_nodes
    }

    pub fn run<E>(
        &self,
        executor: &mut E,
        resume: Option<&ExportPipelineReport>,
    ) -> Result<ExportPipelineReport, ExportPipelineRunError<E::Error>>
    where
        E: ExportStageExecutor,
    {
        let mut report = ExportPipelineReport::default();
        for node in &self.ordered_nodes {
            let preparation = match executor.prepare(node.stage, &report.stages) {
                Ok(preparation) => preparation,
                Err(source) => {
                    report.stages.push(ExportStageRecord {
                        stage: node.stage,
                        io: ExportStageIo {
                            inputs: Vec::new(),
                            outputs: Vec::new(),
                            fingerprint: ExportDigest::ZERO,
                        },
                        status: ExportStageStatus::Failed,
                        diagnostics: vec![source.to_string()],
                    });
                    return Err(ExportPipelineRunError::prepared(node.stage, report, source));
                }
            };
            let fingerprint = stage_fingerprint(node.stage, &preparation);

            if let Some(previous) = reusable_record(resume, node.stage, fingerprint)
                .filter(|previous| executor.can_reuse(node.stage, previous, &preparation))
            {
                report.stages.push(ExportStageRecord {
                    stage: node.stage,
                    io: ExportStageIo {
                        inputs: preparation.inputs,
                        outputs: previous.io.outputs.clone(),
                        fingerprint,
                    },
                    status: ExportStageStatus::Skipped,
                    diagnostics: previous.diagnostics.clone(),
                });
                continue;
            }

            match executor.execute(node.stage, &preparation) {
                Ok(output) => report.stages.push(ExportStageRecord {
                    stage: node.stage,
                    io: ExportStageIo {
                        inputs: preparation.inputs,
                        outputs: output.outputs,
                        fingerprint,
                    },
                    status: ExportStageStatus::Passed,
                    diagnostics: output.diagnostics,
                }),
                Err(source) => {
                    let diagnostics = vec![source.to_string()];
                    report.stages.push(ExportStageRecord {
                        stage: node.stage,
                        io: ExportStageIo {
                            inputs: preparation.inputs,
                            outputs: preparation.expected_outputs,
                            fingerprint,
                        },
                        status: ExportStageStatus::Failed,
                        diagnostics,
                    });
                    return Err(ExportPipelineRunError::executed(node.stage, report, source));
                }
            }
        }
        Ok(report)
    }
}

pub trait ExportStageExecutor {
    type Error: Error + Send + Sync + 'static;

    fn prepare(
        &mut self,
        stage: ExportStage,
        completed: &[ExportStageRecord],
    ) -> Result<ExportStagePreparation, Self::Error>;

    fn execute(
        &mut self,
        stage: ExportStage,
        preparation: &ExportStagePreparation,
    ) -> Result<ExportStageOutput, Self::Error>;

    /// Revalidates persisted outputs before a matching fingerprint is skipped.
    /// Concrete filesystem executors must reject missing or changed artifacts.
    fn can_reuse(
        &mut self,
        _stage: ExportStage,
        _previous: &ExportStageRecord,
        _preparation: &ExportStagePreparation,
    ) -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportStagePreparation {
    pub inputs: Vec<ExportArtifactRef>,
    pub expected_outputs: Vec<ExportArtifactRef>,
    pub parameter_digest: ExportDigest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportStageOutput {
    pub outputs: Vec<ExportArtifactRef>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportPipelinePlanError {
    DuplicateStage {
        stage: ExportStage,
    },
    MissingDependency {
        stage: ExportStage,
        dependency: ExportStage,
    },
    DependencyCycle {
        stages: Vec<ExportStage>,
    },
}

impl fmt::Display for ExportPipelinePlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateStage { stage } => {
                write!(
                    formatter,
                    "export stage `{}` is declared more than once",
                    stage.cli_id()
                )
            }
            Self::MissingDependency { stage, dependency } => write!(
                formatter,
                "export stage `{}` depends on missing stage `{}`",
                stage.cli_id(),
                dependency.cli_id()
            ),
            Self::DependencyCycle { stages } => write_dependency_cycle(formatter, stages),
        }
    }
}

fn write_dependency_cycle(
    formatter: &mut fmt::Formatter<'_>,
    stages: &[ExportStage],
) -> fmt::Result {
    formatter.write_str("export stage dependency cycle contains: ")?;
    for (index, stage) in stages.iter().enumerate() {
        if index != 0 {
            formatter.write_str(", ")?;
        }
        formatter.write_str(stage.cli_id())?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "pipeline/dependency_cycle_format_tests.rs"]
mod dependency_cycle_format_tests;

impl Error for ExportPipelinePlanError {}

#[derive(Debug)]
pub struct ExportPipelineRunError<E> {
    stage: ExportStage,
    phase: ExportPipelineFailurePhase,
    report: ExportPipelineReport,
    source: E,
}

impl<E> ExportPipelineRunError<E> {
    fn prepared(stage: ExportStage, report: ExportPipelineReport, source: E) -> Self {
        Self {
            stage,
            phase: ExportPipelineFailurePhase::Prepare,
            report,
            source,
        }
    }

    fn executed(stage: ExportStage, report: ExportPipelineReport, source: E) -> Self {
        Self {
            stage,
            phase: ExportPipelineFailurePhase::Execute,
            report,
            source,
        }
    }

    pub const fn stage(&self) -> ExportStage {
        self.stage
    }

    pub fn report(&self) -> &ExportPipelineReport {
        &self.report
    }

    pub fn into_parts(self) -> (ExportPipelineReport, E) {
        (self.report, self.source)
    }
}

impl<E: fmt::Display> fmt::Display for ExportPipelineRunError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "export stage `{}` {} failed: {}",
            self.stage.cli_id(),
            self.phase.as_str(),
            self.source
        )
    }
}

impl<E: Error + 'static> Error for ExportPipelineRunError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Clone, Copy, Debug)]
enum ExportPipelineFailurePhase {
    Prepare,
    Execute,
}

impl ExportPipelineFailurePhase {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Prepare => "preparation",
            Self::Execute => "execution",
        }
    }
}

fn reject_duplicate_stages(nodes: &[ExportStageNode]) -> Result<(), ExportPipelinePlanError> {
    let mut seen = Vec::new();
    for node in nodes {
        if seen.contains(&node.stage) {
            return Err(ExportPipelinePlanError::DuplicateStage { stage: node.stage });
        }
        seen.push(node.stage);
    }
    Ok(())
}

fn reject_missing_dependencies(nodes: &[ExportStageNode]) -> Result<(), ExportPipelinePlanError> {
    for node in nodes {
        for dependency in &node.dependencies {
            if !nodes.iter().any(|candidate| candidate.stage == *dependency) {
                return Err(ExportPipelinePlanError::MissingDependency {
                    stage: node.stage,
                    dependency: *dependency,
                });
            }
        }
    }
    Ok(())
}

fn topological_order(
    mut remaining: Vec<ExportStageNode>,
) -> Result<Vec<ExportStageNode>, ExportPipelinePlanError> {
    let mut ordered = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let Some(index) = remaining.iter().position(|node| {
            node.dependencies.iter().all(|dependency| {
                ordered
                    .iter()
                    .any(|done: &ExportStageNode| done.stage == *dependency)
            })
        }) else {
            return Err(ExportPipelinePlanError::DependencyCycle {
                stages: remaining.iter().map(|node| node.stage).collect(),
            });
        };
        ordered.push(remaining.remove(index));
    }
    Ok(ordered)
}

fn reusable_record(
    resume: Option<&ExportPipelineReport>,
    stage: ExportStage,
    fingerprint: ExportDigest,
) -> Option<&ExportStageRecord> {
    resume?.record(stage).filter(|record| {
        matches!(
            record.status,
            ExportStageStatus::Passed | ExportStageStatus::Skipped
        ) && record.io.fingerprint == fingerprint
    })
}

fn stage_fingerprint(stage: ExportStage, preparation: &ExportStagePreparation) -> ExportDigest {
    let mut hasher = blake3::Hasher::new();
    hash_bytes(&mut hasher, stage.cli_id().as_bytes());
    hash_bytes(&mut hasher, preparation.parameter_digest.as_bytes());
    hash_usize(&mut hasher, preparation.inputs.len());
    for input in &preparation.inputs {
        hash_bytes(&mut hasher, input.key.as_bytes());
        hash_bytes(&mut hasher, input.locator.as_bytes());
        match input.digest {
            Some(digest) => {
                hasher.update(&[1]);
                hash_bytes(&mut hasher, digest.as_bytes());
            }
            None => {
                hasher.update(&[0]);
            }
        }
    }
    hash_usize(&mut hasher, preparation.expected_outputs.len());
    for output in &preparation.expected_outputs {
        hash_bytes(&mut hasher, output.key.as_bytes());
        hash_bytes(&mut hasher, output.locator.as_bytes());
        match output.digest {
            Some(digest) => {
                hasher.update(&[1]);
                hash_bytes(&mut hasher, digest.as_bytes());
            }
            None => {
                hasher.update(&[0]);
            }
        }
    }
    ExportDigest::from_bytes(*hasher.finalize().as_bytes())
}

fn hash_usize(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

fn hash_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hash_usize(hasher, bytes.len());
    hasher.update(bytes);
}
