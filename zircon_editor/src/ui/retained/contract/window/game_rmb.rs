use std::cell::RefCell;
use std::rc::Rc;

use zircon_runtime_interface::ui::{
    dispatch::UiPointerId, layout::UiPoint, surface::UiPointerButton,
};

use crate::core::play::PlayPreviewInputOwner;

use super::super::componentized_workbench_regions::authored_level_viewport;
use super::super::data::{
    FrameRect, HostDocumentDockSurfaceData, HostPresentationGeneration, HostWindowPresentationData,
};
use super::super::native_pointer::routing::{contains, PanePointerRoute, PanePointerTarget};
use super::super::native_pointer::NativePointerButtonState;
use super::UiHostWindow;

pub(in crate::ui::retained_host) enum GameRmbInput<'a> {
    Down(UiPoint),
    Up {
        point: UiPoint,
        owner: &'a PlayPreviewInputOwner,
        inside: bool,
    },
}

type Route = Box<dyn for<'a> Fn(GameRmbInput<'a>) -> Option<PlayPreviewInputOwner>>;

// Only the app's actual root registers. The window's strong clones share its port;
// the callback holds the app weakly and is disconnected by its owner on app drop.
pub(super) type GameRmbPort = Rc<RefCell<Option<Rc<RmbDispatch>>>>;

pub(super) struct RmbDispatch {
    route: Route,
    held: RefCell<Option<HeldRmb>>,
    edge: RefCell<Option<RmbEdge>>,
}

struct HeldRmb {
    pointer: UiPointerId,
    // None means the physical down was observed, but Play did not take this gesture.
    owner: Option<AcceptedRmb>,
}

struct AcceptedRmb {
    surface: String,
    pane: String,
    runtime: PlayPreviewInputOwner,
}

enum RmbEdge {
    Down(UiPointerId),
    Up(Option<AcceptedRmb>),
}

struct EdgeScope<'a> {
    edge: &'a RefCell<Option<RmbEdge>>,
    previous: Option<RmbEdge>,
}

impl Drop for EdgeScope<'_> {
    fn drop(&mut self) {
        self.edge.replace(self.previous.take());
    }
}

impl UiHostWindow {
    pub(in crate::ui::retained_host) fn on_game_rmb(
        &self,
        route: impl for<'a> Fn(GameRmbInput<'a>) -> Option<PlayPreviewInputOwner> + 'static,
    ) {
        self.game_rmb_port.replace(Some(Rc::new(RmbDispatch {
            route: Box::new(route),
            held: RefCell::new(None),
            edge: RefCell::new(None),
        })));
    }

    pub(in crate::ui::retained_host) fn disconnect_game_rmb(&self) {
        self.game_rmb_port.borrow_mut().take();
    }

    /// A real root/Game focus loss may retire ownership but not a physical up not yet seen.
    pub(in crate::ui::retained_host) fn take_game_rmb_owner(
        &self,
    ) -> Option<PlayPreviewInputOwner> {
        let port = self.game_rmb_port.borrow().clone()?;
        let mut held = port.held.borrow_mut();
        held.as_mut()?.owner.take().map(|owner| owner.runtime)
    }

    pub(super) fn with_native_game_rmb<T>(
        &self,
        pointer: UiPointerId,
        state: NativePointerButtonState,
        dispatch: impl FnOnce() -> T,
    ) -> T {
        let port = self.game_rmb_port.borrow().clone();
        let Some(port) = port else {
            return dispatch();
        };
        let edge = match state {
            NativePointerButtonState::Pressed => {
                port.held.replace(Some(HeldRmb {
                    pointer,
                    owner: None,
                }));
                Some(RmbEdge::Down(pointer))
            }
            NativePointerButtonState::Released => {
                // Take even when a real retained/modal/overlay route handles the up first.
                // An unobserved up has no edge here and keeps all original routing.
                port.held
                    .borrow_mut()
                    .take_if(|held| held.pointer == pointer)
                    .map(|held| RmbEdge::Up(held.owner))
            }
        };
        let _scope = EdgeScope {
            edge: &port.edge,
            previous: port.edge.replace(edge),
        };
        dispatch()
    }

    pub(in crate::ui::retained_host::host_contract) fn dispatch_game_rmb_pane(
        &self,
        generation: &HostPresentationGeneration,
        hit: Option<&PanePointerRoute<'_>>,
        state: NativePointerButtonState,
        button: UiPointerButton,
        x: f32,
        y: f32,
    ) -> bool {
        if button != UiPointerButton::Secondary {
            return false;
        }
        let Some(port) = self.game_rmb_port.borrow().clone() else {
            return false;
        };
        let edge = port.edge.borrow_mut().take();
        let presentation = generation.structure();
        let game = hit.and_then(|hit| match &hit.target {
            PanePointerTarget::GameViewport(surface) => Some((*surface, hit)),
            _ => None,
        });
        match (state, edge) {
            (NativePointerButtonState::Pressed, Some(RmbEdge::Down(pointer))) => {
                let Some((surface, hit)) = game else {
                    return false;
                };
                let Some((dock, _)) = authored_game_frame(presentation, surface)
                    .filter(|(_, frame)| *frame == hit.frame)
                else {
                    // Outside the canonical root document case, keep the whole legacy pair.
                    port.held.borrow_mut().take();
                    return false;
                };
                let owner =
                    (port.route)(GameRmbInput::Down(UiPoint::new(hit.local_x, hit.local_y))).map(
                        |runtime| AcceptedRmb {
                            surface: surface.into(),
                            pane: dock.pane.id.to_string(),
                            runtime,
                        },
                    );
                if let Some(held) = port
                    .held
                    .borrow_mut()
                    .as_mut()
                    .filter(|held| held.pointer == pointer)
                {
                    held.owner = owner;
                }
                true
            }
            (NativePointerButtonState::Released, Some(RmbEdge::Up(owner))) => {
                if let Some(owner) = owner {
                    let inside = game.is_some_and(|(surface, _)| surface == owner.surface);
                    if self.may_continue_game_rmb(generation, hit, inside, x, y) {
                        if let Some((dock, frame)) =
                            authored_game_frame(presentation, &owner.surface)
                        {
                            let point = UiPoint::new(x - frame.x, y - frame.y);
                            if dock.pane.id.as_str() == owner.pane
                                && point.x.is_finite()
                                && point.y.is_finite()
                            {
                                (port.route)(GameRmbInput::Up {
                                    point,
                                    owner: &owner.runtime,
                                    inside,
                                });
                                // A checked terminal never falls back, even on stale identity or a dispatch error.
                                return true;
                            }
                        }
                    }
                }
                // A known non-Game, blurred or unmappable tail may not fresh-hit the current
                // Game. An unrelated pane keeps its original route; no lifecycle event here.
                game.is_some()
            }
            _ => false,
        }
    }

    fn may_continue_game_rmb(
        &self,
        generation: &HostPresentationGeneration,
        hit: Option<&PanePointerRoute<'_>>,
        inside: bool,
        x: f32,
        y: f32,
    ) -> bool {
        // A popup's blocked row may be projected as no ordinary hit. Only extend outside
        // the original Game body when that ambiguity is absent. Down and inside Up are unchanged.
        if !inside
            && (generation.menu_state().open_menu_index >= 0
                || generation.page_overflow_menu_state().open
                || generation.dock_overflow_menu_state().open
                || !generation.workbench_hit_index().popup_rows().is_empty())
        {
            return false;
        }
        // Only ordinary bodies (or no pane) are behind an accepted Game terminal. Retained,
        // modal and authored workbench dispatch have already run; preserve other pane controls.
        if let Some(hit) = hit {
            if !matches!(
                hit.target,
                PanePointerTarget::GameViewport(_)
                    | PanePointerTarget::SceneViewport(_)
                    | PanePointerTarget::Hierarchy
                    | PanePointerTarget::Console
            ) {
                return false;
            }
        }
        let window = self.window();
        let size = window.size();
        window.is_visible() && x >= 0.0 && x < size.width as f32 && y >= 0.0 && y < size.height as f32
            // The ordinary pane lookup also reports an occluded floating surface as a miss.
            && !generation.structure().host_scene_data.floating_layer.floating_windows.iter()
                .any(|window| contains(&window.frame, x, y))
    }
}

fn authored_game_frame<'a>(
    presentation: &'a HostWindowPresentationData,
    surface: &str,
) -> Option<(&'a HostDocumentDockSurfaceData, FrameRect)> {
    if presentation.host_shell.native_floating_window_mode || surface.is_empty() {
        return None;
    }
    let mut documents = presentation
        .host_scene_data
        .document_surfaces()
        .iter()
        .filter(|dock| dock.pane.kind.as_str() == "Game" && dock.surface_key.as_str() == surface);
    let dock = documents.next()?;
    if documents.next().is_some() {
        return None;
    }
    Some((dock, authored_level_viewport(presentation, dock)?.frame))
}
