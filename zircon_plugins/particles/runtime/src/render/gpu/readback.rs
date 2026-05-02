use std::fmt;

use zircon_runtime::core::framework::render::RenderParticleGpuReadbackOutputs;

use super::program::PARTICLE_GPU_COUNTER_WORDS_BASE;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParticleGpuReadbackDecodeError {
    MissingCounterWords { expected: usize, actual: usize },
}

impl fmt::Display for ParticleGpuReadbackDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCounterWords { expected, actual } => write!(
                formatter,
                "particle GPU readback expected at least {expected} counter words, got {actual}"
            ),
        }
    }
}

impl std::error::Error for ParticleGpuReadbackDecodeError {}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParticleGpuCounterReadback {
    pub alive_count: u32,
    pub spawned_total: u32,
    pub debug_flags: u32,
    pub reserved: u32,
    pub per_emitter_spawned: Vec<u32>,
}

impl ParticleGpuCounterReadback {
    pub fn from_words(
        words: &[u32],
        emitter_count: u32,
    ) -> Result<Self, ParticleGpuReadbackDecodeError> {
        let expected = PARTICLE_GPU_COUNTER_WORDS_BASE as usize + emitter_count as usize;
        if words.len() < expected {
            return Err(ParticleGpuReadbackDecodeError::MissingCounterWords {
                expected,
                actual: words.len(),
            });
        }

        Ok(Self {
            alive_count: words[0],
            spawned_total: words[1],
            debug_flags: words[2],
            reserved: words[3],
            per_emitter_spawned: words[PARTICLE_GPU_COUNTER_WORDS_BASE as usize..expected].to_vec(),
        })
    }

    pub fn to_render_outputs(
        &self,
        indirect_draw_args: [u32; 4],
    ) -> RenderParticleGpuReadbackOutputs {
        RenderParticleGpuReadbackOutputs {
            alive_count: self.alive_count,
            spawned_total: self.spawned_total,
            debug_flags: self.debug_flags,
            per_emitter_spawned: self.per_emitter_spawned.clone(),
            indirect_draw_args,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuCpuParityReport {
    pub expected_cpu_live_particles: u32,
    pub gpu_alive_particles: u32,
    pub expected_cpu_spawned_particles: u32,
    pub gpu_spawned_particles: u32,
    mismatches: Vec<String>,
}

impl ParticleGpuCpuParityReport {
    pub fn compare_counts(
        expected_cpu_live_particles: u32,
        expected_cpu_spawned_particles: u32,
        readback: &ParticleGpuCounterReadback,
    ) -> Self {
        let mut mismatches = Vec::new();
        if expected_cpu_live_particles != readback.alive_count {
            mismatches.push(format!(
                "alive count CPU={} GPU={}",
                expected_cpu_live_particles, readback.alive_count
            ));
        }
        if expected_cpu_spawned_particles != readback.spawned_total {
            mismatches.push(format!(
                "spawned count CPU={} GPU={}",
                expected_cpu_spawned_particles, readback.spawned_total
            ));
        }

        Self {
            expected_cpu_live_particles,
            gpu_alive_particles: readback.alive_count,
            expected_cpu_spawned_particles,
            gpu_spawned_particles: readback.spawned_total,
            mismatches,
        }
    }

    pub fn matches(&self) -> bool {
        self.mismatches.is_empty()
    }

    pub fn mismatches(&self) -> &[String] {
        &self.mismatches
    }
}
