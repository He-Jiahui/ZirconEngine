use std::path::Path;
use std::sync::Arc;

use glyphon::{fontdb, FontSystem};
use ttf2woff2::{encode, BrotliQuality};

use super::*;
use crate::asset::{FontAsset, FontAssetFaceStyle, FontAssetFamilyMember, FontAssetRenderStrategy};
use crate::text::font::test_font_fixtures::{
    unique_font_fixture_path, write_ttc_fixture, write_weight_fixture,
};
use crate::text::{
    CompositeFontDescriptor, FontCultureTag, FontFaceDescriptor, FontFamilyName, FontQuery,
    FontScript, FontStretch, FontStyle, FontWeight, SubFontRange,
};

#[path = "asset_lifecycle.rs"]
mod asset_lifecycle;
#[path = "composite.rs"]
mod composite;
#[path = "fallback.rs"]
mod fallback;
#[path = "matching.rs"]
mod matching;
#[path = "performance.rs"]
mod performance;
#[path = "sources.rs"]
mod sources;
#[path = "system_policy.rs"]
mod system_policy;
#[path = "variations.rs"]
mod variations;
