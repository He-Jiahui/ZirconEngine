use woc_protocol::{PRESENTATION_HZ, SIMULATION_HZ};

const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentationCadence {
    simulation_hz: u32,
    presentation_hz: u32,
    simulation_step_ns: u64,
    presentation_subframes_per_tick: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationCadenceError {
    ZeroRate,
    NonIntegralSimulationStep {
        simulation_hz: u32,
    },
    NonIntegralPresentationRatio {
        simulation_hz: u32,
        presentation_hz: u32,
    },
}

impl PresentationCadence {
    pub fn new(simulation_hz: u32, presentation_hz: u32) -> Result<Self, PresentationCadenceError> {
        if simulation_hz == 0 || presentation_hz == 0 {
            return Err(PresentationCadenceError::ZeroRate);
        }
        if NANOSECONDS_PER_SECOND % u64::from(simulation_hz) != 0 {
            return Err(PresentationCadenceError::NonIntegralSimulationStep { simulation_hz });
        }
        if presentation_hz % simulation_hz != 0 {
            return Err(PresentationCadenceError::NonIntegralPresentationRatio {
                simulation_hz,
                presentation_hz,
            });
        }
        Ok(Self {
            simulation_hz,
            presentation_hz,
            simulation_step_ns: NANOSECONDS_PER_SECOND / u64::from(simulation_hz),
            presentation_subframes_per_tick: presentation_hz / simulation_hz,
        })
    }

    pub fn woc_default() -> Self {
        Self::new(SIMULATION_HZ, PRESENTATION_HZ)
            .expect("the WOC protocol clocks must form an integral cadence")
    }

    pub fn simulation_hz(self) -> u32 {
        self.simulation_hz
    }

    pub fn presentation_hz(self) -> u32 {
        self.presentation_hz
    }

    pub fn simulation_step_ns(self) -> u64 {
        self.simulation_step_ns
    }

    pub fn presentation_subframes_per_tick(self) -> u32 {
        self.presentation_subframes_per_tick
    }
}

impl Default for PresentationCadence {
    fn default() -> Self {
        Self::woc_default()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PresentationSnapshot<T> {
    pub generation: u64,
    pub tick: u64,
    pub state_digest: u32,
    pub event_digest: u32,
    pub presentation_digest: u32,
    pub received_at_ns: u64,
    pub projection: T,
}

impl<T> PresentationSnapshot<T> {
    pub fn new(
        generation: u64,
        tick: u64,
        state_digest: u32,
        event_digest: u32,
        presentation_digest: u32,
        received_at_ns: u64,
        projection: T,
    ) -> Self {
        Self {
            generation,
            tick,
            state_digest,
            event_digest,
            presentation_digest,
            received_at_ns,
            projection,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationTimelinePush {
    Reset,
    Advanced,
    Duplicate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationTimelineError {
    GenerationRegressed {
        actual: u64,
        current: u64,
    },
    TickRegressed {
        generation: u64,
        actual: u64,
        current: u64,
    },
    ConflictingSnapshot {
        generation: u64,
        tick: u64,
    },
    ReceiptTimeRegressed {
        actual_ns: u64,
        current_ns: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationBlendMode {
    HoldCurrent,
    Interpolate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresentationSample<'a, T> {
    pub from: &'a T,
    pub to: &'a T,
    pub alpha: f32,
    pub mode: PresentationBlendMode,
}

pub struct PresentationTimeline<T> {
    cadence: PresentationCadence,
    previous: Option<PresentationSnapshot<T>>,
    current: Option<PresentationSnapshot<T>>,
}

impl<T> PresentationTimeline<T> {
    pub fn new(cadence: PresentationCadence) -> Self {
        Self {
            cadence,
            previous: None,
            current: None,
        }
    }

    pub fn push(
        &mut self,
        snapshot: PresentationSnapshot<T>,
    ) -> Result<PresentationTimelinePush, PresentationTimelineError> {
        let Some(current) = self.current.as_ref() else {
            self.current = Some(snapshot);
            return Ok(PresentationTimelinePush::Reset);
        };

        if snapshot.generation < current.generation {
            return Err(PresentationTimelineError::GenerationRegressed {
                actual: snapshot.generation,
                current: current.generation,
            });
        }
        if snapshot.generation == current.generation {
            if snapshot.tick < current.tick {
                return Err(PresentationTimelineError::TickRegressed {
                    generation: snapshot.generation,
                    actual: snapshot.tick,
                    current: current.tick,
                });
            }
            if snapshot.tick == current.tick {
                if snapshot.state_digest != current.state_digest
                    || snapshot.event_digest != current.event_digest
                    || snapshot.presentation_digest != current.presentation_digest
                {
                    return Err(PresentationTimelineError::ConflictingSnapshot {
                        generation: snapshot.generation,
                        tick: snapshot.tick,
                    });
                }
                return Ok(PresentationTimelinePush::Duplicate);
            }
        }
        if snapshot.received_at_ns < current.received_at_ns {
            return Err(PresentationTimelineError::ReceiptTimeRegressed {
                actual_ns: snapshot.received_at_ns,
                current_ns: current.received_at_ns,
            });
        }

        if snapshot.generation > current.generation {
            self.previous = None;
            self.current = Some(snapshot);
            return Ok(PresentationTimelinePush::Reset);
        }

        self.previous = self.current.take();
        self.current = Some(snapshot);
        Ok(PresentationTimelinePush::Advanced)
    }

    pub fn sample(&self, presentation_time_ns: u64) -> Option<PresentationSample<'_, T>> {
        let current = self.current.as_ref()?;
        let Some(previous) = self.previous.as_ref() else {
            return Some(hold_sample(&current.projection));
        };
        let elapsed = presentation_time_ns.saturating_sub(current.received_at_ns);
        if elapsed >= self.cadence.simulation_step_ns {
            return Some(hold_sample(&current.projection));
        }
        Some(PresentationSample {
            from: &previous.projection,
            to: &current.projection,
            alpha: elapsed as f32 / self.cadence.simulation_step_ns as f32,
            mode: PresentationBlendMode::Interpolate,
        })
    }

    pub fn previous(&self) -> Option<&PresentationSnapshot<T>> {
        self.previous.as_ref()
    }

    pub fn current(&self) -> Option<&PresentationSnapshot<T>> {
        self.current.as_ref()
    }

    pub fn cadence(&self) -> PresentationCadence {
        self.cadence
    }
}

fn hold_sample<T>(projection: &T) -> PresentationSample<'_, T> {
    PresentationSample {
        from: projection,
        to: projection,
        alpha: 1.0,
        mode: PresentationBlendMode::HoldCurrent,
    }
}
