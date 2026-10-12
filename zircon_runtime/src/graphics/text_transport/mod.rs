//! Transport adapters between public UI DTOs and neutral runtime text contracts.

#[path = "conversion.rs"]
mod conversion;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
