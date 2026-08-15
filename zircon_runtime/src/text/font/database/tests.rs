use std::path::Path;
use std::sync::Arc;

use glyphon::{FontSystem, fontdb};
use ttf2woff2::{BrotliQuality, encode};

use super::*;
use crate::asset::{FontAsset, FontAssetFaceStyle, FontAssetFamilyMember, FontAssetRenderStrategy};
use crate::text::font::test_font_fixtures::{
    unique_font_fixture_path, write_ttc_fixture, write_weight_fixture,
};
use crate::text::{
    CompositeFontDescriptor, FontCultureTag, FontFaceDescriptor, FontFamilyName, FontQuery,
    FontScript, FontStretch, FontStyle, FontWeight, SubFontRange,
};

mod asset_lifecycle;
mod composite;
mod fallback;
mod matching;
mod performance;
mod sources;
mod system_policy;
mod variations;
