#[path = "rects/fields.rs"]
mod fields;
#[path = "rects/frame.rs"]
mod frame;
#[path = "rects/top.rs"]
mod top;

pub(in super::super) use fields::{
    location_fallback, name_fallback, preview_fallback, validation_fallback,
};
pub(in super::super) use top::{header_fallback, hero_fallback, status_fallback};
