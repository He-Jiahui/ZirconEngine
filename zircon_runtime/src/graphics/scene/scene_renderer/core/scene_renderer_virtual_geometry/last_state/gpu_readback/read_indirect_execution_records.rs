#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_execution_records(
        &self,
    ) -> Result<Vec<(u32, u64, u32, u32, u32)>, GraphicsError> {
        let execution_authority_records =
            self.read_last_virtual_geometry_indirect_execution_authority_records()?;
        if !execution_authority_records.is_empty() {
            return Ok(execution_authority_records
                .into_iter()
                .map(|record| record.execution_record())
                .collect());
        }
        let authority_records = self.read_last_virtual_geometry_indirect_authority_records()?;
        if authority_records.is_empty() {
            return Ok(Vec::new());
        }
        let authority_by_draw_ref_index = authority_records
            .into_iter()
            .map(|record| (record.draw_ref_index(), record.execution_record()))
            .collect::<std::collections::HashMap<_, _>>();
        Ok(self
            .read_last_virtual_geometry_indirect_execution_draw_ref_indices()?
            .into_iter()
            .filter_map(|draw_ref_index| authority_by_draw_ref_index.get(&draw_ref_index).copied())
            .collect())
    }
}
