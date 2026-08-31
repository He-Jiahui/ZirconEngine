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
