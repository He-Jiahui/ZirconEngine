use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptSourceLocation {
    pub path: String,
    pub line: u32,
    pub column: u32,
}

impl ScriptSourceLocation {
    pub fn new(path: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            path: path.into(),
            line,
            column,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptDiagnostic {
    pub severity: ScriptDiagnosticSeverity,
    pub code: String,
    pub module: String,
    pub message: String,
    pub location: Option<ScriptSourceLocation>,
}

impl ScriptDiagnostic {
    pub fn new(
        severity: ScriptDiagnosticSeverity,
        code: impl Into<String>,
        module: impl Into<String>,
        message: impl Into<String>,
        location: Option<ScriptSourceLocation>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            module: module.into(),
            message: message.into(),
            location,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ScriptDiagnostic, ScriptDiagnosticSeverity, ScriptSourceLocation};

    #[test]
    fn script_diagnostic_json_round_trip_preserves_source_location() {
        let diagnostic = ScriptDiagnostic::new(
            ScriptDiagnosticSeverity::Error,
            "ZR2002",
            "game.player",
            "type mismatch",
            Some(ScriptSourceLocation::new("res://scripts/player.zr", 12, 4)),
        );

        let json = serde_json::to_string(&diagnostic).unwrap();
        let decoded = serde_json::from_str::<ScriptDiagnostic>(&json).unwrap();

        assert_eq!(decoded, diagnostic);
    }
}
