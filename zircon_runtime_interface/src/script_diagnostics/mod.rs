mod diagnostic;

pub use diagnostic::{ScriptDiagnostic, ScriptDiagnosticSeverity, ScriptSourceLocation};

#[cfg(test)]
mod tests;
