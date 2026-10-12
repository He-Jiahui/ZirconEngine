#[path = "generation.rs"]
mod generation;
#[path = "impact.rs"]
mod impact;

pub(crate) use generation::UiAssetDependencyGeneration;
pub(crate) use impact::UiAssetDependencyImpact;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
