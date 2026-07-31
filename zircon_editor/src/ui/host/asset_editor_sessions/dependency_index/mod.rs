mod generation;
mod impact;

pub(crate) use generation::UiAssetDependencyGeneration;
pub(crate) use impact::UiAssetDependencyImpact;

#[cfg(test)]
mod tests;
