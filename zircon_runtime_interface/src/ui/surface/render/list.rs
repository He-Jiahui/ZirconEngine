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
        self.commands
            .iter()
            .enumerate()
            .flat_map(|(index, command)| {
                command.to_paint_elements_with_metrics(index as u64 * 2, metrics)
            })
            .collect()
    }
}
