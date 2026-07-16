use std::collections::{BTreeMap, HashMap};

use zircon_runtime::core::framework::ai::{
    AiPerceptionSense, AiPerceptionSnapshot, AiPerceptionStimulus,
};
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::ecs::Resource;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct StimulusKey {
    source: EntityId,
    sense: AiPerceptionSense,
}

#[derive(Debug, Default)]
pub struct PerceivedStimuli {
    by_receiver: BTreeMap<EntityId, HashMap<StimulusKey, AiPerceptionStimulus>>,
    scan_cursor: usize,
}

impl Resource for PerceivedStimuli {}

impl PerceivedStimuli {
    pub fn snapshot(&self, receiver: EntityId) -> Option<AiPerceptionSnapshot> {
        self.by_receiver
            .get(&receiver)
            .map(|stimuli| snapshot(receiver, stimuli))
    }

    pub fn snapshots(&self) -> Vec<AiPerceptionSnapshot> {
        self.by_receiver
            .iter()
            .map(|(receiver, stimuli)| snapshot(*receiver, stimuli))
            .collect()
    }

    pub(crate) fn begin_frame(
        &mut self,
        delta_seconds: Real,
        receivers: &[(EntityId, Real)],
    ) -> usize {
        let delta_seconds = if delta_seconds.is_finite() {
            delta_seconds.max(0.0)
        } else {
            0.0
        };
        let forget_by_receiver = receivers.iter().copied().collect::<HashMap<_, _>>();
        self.by_receiver
            .retain(|receiver, _| forget_by_receiver.contains_key(receiver));
        for (receiver, _) in receivers {
            self.by_receiver.entry(*receiver).or_default();
        }

        let before = stimulus_count(&self.by_receiver);
        for (receiver, stimuli) in &mut self.by_receiver {
            let forget_seconds = forget_by_receiver
                .get(receiver)
                .copied()
                .unwrap_or_default()
                .max(0.0);
            stimuli.retain(|_, stimulus| {
                stimulus.age_seconds += delta_seconds;
                stimulus.age_seconds < forget_seconds
            });
        }
        before.saturating_sub(stimulus_count(&self.by_receiver))
    }

    pub(crate) fn refresh(&mut self, receiver: EntityId, stimulus: AiPerceptionStimulus) {
        let key = StimulusKey {
            source: stimulus.source,
            sense: stimulus.sense,
        };
        let stimuli = self.by_receiver.entry(receiver).or_default();
        match stimuli.get_mut(&key) {
            Some(current) if current.age_seconds <= stimulus.age_seconds => {}
            Some(current) => *current = stimulus,
            None => {
                stimuli.insert(key, stimulus);
            }
        }
    }

    pub(crate) fn scan_cursor(&self, pair_slot_count: usize) -> usize {
        if pair_slot_count == 0 {
            0
        } else {
            self.scan_cursor % pair_slot_count
        }
    }

    pub(crate) fn set_scan_cursor(&mut self, scan_cursor: usize) {
        self.scan_cursor = scan_cursor;
    }
}

fn snapshot(
    receiver: EntityId,
    stimuli: &HashMap<StimulusKey, AiPerceptionStimulus>,
) -> AiPerceptionSnapshot {
    let mut stimuli = stimuli.values().cloned().collect::<Vec<_>>();
    stimuli.sort_by_key(|stimulus| (sense_rank(stimulus.sense), stimulus.source));
    AiPerceptionSnapshot {
        agent: receiver,
        stimuli,
    }
}

fn sense_rank(sense: AiPerceptionSense) -> u8 {
    match sense {
        AiPerceptionSense::Sight => 0,
        AiPerceptionSense::Hearing => 1,
        AiPerceptionSense::Damage => 2,
        AiPerceptionSense::Touch => 3,
        AiPerceptionSense::Custom => 4,
    }
}

fn stimulus_count(
    by_receiver: &BTreeMap<EntityId, HashMap<StimulusKey, AiPerceptionStimulus>>,
) -> usize {
    by_receiver.values().map(HashMap::len).sum()
}
