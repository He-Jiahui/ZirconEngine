mod generation;

pub(crate) use generation::{
    SampleGridGeneration, SampleGridGenerationInput, SampleGridPoint, SampleGridTick,
};

#[cfg(test)]
mod tests;
