use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Executable,
    DynamicLibrary,
    SymbolFile,
    Resource,
    Sbom,
}
