use std::io::{self, Write};

use crate::scene::dynamic_scene::DynamicSceneError;

use super::World;

impl World {
    pub(in crate::scene) fn estimate_dynamic_scene_preflight_bytes(
        &self,
        limit_bytes: usize,
    ) -> Result<usize, DynamicSceneError> {
        let fixed_bytes = std::mem::size_of::<World>();
        if fixed_bytes > limit_bytes {
            return Err(DynamicSceneError::TargetSnapshotTooLarge {
                estimated_bytes: fixed_bytes,
                limit_bytes,
            });
        }

        let mut counter = BoundedByteCounter::new(limit_bytes.saturating_sub(fixed_bytes));
        if let Err(error) = serde_json::to_writer(&mut counter, self) {
            if counter.exceeded {
                return Err(DynamicSceneError::TargetSnapshotTooLarge {
                    estimated_bytes: limit_bytes.saturating_add(1),
                    limit_bytes,
                });
            }
            return Err(DynamicSceneError::TargetSnapshotEstimation {
                reason: error.to_string(),
            });
        }
        let estimated_bytes = fixed_bytes.saturating_add(counter.bytes);
        if estimated_bytes > limit_bytes {
            return Err(DynamicSceneError::TargetSnapshotTooLarge {
                estimated_bytes,
                limit_bytes,
            });
        }
        Ok(estimated_bytes)
    }

    pub(in crate::scene) fn clone_for_dynamic_scene_staging(
        &mut self,
        limit_bytes: usize,
    ) -> Result<(Self, usize), DynamicSceneError> {
        let schedule = std::mem::take(&mut self.schedule);
        let removed_component_events = std::mem::take(&mut self.removed_component_events);
        let events = std::mem::take(&mut self.events);
        let event_mirrors = std::mem::take(&mut self.event_mirrors);
        let messages = std::mem::take(&mut self.messages);
        let observers = std::mem::take(&mut self.observers);
        let staged_lifecycle_events = std::mem::take(&mut self.staged_lifecycle_events);
        let record_staged_lifecycle_events =
            std::mem::replace(&mut self.record_staged_lifecycle_events, false);
        let command_queue = std::mem::take(&mut self.command_queue);
        let deferred_command_errors = std::mem::take(&mut self.deferred_command_errors);
        let ecs_frame_performance_diagnostics =
            std::mem::take(&mut self.ecs_frame_performance_diagnostics);
        let inspection_artifact_cache = std::mem::take(&mut self.inspection_artifact_cache);

        let result = (|| {
            let fixed_bytes = std::mem::size_of::<World>();
            let serialized_limit = limit_bytes.saturating_sub(fixed_bytes) / 2;
            let mut counter = BoundedByteCounter::new(serialized_limit);
            if let Err(error) = serde_json::to_writer(&mut counter, &*self) {
                if counter.exceeded {
                    return Err(DynamicSceneError::TargetSnapshotTooLarge {
                        estimated_bytes: limit_bytes.saturating_add(1),
                        limit_bytes,
                    });
                }
                return Err(DynamicSceneError::TargetSnapshotEstimation {
                    reason: error.to_string(),
                });
            }
            let estimated_bytes = counter.bytes.saturating_mul(2).saturating_add(fixed_bytes);
            if estimated_bytes > limit_bytes {
                return Err(DynamicSceneError::TargetSnapshotTooLarge {
                    estimated_bytes,
                    limit_bytes,
                });
            }
            Ok((self.clone(), estimated_bytes))
        })();

        self.schedule = schedule;
        self.removed_component_events = removed_component_events;
        self.events = events;
        self.event_mirrors = event_mirrors;
        self.messages = messages;
        self.observers = observers;
        self.staged_lifecycle_events = staged_lifecycle_events;
        self.record_staged_lifecycle_events = record_staged_lifecycle_events;
        self.command_queue = command_queue;
        self.deferred_command_errors = deferred_command_errors;
        self.ecs_frame_performance_diagnostics = ecs_frame_performance_diagnostics;
        self.inspection_artifact_cache = inspection_artifact_cache;

        let (mut snapshot, estimated_bytes) = result?;
        snapshot.staged_lifecycle_events.clear();
        snapshot.record_staged_lifecycle_events = true;
        Ok((snapshot, estimated_bytes))
    }
}

struct BoundedByteCounter {
    bytes: usize,
    limit: usize,
    exceeded: bool,
}

impl BoundedByteCounter {
    fn new(limit: usize) -> Self {
        Self {
            bytes: 0,
            limit,
            exceeded: false,
        }
    }
}

impl Write for BoundedByteCounter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let projected = self.bytes.saturating_add(buffer.len());
        if projected > self.limit {
            self.exceeded = true;
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "dynamic scene target snapshot byte limit exceeded",
            ));
        }
        self.bytes = projected;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::scene::{DynamicSceneError, Resource, World};

    #[derive(Debug, PartialEq, Eq)]
    struct LiveOnlyResource(u32);

    impl Resource for LiveOnlyResource {}

    #[test]
    fn dynamic_scene_staging_snapshot_restores_live_only_state_on_success_and_failure() {
        let mut world = World::empty();
        world.insert_resource(LiveOnlyResource(47));

        let (snapshot, _) = world
            .clone_for_dynamic_scene_staging(1024 * 1024)
            .expect("bounded persistent world should clone");

        assert_eq!(
            world.get_resource::<LiveOnlyResource>(),
            Some(&LiveOnlyResource(47))
        );
        assert_eq!(snapshot.get_resource::<LiveOnlyResource>(), None);

        let error = world
            .clone_for_dynamic_scene_staging(0)
            .expect_err("zero bytes must reject the target snapshot");
        assert!(matches!(
            error,
            DynamicSceneError::TargetSnapshotTooLarge { .. }
        ));
        assert_eq!(
            world.get_resource::<LiveOnlyResource>(),
            Some(&LiveOnlyResource(47))
        );
    }
}
