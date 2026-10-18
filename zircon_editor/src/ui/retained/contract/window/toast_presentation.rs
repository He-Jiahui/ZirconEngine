use std::cell::RefCell;
use std::rc::Rc;

use super::super::data::{
    FrameRect, HostPresentationGenerationCursor, HostWindowPresentationData, TemplatePaneNodeData,
};
use super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::super::presenter::{HostChromePresenter, HostPresenterError};
use super::UiHostWindow;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ToastIdentity {
    id: String,
    token: u64,
}

/// Evidence belongs to this session and the immutable generation actually painted.
pub(in crate::ui::retained_host::host_contract) struct ToastPresentationReceipt {
    session: usize,
    cursor: HostPresentationGenerationCursor,
    identity: ToastIdentity,
}
struct PaintAttempt {
    session: usize,
    cursor: HostPresentationGenerationCursor,
    expected: Option<ToastIdentity>,
    painted: Option<ToastIdentity>,
}
thread_local! {
    static ACTIVE_PAINT: RefCell<Option<PaintAttempt>> = const { RefCell::new(None) };
}
struct PaintScope {
    previous: Option<PaintAttempt>,
}
impl PaintScope {
    fn enter(
        session: usize,
        cursor: HostPresentationGenerationCursor,
        expected: Option<ToastIdentity>,
    ) -> Self {
        Self {
            previous: ACTIVE_PAINT.with(|active| {
                active.replace(Some(PaintAttempt {
                    session,
                    cursor,
                    expected,
                    painted: None,
                }))
            }),
        }
    }
    fn receipt(&self) -> Option<ToastPresentationReceipt> {
        ACTIVE_PAINT.with(|active| {
            let mut active = active.borrow_mut();
            let attempt = active.as_mut()?;
            Some(ToastPresentationReceipt {
                session: attempt.session,
                cursor: attempt.cursor,
                identity: attempt.painted.take()?,
            })
        })
    }
}
impl Drop for PaintScope {
    fn drop(&mut self) {
        ACTIVE_PAINT.with(|active| {
            active.replace(self.previous.take());
        });
    }
}
fn visible_intersection(rect: &FrameRect, clip: &FrameRect) -> bool {
    [
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        clip.x,
        clip.y,
        clip.width,
        clip.height,
    ]
    .iter()
    .all(|value| value.is_finite())
        && rect.width > 0.0
        && rect.height > 0.0
        && clip.width > 0.0
        && clip.height > 0.0
        && rect.x < clip.x + clip.width
        && clip.x < rect.x + rect.width
        && rect.y < clip.y + clip.height
        && clip.y < rect.y + rect.height
}
fn identity(node: &TemplatePaneNodeData) -> Option<ToastIdentity> {
    let (id, token) = node.toast_presentation.as_ref()?;
    (!id.is_empty() && *token != 0).then(|| ToastIdentity {
        id: id.to_string(),
        token: *token,
    })
}

/// Called only after push_toast emitted commands for a visible, nontransparent surface intersection.
pub(in crate::ui::retained_host::host_contract) fn record_toast_paint(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    opacity: f32,
    emitted_commands: bool,
) {
    if !emitted_commands
        || !opacity.is_finite()
        || opacity <= 0.0
        || !visible_intersection(rect, clip)
    {
        return;
    }
    let Some(identity) = identity(node) else {
        return;
    };
    ACTIVE_PAINT.with(|active| {
        if let Some(attempt) = active.borrow_mut().as_mut() {
            if attempt.expected.as_ref() == Some(&identity) {
                attempt.painted = Some(identity);
            }
        }
    });
}

/// The exclusive presenter borrow makes both counters refer to this one ordinary present.
/// Native resize, snapshots and capture-only no-ops never independently acknowledge delivery.
pub(super) fn present_with_toast_confirmation(
    session: usize,
    presenter: &mut dyn HostChromePresenter,
    presentation: &HostWindowPresentationData,
    cursor: HostPresentationGenerationCursor,
    damage: Option<FrameRect>,
    invalidation: HostInvalidationDiagnostics,
    visible: bool,
) -> (
    Result<HostRefreshDiagnostics, HostPresenterError>,
    Option<ToastPresentationReceipt>,
    bool,
) {
    let expected = visible
        .then(|| {
            presentation.workbench_window_nodes.iter().find_map(|node| {
                let rect = FrameRect {
                    x: node.frame.x,
                    y: node.frame.y,
                    width: node.frame.width,
                    height: node.frame.height,
                };
                let clip = if node.has_clip_frame {
                    FrameRect {
                        x: node.clip_frame.x,
                        y: node.clip_frame.y,
                        width: node.clip_frame.width,
                        height: node.clip_frame.height,
                    }
                } else {
                    rect.clone()
                };
                visible_intersection(&rect, &clip)
                    .then(|| identity(&node))
                    .flatten()
            })
        })
        .flatten();
    let awaiting = expected.is_some();
    let scope = PaintScope::enter(session, cursor, expected);
    let before = presenter.diagnostics_snapshot().present_count;
    // A pending head must enter the paint path even if an older draw list was retained.
    let result = presenter.present(
        presentation,
        cursor,
        if awaiting { None } else { damage },
        invalidation,
    );
    let after = presenter.diagnostics_snapshot().present_count;
    let receipt = if result.is_ok() && after > before {
        scope.receipt()
    } else {
        None
    };
    (result, receipt, awaiting)
}

type ToastPresentedCallback = Rc<dyn Fn(&str, u64) -> bool>;
#[derive(Default)]
pub(super) struct ToastPresentationObserver {
    callback: RefCell<Option<ToastPresentedCallback>>,
}
impl ToastPresentationObserver {
    fn register(&self, callback: impl Fn(&str, u64) -> bool + 'static) -> Result<(), &'static str> {
        let mut stored = self.callback.borrow_mut();
        if stored.is_some() {
            return Err("toast presentation observer is already registered");
        }
        *stored = Some(Rc::new(callback));
        Ok(())
    }
    pub(super) fn clear(&self) {
        self.callback.borrow_mut().take();
    }
    fn notify(
        &self,
        receipt: ToastPresentationReceipt,
        session: usize,
        current: HostPresentationGenerationCursor,
        visible: bool,
    ) -> bool {
        if !visible || receipt.session != session || receipt.cursor != current {
            return false;
        }
        let callback = self.callback.borrow().clone();
        callback.is_some_and(|callback| callback(&receipt.identity.id, receipt.identity.token))
    }
}
impl UiHostWindow {
    pub(crate) fn on_toast_presented(
        &self,
        callback: impl Fn(&str, u64) -> bool + 'static,
    ) -> Result<(), &'static str> {
        self.toast_presentation_observer.register(callback)
    }
    pub(in crate::ui::retained_host::host_contract) fn notify_toast_presented(
        &self,
        receipt: ToastPresentationReceipt,
    ) -> bool {
        self.toast_presentation_observer.notify(
            receipt,
            self.state.as_ptr() as usize,
            self.get_host_presentation_generation().cursor(),
            self.window().is_visible(),
        )
    }
}

#[cfg(test)]
#[path = "tests/toast_presentation.rs"]
mod tests;
