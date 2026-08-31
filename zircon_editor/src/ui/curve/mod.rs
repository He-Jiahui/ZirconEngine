//! Renderer-neutral curve-editor contracts and coordinate transforms.
//!
//! Runtime assets retain curve evaluation and mutation authority. This module only defines the
//! view, selection, and coordinate vocabulary shared by animation and inspector toolkits.

mod canvas;
mod model;

#[cfg(test)]
mod tests;

pub use canvas::CurveCanvasTransform;
pub use model::{
    CurveBounds, CurveElementKind, CurveElementRef, CurveInterpolation, CurveKey, CurveModel,
    CurvePoint, CurveSelection, CurveView,
};
