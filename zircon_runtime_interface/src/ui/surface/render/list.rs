use serde::{Deserialize, Serialize};

use super::UiRenderCommand;
use super::UiPaintElement;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderList {
    pub commands: Vec<UiRenderCommand>,
}

impl UiRenderList {
    pub fn to_paint_elements(&self) -> Vec<UiPaintElement> {
        self.commands
            .iter()
            .enumerate()
            .flat_map(|(index, command)| command.to_paint_elements(index as u64 * 2))
            .collect()
    }
}
