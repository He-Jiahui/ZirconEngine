use super::{HybridGiRadianceCacheSample, HybridGiRadianceCacheSource};

#[derive(Debug)]
pub(super) struct HybridGiRadianceCacheInterpolationAccumulator {
    total_corner_weight: u64,
    weighted_confidence: u64,
    weighted_radiance: [u64; 3],
    dominant_source_weight: u64,
    dominant_source: HybridGiRadianceCacheSource,
}

impl Default for HybridGiRadianceCacheInterpolationAccumulator {
    fn default() -> Self {
        Self {
            total_corner_weight: 0,
            weighted_confidence: 0,
            weighted_radiance: [0; 3],
            dominant_source_weight: 0,
            dominant_source: HybridGiRadianceCacheSource::Missing,
        }
    }
}

impl HybridGiRadianceCacheInterpolationAccumulator {
    pub(super) fn add(&mut self, sample: HybridGiRadianceCacheSample, corner_weight: u64) {
        self.total_corner_weight = self.total_corner_weight.saturating_add(corner_weight);
        let contribution_weight = corner_weight.saturating_mul(u64::from(sample.confidence_q8));
        self.weighted_confidence = self.weighted_confidence.saturating_add(contribution_weight);
        for (component, radiance) in self.weighted_radiance.iter_mut().zip(sample.radiance_rgb) {
            *component =
                component.saturating_add(contribution_weight.saturating_mul(u64::from(radiance)));
        }
        if contribution_weight > self.dominant_source_weight {
            self.dominant_source_weight = contribution_weight;
            self.dominant_source = sample.source;
        }
    }

    pub(super) fn finish(self) -> HybridGiRadianceCacheSample {
        if self.total_corner_weight == 0 || self.weighted_confidence == 0 {
            return HybridGiRadianceCacheSample::MISSING;
        }
        HybridGiRadianceCacheSample {
            // Keep known radiance normalized while confidence reflects the complete neighborhood.
            radiance_rgb: self.weighted_radiance.map(|component| {
                rounded_divide(component, self.weighted_confidence).min(u64::from(u8::MAX)) as u8
            }),
            confidence_q8: rounded_divide(self.weighted_confidence, self.total_corner_weight)
                .min(u64::from(u8::MAX)) as u8,
            source: self.dominant_source,
        }
    }
}

fn rounded_divide(numerator: u64, denominator: u64) -> u64 {
    numerator.saturating_add(denominator / 2) / denominator
}
