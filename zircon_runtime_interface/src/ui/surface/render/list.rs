use serde::{Deserialize, Serialize};

use crate::ui::layout::UiLayoutMetrics;

use super::UiPaintElement;
use super::UiRenderCommand;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderList {
    pub commands: Vec<UiRenderCommand>,
}

impl UiRenderList {
    pub fn to_paint_elements(&self) -> Vec<UiPaintElement> {
        self.to_paint_elements_with_metrics(UiLayoutMetrics::default())
    }

    pub fn to_paint_elements_with_metrics(&self, metrics: UiLayoutMetrics) -> Vec<UiPaintElement> {
        let mut elements = Vec::new();
        let mut next_paint_order = 0;
        for command in &self.commands {
            let mut command_elements =
                command.to_paint_elements_with_metrics(next_paint_order, metrics);
            next_paint_order += command_elements.len() as u64;
            elements.append(&mut command_elements);
        }
        elements
    }
}
