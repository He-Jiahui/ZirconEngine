#[path = "clip/builders.rs"]
mod builders;
#[path = "clip/validation.rs"]
mod validation;

pub(in crate::tests) use builders::{
    test_clip, test_clip_with_channels, test_clip_with_layout, test_clip_with_rate,
    test_stereo_clip_with_rate,
};
