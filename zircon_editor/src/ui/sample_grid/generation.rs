use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SampleGridTick {
    value: f32,
    label: Arc<str>,
}

impl SampleGridTick {
    pub(crate) fn value(&self) -> f32 {
        self.value
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SampleGridPoint {
    x: f32,
    y: f32,
    label: Arc<str>,
    selected: bool,
}

impl SampleGridPoint {
    pub(crate) fn new(x: f32, y: f32, label: impl Into<String>, selected: bool) -> Self {
        Self {
            x,
            y,
            label: Arc::from(label.into()),
            selected,
        }
    }

    pub(crate) fn x(&self) -> f32 {
        self.x
    }

    pub(crate) fn y(&self) -> f32 {
        self.y
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    pub(crate) fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SampleGridGeneration {
    x_axis_label: Arc<str>,
    y_axis_label: Arc<str>,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    x_ticks: Arc<Vec<SampleGridTick>>,
    y_ticks: Arc<Vec<SampleGridTick>>,
    points: Arc<Vec<SampleGridPoint>>,
    static_generation: u64,
    dynamic_generation: u64,
}

pub(crate) struct SampleGridGenerationInput {
    pub(crate) x_axis_label: String,
    pub(crate) y_axis_label: String,
    pub(crate) x_min: f32,
    pub(crate) x_max: f32,
    pub(crate) y_min: f32,
    pub(crate) y_max: f32,
    pub(crate) x_ticks: Vec<f32>,
    pub(crate) y_ticks: Vec<f32>,
    pub(crate) points: Vec<SampleGridPoint>,
}

impl Default for SampleGridGeneration {
    fn default() -> Self {
        Self::new(SampleGridGenerationInput {
            x_axis_label: String::new(),
            y_axis_label: String::new(),
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
            x_ticks: Vec::new(),
            y_ticks: Vec::new(),
            points: Vec::new(),
        })
    }
}

impl SampleGridGeneration {
    pub(crate) fn new(input: SampleGridGenerationInput) -> Self {
        let x_axis_label: Arc<str> = Arc::from(input.x_axis_label);
        let y_axis_label: Arc<str> = Arc::from(input.y_axis_label);
        let x_ticks = shared_vec(
            input
                .x_ticks
                .into_iter()
                .map(SampleGridTick::from_value)
                .collect(),
        );
        let y_ticks = shared_vec(
            input
                .y_ticks
                .into_iter()
                .map(SampleGridTick::from_value)
                .collect(),
        );
        let points = shared_vec(input.points);
        let static_generation = static_generation(
            &x_axis_label,
            &y_axis_label,
            input.x_min,
            input.x_max,
            input.y_min,
            input.y_max,
            x_ticks.as_slice(),
            y_ticks.as_slice(),
        );
        let dynamic_generation = dynamic_generation(
            input.x_min,
            input.x_max,
            input.y_min,
            input.y_max,
            points.as_slice(),
        );
        Self {
            x_axis_label,
            y_axis_label,
            x_min: input.x_min,
            x_max: input.x_max,
            y_min: input.y_min,
            y_max: input.y_max,
            x_ticks,
            y_ticks,
            points,
            static_generation,
            dynamic_generation,
        }
    }

    pub(crate) fn x_axis_label(&self) -> &str {
        &self.x_axis_label
    }

    pub(crate) fn y_axis_label(&self) -> &str {
        &self.y_axis_label
    }

    pub(crate) fn x_min(&self) -> f32 {
        self.x_min
    }

    pub(crate) fn x_max(&self) -> f32 {
        self.x_max
    }

    pub(crate) fn y_min(&self) -> f32 {
        self.y_min
    }

    pub(crate) fn y_max(&self) -> f32 {
        self.y_max
    }

    pub(crate) fn x_ticks(&self) -> &[SampleGridTick] {
        self.x_ticks.as_slice()
    }

    pub(crate) fn y_ticks(&self) -> &[SampleGridTick] {
        self.y_ticks.as_slice()
    }

    pub(crate) fn points(&self) -> &[SampleGridPoint] {
        self.points.as_slice()
    }

    pub(crate) fn static_generation(&self) -> u64 {
        self.static_generation
    }

    pub(crate) fn dynamic_generation(&self) -> u64 {
        self.dynamic_generation
    }
}

fn shared_vec<T>(values: Vec<T>) -> Arc<Vec<T>> {
    Arc::new(values)
}

impl SampleGridTick {
    fn from_value(value: f32) -> Self {
        Self {
            value,
            label: Arc::from(format_tick(value)),
        }
    }
}

fn format_tick(value: f32) -> String {
    if value.fract().abs() < f32::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

fn static_generation(
    x_axis_label: &str,
    y_axis_label: &str,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    x_ticks: &[SampleGridTick],
    y_ticks: &[SampleGridTick],
) -> u64 {
    let mut generation = GenerationHash::new();
    generation.add_str(x_axis_label);
    generation.add_str(y_axis_label);
    for value in [x_min, x_max, y_min, y_max] {
        generation.add_f32(value);
    }
    generation.add_u64(x_ticks.len() as u64);
    for tick in x_ticks {
        generation.add_f32(tick.value);
    }
    generation.add_u64(y_ticks.len() as u64);
    for tick in y_ticks {
        generation.add_f32(tick.value);
    }
    generation.finish()
}

fn dynamic_generation(
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    points: &[SampleGridPoint],
) -> u64 {
    let mut generation = GenerationHash::new();
    for value in [x_min, x_max, y_min, y_max] {
        generation.add_f32(value);
    }
    generation.add_u64(points.len() as u64);
    for point in points {
        generation.add_f32(point.x);
        generation.add_f32(point.y);
        generation.add_str(&point.label);
        generation.add_byte(u8::from(point.selected));
    }
    generation.finish()
}

struct GenerationHash(u64);

impl GenerationHash {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    fn new() -> Self {
        Self(Self::FNV_OFFSET)
    }

    fn add_byte(&mut self, byte: u8) {
        self.0 ^= u64::from(byte);
        self.0 = self.0.wrapping_mul(Self::FNV_PRIME);
    }

    fn add_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.add_byte(*byte);
        }
    }

    fn add_f32(&mut self, value: f32) {
        self.add_bytes(&value.to_bits().to_le_bytes());
    }

    fn add_u64(&mut self, value: u64) {
        self.add_bytes(&value.to_le_bytes());
    }

    fn add_str(&mut self, value: &str) {
        self.add_u64(value.len() as u64);
        self.add_bytes(value.as_bytes());
    }

    fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
#[path = "generation/shared_vec_storage_tests.rs"]
mod shared_vec_storage_tests;
