use super::{TextDirection, TextLayoutMetrics, TextShapeRun};

#[derive(Clone, Debug, PartialEq)]
pub struct TextShapeResult {
    pub runs: Vec<TextShapeRun>,
    pub metrics: TextLayoutMetrics,
    pub resolved_direction: TextDirection,
}
