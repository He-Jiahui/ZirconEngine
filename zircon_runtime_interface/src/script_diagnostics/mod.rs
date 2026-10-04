mod diagnostic;

pub use diagnostic::{ScriptDiagnostic, ScriptDiagnosticSeverity, ScriptSourceLocation};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
