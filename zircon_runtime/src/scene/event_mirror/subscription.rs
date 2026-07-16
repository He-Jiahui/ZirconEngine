use serde::Serialize;

use crate::scene::ecs::{Event, EventSubscription};
use crate::scene::World;

use super::{RuntimeEventMirrorError, RuntimeEventMirrorRegistration};

trait ErasedRuntimeEventMirrorSubscription: Send + Sync {
    fn connect(&mut self, world: &mut World) -> bool;
    fn disconnect(&mut self, world: &mut World) -> bool;
    fn drain(&mut self, world: &World) -> Result<Vec<serde_json::Value>, String>;
}

struct TypedRuntimeEventMirrorSubscription<E> {
    subscription: EventSubscription<E>,
}

impl<E> ErasedRuntimeEventMirrorSubscription for TypedRuntimeEventMirrorSubscription<E>
where
    E: Event + Serialize,
{
    fn connect(&mut self, world: &mut World) -> bool {
        world.connect_event_subscription(&mut self.subscription)
    }

    fn disconnect(&mut self, world: &mut World) -> bool {
        world.disconnect_event_subscription(&mut self.subscription)
    }

    fn drain(&mut self, world: &World) -> Result<Vec<serde_json::Value>, String> {
        world
            .read_event_subscription(&mut self.subscription)
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }
}

pub struct RuntimeEventMirrorSubscription {
    registration: Option<RuntimeEventMirrorRegistration>,
    erased: Box<dyn ErasedRuntimeEventMirrorSubscription>,
    connected: bool,
}

impl RuntimeEventMirrorSubscription {
    pub fn descriptor(&self) -> &super::RuntimeEventMirrorDescriptor {
        self.registration().descriptor()
    }

    pub(crate) fn typed<E>(subscription: EventSubscription<E>) -> Self
    where
        E: Event + Serialize,
    {
        Self {
            registration: None,
            erased: Box::new(TypedRuntimeEventMirrorSubscription { subscription }),
            connected: false,
        }
    }

    pub(crate) fn attach_registration(&mut self, registration: RuntimeEventMirrorRegistration) {
        self.registration = Some(registration);
    }

    pub(crate) fn connect(&mut self, world: &mut World) -> bool {
        let connected = self.erased.connect(world);
        self.connected |= connected;
        connected
    }

    pub(crate) fn disconnect(&mut self, world: &mut World) -> bool {
        let disconnected = self.erased.disconnect(world);
        if disconnected {
            self.connected = false;
        }
        disconnected
    }

    pub(crate) fn registration(&self) -> &RuntimeEventMirrorRegistration {
        self.registration
            .as_ref()
            .expect("runtime event mirror subscription has registration")
    }

    pub(crate) fn drain(
        &mut self,
        world: &World,
    ) -> Result<Vec<serde_json::Value>, RuntimeEventMirrorError> {
        let event_id = self.registration().descriptor().event_id.clone();
        if !self.connected {
            return Err(RuntimeEventMirrorError::Disconnected { event_id });
        }
        self.erased
            .drain(world)
            .map_err(|message| RuntimeEventMirrorError::Serialize { event_id, message })
    }
}
