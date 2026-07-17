use std::collections::HashMap;

use crate::render_graph::{CompiledRenderGraph, RenderGraphResourceAccessKind};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CompiledRenderPipelineResourceWriteIndex {
    resource_indices: HashMap<String, usize>,
    write_bits: Box<[u64]>,
    #[cfg(test)]
    executable_pass_count: usize,
    #[cfg(test)]
    resource_access_count: usize,
}

impl CompiledRenderPipelineResourceWriteIndex {
    pub(super) fn from_graph(graph: &CompiledRenderGraph) -> Self {
        let mut resource_indices = HashMap::new();
        let mut resource_write_flags = Vec::new();
        let mut executable_pass_count = 0;
        let mut resource_access_count = 0;
        for pass in graph.passes().iter().filter(|pass| !pass.culled) {
            executable_pass_count += 1;
            for access in &pass.resources {
                resource_access_count += 1;
                let index = match resource_indices.get(access.name.as_str()).copied() {
                    Some(index) => index,
                    None => {
                        let index = resource_indices.len();
                        resource_indices.insert(access.name.clone(), index);
                        resource_write_flags.push(false);
                        index
                    }
                };
                if access.access == RenderGraphResourceAccessKind::Write {
                    resource_write_flags[index] = true;
                }
            }
        }
        let mut write_bits = vec![0_u64; resource_indices.len().div_ceil(64)];
        for (index, written) in resource_write_flags.into_iter().enumerate() {
            if written {
                write_bits[index / 64] |= 1_u64 << (index % 64);
            }
        }
        Self {
            resource_indices,
            write_bits: write_bits.into_boxed_slice(),
            #[cfg(test)]
            executable_pass_count,
            #[cfg(test)]
            resource_access_count,
        }
    }

    pub(super) fn contains(&self, resource_name: &str) -> bool {
        let Some(index) = self.resource_indices.get(resource_name).copied() else {
            return false;
        };
        self.write_bits
            .get(index / 64)
            .is_some_and(|word| word & (1_u64 << (index % 64)) != 0)
    }

    #[cfg(test)]
    pub(super) fn build_stats(&self) -> (usize, usize) {
        (self.executable_pass_count, self.resource_access_count)
    }

    #[cfg(test)]
    pub(super) fn storage_snapshot(&self) -> (usize, usize, usize) {
        (
            self.resource_indices.capacity(),
            self.write_bits.as_ptr() as usize,
            self.write_bits.len(),
        )
    }
}
