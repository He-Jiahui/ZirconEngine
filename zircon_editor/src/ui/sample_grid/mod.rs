mod generation;

pub(crate) use generation::{
    SampleGridGeneration, SampleGridGenerationInput, SampleGridPoint, SampleGridTick,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
