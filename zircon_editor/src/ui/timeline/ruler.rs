use super::TimelineRange;

const DEFAULT_MIN_LABEL_SPACING: f32 = 80.0;
const MAX_RULER_TICKS: usize = 4_096;

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineRulerTick {
    pub time: f32,
    pub label: String,
}

pub fn build_timeline_ruler_ticks(
    range: TimelineRange,
    pixel_width: f32,
    minimum_label_spacing: f32,
) -> Vec<TimelineRulerTick> {
    if range.duration() == 0.0 {
        return vec![TimelineRulerTick {
            time: range.start,
            label: format_time(range.start),
        }];
    }

    let spacing = if minimum_label_spacing.is_finite() && minimum_label_spacing > 0.0 {
        minimum_label_spacing
    } else {
        DEFAULT_MIN_LABEL_SPACING
    };
    let width = pixel_width
        .is_finite()
        .then_some(pixel_width)
        .unwrap_or_default();
    let desired_intervals = (width / spacing).floor().max(1.0);
    let step = nice_step(range.duration() / desired_intervals);
    let mut time = (range.start / step).ceil() * step;
    let mut ticks = Vec::new();
    while time <= range.end + step * 0.0001 && ticks.len() < MAX_RULER_TICKS {
        let clamped = range.clamp(time);
        if ticks
            .last()
            .is_none_or(|previous: &TimelineRulerTick| previous.time != clamped)
        {
            ticks.push(TimelineRulerTick {
                time: clamped,
                label: format_time(clamped),
            });
        }
        time += step;
    }
    if ticks.first().is_none_or(|tick| tick.time != range.start) {
        ticks.insert(
            0,
            TimelineRulerTick {
                time: range.start,
                label: format_time(range.start),
            },
        );
    }
    if ticks.last().is_none_or(|tick| tick.time != range.end) {
        ticks.push(TimelineRulerTick {
            time: range.end,
            label: format_time(range.end),
        });
    }
    ticks
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineSnapSettings {
    grid_step: Option<f32>,
    threshold: f32,
}

impl TimelineSnapSettings {
    pub fn new(grid_step: Option<f32>, threshold: f32) -> Self {
        Self {
            grid_step: grid_step.filter(|step| step.is_finite() && *step > 0.0),
            threshold: threshold
                .is_finite()
                .then_some(threshold.max(0.0))
                .unwrap_or_default(),
        }
    }

    pub fn snap(self, requested: f32, range: TimelineRange, authored_boundaries: &[f32]) -> f32 {
        let requested = range.clamp(requested);
        let mut best = None::<(f32, f32)>;
        let mut consider = |candidate: f32| {
            if !candidate.is_finite() {
                return;
            }
            let candidate = range.clamp(candidate);
            let distance = (candidate - requested).abs();
            if distance > self.threshold {
                return;
            }
            match best {
                Some((best_candidate, best_distance))
                    if distance > best_distance
                        || (distance == best_distance && candidate >= best_candidate) => {}
                _ => best = Some((candidate, distance)),
            }
        };

        if let Some(step) = self.grid_step {
            let units = ((requested - range.start) / step).round();
            consider(range.start + units * step);
        }
        for boundary in authored_boundaries {
            consider(*boundary);
        }
        best.map(|(candidate, _)| candidate).unwrap_or(requested)
    }
}

fn nice_step(raw_step: f32) -> f32 {
    if !raw_step.is_finite() || raw_step <= 0.0 {
        return 1.0;
    }
    let exponent = raw_step.log10().floor();
    let magnitude = 10_f32.powf(exponent);
    let normalized = raw_step / magnitude;
    let nice = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

fn format_time(time: f32) -> String {
    if (time - time.round()).abs() < 0.0001 {
        format!("{time:.0}")
    } else {
        format!("{time:.3}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}
