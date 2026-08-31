use std::error::Error;
use std::fmt;

use crate::core::framework::window::{NativeWindowId, WindowId};
use zircon_runtime_interface::ZrRuntimeViewportHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WindowRegistryError {
    RegistryIdentityExhausted,
    DuplicateNativeWindow {
        native_window: NativeWindowId,
    },
    InconsistentNativeWindowMapping {
        native_window: NativeWindowId,
        slot: u32,
    },
    UnknownNativeWindow {
        native_window: NativeWindowId,
    },
    UnknownWindow {
        window: WindowId,
    },
    StaleWindow {
        window: WindowId,
    },
    ClosingWindow {
        window: WindowId,
    },
    WindowNotClosing {
        window: WindowId,
    },
    InconsistentCloseTransaction {
        window: WindowId,
    },
    WindowHasLiveChildren {
        window: WindowId,
        child_count: usize,
    },
    WindowRelationshipCycle {
        child: WindowId,
        parent: WindowId,
    },
    InconsistentWindowRelationship {
        parent: WindowId,
        child: WindowId,
    },
    InvalidViewport {
        viewport: ZrRuntimeViewportHandle,
    },
    ViewportAlreadyBound {
        viewport: ZrRuntimeViewportHandle,
        window: WindowId,
    },
    ViewportBoundToDifferentWindow {
        viewport: ZrRuntimeViewportHandle,
        expected_window: WindowId,
        observed_window: WindowId,
    },
    UnknownViewportBinding {
        viewport: ZrRuntimeViewportHandle,
    },
    InconsistentViewportBinding {
        window: WindowId,
        viewport: ZrRuntimeViewportHandle,
    },
    WindowHasLiveViewportBindings {
        window: WindowId,
        viewport_count: usize,
    },
    PrimaryRoleGenerationExhausted,
    RelationshipCapacityExhausted,
    SlotCapacityExhausted,
}

impl fmt::Display for WindowRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegistryIdentityExhausted => write!(
                formatter,
                "platform window registry identity space is exhausted"
            ),
            Self::DuplicateNativeWindow { native_window } => write!(
                formatter,
                "native window {} is already registered in this platform host",
                native_window.raw()
            ),
            Self::InconsistentNativeWindowMapping {
                native_window,
                slot,
            } => write!(
                formatter,
                "native window {} has an inconsistent registry slot mapping to slot {slot}",
                native_window.raw()
            ),
            Self::UnknownNativeWindow { native_window } => write!(
                formatter,
                "native window {} is not registered in this platform host",
                native_window.raw()
            ),
            Self::UnknownWindow { window } => write!(
                formatter,
                "window registry {} does not own slot {}",
                window.registry().raw(),
                window.slot()
            ),
            Self::StaleWindow { window } => write!(
                formatter,
                "window registry {} slot {} generation {} is stale",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::ClosingWindow { window } => write!(
                formatter,
                "window registry {} slot {} generation {} is closing",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::WindowNotClosing { window } => write!(
                formatter,
                "window registry {} slot {} generation {} must enter closing before destruction",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::InconsistentCloseTransaction { window } => write!(
                formatter,
                "window registry {} slot {} generation {} produced an incomplete close transaction",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::WindowHasLiveChildren {
                window,
                child_count,
            } => write!(
                formatter,
                "window registry {} slot {} generation {} retains {child_count} live child windows",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::WindowRelationshipCycle { child, parent } => write!(
                formatter,
                "window {}:{}:{} cannot become a child of {}:{}:{} because the relationship would cycle",
                child.registry().raw(),
                child.slot(),
                child.generation(),
                parent.registry().raw(),
                parent.slot(),
                parent.generation()
            ),
            Self::InconsistentWindowRelationship { parent, child } => write!(
                formatter,
                "window registry relationship {}:{}:{} does not retain child {}:{}:{} exactly once",
                parent.registry().raw(),
                parent.slot(),
                parent.generation(),
                child.registry().raw(),
                child.slot(),
                child.generation()
            ),
            Self::InvalidViewport { viewport } => write!(
                formatter,
                "viewport {} is not a valid runtime viewport handle",
                viewport.raw()
            ),
            Self::ViewportAlreadyBound { viewport, window } => write!(
                formatter,
                "viewport {} is already bound to window registry {} slot {} generation {}",
                viewport.raw(),
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::ViewportBoundToDifferentWindow {
                viewport,
                expected_window,
                observed_window,
            } => write!(
                formatter,
                "viewport {} is bound to window registry {} slot {} generation {} instead of expected window registry {} slot {} generation {}",
                viewport.raw(),
                observed_window.registry().raw(),
                observed_window.slot(),
                observed_window.generation(),
                expected_window.registry().raw(),
                expected_window.slot(),
                expected_window.generation()
            ),
            Self::UnknownViewportBinding { viewport } => write!(
                formatter,
                "viewport {} is not bound to a routable platform window",
                viewport.raw()
            ),
            Self::InconsistentViewportBinding { window, viewport } => write!(
                formatter,
                "viewport {} has inconsistent binding ownership for window registry {} slot {} generation {}",
                viewport.raw(),
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::WindowHasLiveViewportBindings {
                window,
                viewport_count,
            } => write!(
                formatter,
                "window registry {} slot {} generation {} retains {viewport_count} live viewport bindings",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::PrimaryRoleGenerationExhausted => write!(
                formatter,
                "platform window registry exhausted primary role generations"
            ),
            Self::RelationshipCapacityExhausted => write!(
                formatter,
                "platform window registry exhausted relationship graph capacity"
            ),
            Self::SlotCapacityExhausted => {
                write!(
                    formatter,
                    "platform window registry exhausted its addressable slots"
                )
            }
        }
    }
}

impl Error for WindowRegistryError {}
