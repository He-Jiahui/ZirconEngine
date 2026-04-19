//! Channel and dynamic service object aliases.

use std::any::Any;
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};

pub type ChannelSender<T> = Sender<T>;
pub type ChannelReceiver<T> = Receiver<T>;
pub type ServiceObject = Arc<dyn Any + Send + Sync>;
