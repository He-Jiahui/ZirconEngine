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
            let first_element_index = elements.len();
            command.append_paint_elements(next_paint_order, metrics, &mut elements);
            // 单条命令可能展开为多项，后续命令从实际追加数量之后继续排序。
            next_paint_order += (elements.len() - first_element_index) as u64;
        }
        elements
    }
}
