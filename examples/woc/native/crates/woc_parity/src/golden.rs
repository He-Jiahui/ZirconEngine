use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::EXPECTED_GOLDEN_SCENARIOS;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct GoldenScenario {
    pub name: String,
    pub source_owner: String,
    pub golden: String,
    pub golden_sha256: String,
    pub coverage: Vec<String>,
}

#[derive(Deserialize)]
struct GoldenManifest {
    source_commit: String,
    entries: Vec<GoldenScenario>,
}

#[derive(Clone, Debug)]
pub struct GoldenSuite {
    root: PathBuf,
    scenarios: Vec<GoldenScenario>,
    indices: BTreeMap<String, usize>,
}

#[derive(Debug, Error)]
pub enum GoldenError {
    #[error("failed to read parity file {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse parity file {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("parity inventory mismatch for {field}: actual {actual}, expected {expected}")]
    Inventory {
        field: &'static str,
        actual: usize,
        expected: usize,
    },
    #[error("duplicate parity scenario {0}")]
    DuplicateScenario(String),
    #[error("unknown parity scenario {0}")]
    UnknownScenario(String),
    #[error("golden digest mismatch for {scenario}: actual {actual}, expected {expected}")]
    Digest {
        scenario: String,
        actual: String,
        expected: String,
    },
    #[error("trace difference at {path}: expected {expected}, actual {actual}")]
    Difference {
        path: String,
        expected: String,
        actual: String,
    },
    #[error("duplicate execution diverged for {scenario}: {source}")]
    DuplicateRun {
        scenario: String,
        #[source]
        source: Box<GoldenError>,
    },
    #[error("parity source commit {actual} does not match {expected}")]
    SourceCommit {
        actual: String,
        expected: &'static str,
    },
}

impl GoldenSuite {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, GoldenError> {
        let root = root.as_ref().to_path_buf();
        let manifest_path = root.join("scenarios.json");
        let manifest: GoldenManifest = read_json(&manifest_path)?;
        if manifest.source_commit != woc_protocol::REFERENCE_COMMIT {
            return Err(GoldenError::SourceCommit {
                actual: manifest.source_commit,
                expected: woc_protocol::REFERENCE_COMMIT,
            });
        }
        if manifest.entries.len() != EXPECTED_GOLDEN_SCENARIOS {
            return Err(GoldenError::Inventory {
                field: "scenario rows",
                actual: manifest.entries.len(),
                expected: EXPECTED_GOLDEN_SCENARIOS,
            });
        }
        let mut indices = BTreeMap::new();
        for (index, scenario) in manifest.entries.iter().enumerate() {
            if indices.insert(scenario.name.clone(), index).is_some() {
                return Err(GoldenError::DuplicateScenario(scenario.name.clone()));
            }
        }
        let disk_names = fs::read_dir(root.join("golden"))
            .map_err(|source| GoldenError::Read {
                path: root.join("golden"),
                source,
            })?
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                (path.extension().and_then(|value| value.to_str()) == Some("json"))
                    .then(|| path.file_stem()?.to_str().map(str::to_owned))
                    .flatten()
            })
            .collect::<BTreeSet<_>>();
        if disk_names.len() != EXPECTED_GOLDEN_SCENARIOS {
            return Err(GoldenError::Inventory {
                field: "golden files",
                actual: disk_names.len(),
                expected: EXPECTED_GOLDEN_SCENARIOS,
            });
        }
        let manifest_names = indices.keys().cloned().collect::<BTreeSet<_>>();
        if disk_names != manifest_names {
            return Err(GoldenError::Inventory {
                field: "scenario/golden name agreement",
                actual: disk_names.intersection(&manifest_names).count(),
                expected: EXPECTED_GOLDEN_SCENARIOS,
            });
        }
        Ok(Self {
            root,
            scenarios: manifest.entries,
            indices,
        })
    }

    pub fn scenarios(&self) -> &[GoldenScenario] {
        &self.scenarios
    }

    pub fn read_expected(&self, name: &str) -> Result<Value, GoldenError> {
        let scenario = self.scenario(name)?;
        let file_name = Path::new(&scenario.golden)
            .file_name()
            .expect("catalog golden path must have a file name");
        let path = self.root.join("golden").join(file_name);
        let bytes = fs::read(&path).map_err(|source| GoldenError::Read {
            path: path.clone(),
            source,
        })?;
        let actual = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if actual != scenario.golden_sha256 {
            return Err(GoldenError::Digest {
                scenario: name.to_string(),
                actual,
                expected: scenario.golden_sha256.clone(),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| GoldenError::Parse { path, source })
    }

    pub fn compare(&self, name: &str, actual: &Value) -> Result<(), GoldenError> {
        let expected = self.read_expected(name)?;
        if let Some((path, expected, actual)) = first_difference("$", &expected, actual) {
            return Err(GoldenError::Difference {
                path,
                expected,
                actual,
            });
        }
        Ok(())
    }

    pub fn compare_double_run(
        &self,
        name: &str,
        mut run: impl FnMut() -> Value,
    ) -> Result<(), GoldenError> {
        let first = run();
        let second = run();
        if let Some((path, expected, actual)) = first_difference("$", &first, &second) {
            return Err(GoldenError::DuplicateRun {
                scenario: name.to_string(),
                source: Box::new(GoldenError::Difference {
                    path,
                    expected,
                    actual,
                }),
            });
        }
        self.compare(name, &first)
    }

    fn scenario(&self, name: &str) -> Result<&GoldenScenario, GoldenError> {
        self.indices
            .get(name)
            .map(|index| &self.scenarios[*index])
            .ok_or_else(|| GoldenError::UnknownScenario(name.to_string()))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GoldenUpdateGuard {
    enabled: bool,
}

impl GoldenUpdateGuard {
    pub fn disabled() -> Self {
        Self { enabled: false }
    }

    pub fn from_env() -> Self {
        Self::from_values(
            std::env::var("WOC_UPDATE_PARITY").ok().as_deref(),
            std::env::var("WOC_UPDATE_PARITY_CONFIRM").ok().as_deref(),
        )
    }

    pub fn from_values(update: Option<&str>, confirmation: Option<&str>) -> Self {
        Self {
            enabled: update == Some("1") && confirmation == Some(woc_protocol::REFERENCE_COMMIT),
        }
    }

    pub fn is_enabled(self) -> bool {
        self.enabled
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, GoldenError> {
    let bytes = fs::read(path).map_err(|source| GoldenError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| GoldenError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

fn first_difference(
    path: &str,
    expected: &Value,
    actual: &Value,
) -> Option<(String, String, String)> {
    match (expected, actual) {
        (Value::Array(expected), Value::Array(actual)) => {
            for index in 0..expected.len().max(actual.len()) {
                match (expected.get(index), actual.get(index)) {
                    (Some(expected), Some(actual)) => {
                        if let Some(difference) =
                            first_difference(&format!("{path}[{index}]"), expected, actual)
                        {
                            return Some(difference);
                        }
                    }
                    (expected, actual) => return difference(path, expected, actual),
                }
            }
            None
        }
        (Value::Object(expected), Value::Object(actual)) => {
            let keys = expected
                .keys()
                .chain(actual.keys())
                .collect::<BTreeSet<_>>();
            for key in keys {
                match (expected.get(key), actual.get(key)) {
                    (Some(expected), Some(actual)) => {
                        if let Some(difference) =
                            first_difference(&format!("{path}.{key}"), expected, actual)
                        {
                            return Some(difference);
                        }
                    }
                    (expected, actual) => {
                        return difference(&format!("{path}.{key}"), expected, actual);
                    }
                }
            }
            None
        }
        _ if expected == actual => None,
        _ => Some((path.to_string(), compact(expected), compact(actual))),
    }
}

fn difference(
    path: &str,
    expected: Option<&Value>,
    actual: Option<&Value>,
) -> Option<(String, String, String)> {
    Some((
        path.to_string(),
        expected.map_or_else(|| "<missing>".to_string(), compact),
        actual.map_or_else(|| "<missing>".to_string(), compact),
    ))
}

fn compact(value: &Value) -> String {
    serde_json::to_string(value).expect("JSON value must serialize")
}
