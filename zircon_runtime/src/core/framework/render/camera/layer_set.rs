use serde::{Deserialize, Serialize};

use super::{RenderLayer, DEFAULT_RENDER_LAYER};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RenderLayerSet {
    #[serde(default)]
    blocks: Vec<u64>,
}

impl Default for RenderLayerSet {
    fn default() -> Self {
        Self::layer(DEFAULT_RENDER_LAYER)
    }
}

impl RenderLayerSet {
    pub fn layer(layer: RenderLayer) -> Self {
        Self::none().with(layer)
    }

    pub fn none() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn from_layers(layers: impl IntoIterator<Item = RenderLayer>) -> Self {
        layers
            .into_iter()
            .fold(Self::none(), |layers, layer| layers.with(layer))
    }

    pub fn from_scene_schema_v1_mask(mask: u32) -> Self {
        if mask == 0 {
            Self::none()
        } else {
            Self {
                blocks: vec![u64::from(mask)],
            }
        }
    }

    pub fn to_scene_schema_v1_mask_lossy(&self) -> u32 {
        self.blocks.first().copied().unwrap_or_default() as u32
    }

    pub fn with(mut self, layer: RenderLayer) -> Self {
        let block_index = layer_block_index(layer);
        if self.blocks.len() <= block_index {
            self.blocks.resize(block_index + 1, 0);
        }
        self.blocks[block_index] |= layer_bit(layer);
        self
    }

    pub fn without(mut self, layer: RenderLayer) -> Self {
        let block_index = layer_block_index(layer);
        if let Some(block) = self.blocks.get_mut(block_index) {
            *block &= !layer_bit(layer);
        }
        self.shrink()
    }

    pub fn contains(&self, layer: RenderLayer) -> bool {
        self.blocks
            .get(layer_block_index(layer))
            .is_some_and(|block| (*block & layer_bit(layer)) != 0)
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|block| *block == 0)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        self.blocks
            .iter()
            .zip(other.blocks.iter())
            .any(|(left, right)| (*left & *right) != 0)
    }

    pub fn union(&self, other: &Self) -> Self {
        let block_count = self.blocks.len().max(other.blocks.len());
        let blocks = (0..block_count)
            .map(|index| {
                self.blocks.get(index).copied().unwrap_or_default()
                    | other.blocks.get(index).copied().unwrap_or_default()
            })
            .collect::<Vec<_>>();
        Self { blocks }.shrink()
    }

    pub fn intersects_scene_schema_v1_mask(&self, mask: u32) -> bool {
        (self.blocks.first().copied().unwrap_or_default() & u64::from(mask)) != 0
    }

    pub fn iter(&self) -> impl Iterator<Item = RenderLayer> + '_ {
        self.blocks
            .iter()
            .enumerate()
            .flat_map(|(block_index, block)| {
                let mut block = *block;
                std::iter::from_fn(move || {
                    if block == 0 {
                        return None;
                    }
                    let bit = block.trailing_zeros();
                    block &= !(1u64 << bit);
                    Some((block_index as RenderLayer) * u64::BITS + bit)
                })
            })
    }

    fn shrink(mut self) -> Self {
        while self.blocks.last().is_some_and(|block| *block == 0) {
            self.blocks.pop();
        }
        self
    }
}

fn layer_block_index(layer: RenderLayer) -> usize {
    (layer / u64::BITS) as usize
}

fn layer_bit(layer: RenderLayer) -> u64 {
    1u64 << (layer % u64::BITS)
}
