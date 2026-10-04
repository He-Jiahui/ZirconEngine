mod application_handler;
mod application_lifecycle;
mod config;
mod construct;
mod converters;
mod device_events;
mod event_dispatch;
mod event_loop_policy;
mod failure;
mod file_drag_drop;
mod frame_capture;
mod frame_loop;
#[cfg(feature = "gamepad-gilrs")]
mod gamepad;
mod host_requests;
mod ime_input;
mod keyboard_input;
mod mvp_input_probe;
mod pointer_input;
mod runtime_product_diagnostics;
mod surface_present;
mod window_attributes;
mod window_creation;
mod window_events;
mod window_lifecycle;
mod window_surface;

use std::{num::NonZeroU64, sync::Arc};

use winit::dpi::PhysicalPosition;
use winit::window::Window;
use zircon_runtime::asset::project::ResolvedProjectPath;
use zircon_runtime::core::framework::window::{WindowDescriptor, WindowLifecyclePolicy};
use zircon_runtime_interface::{
    ZrRuntimeImeCandidateRectV2, ZrRuntimeImeCursorAreaV1, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1,
};

use super::runtime_library::RuntimeSession;
use crate::reference_cpu_presenter::ReferenceCpuPresenter;
use application_lifecycle::ApplicationLifecycleMachine;
use event_loop_policy::RuntimeFrameCadence;
use ime_input::{RuntimeImeCompositionProducer, RuntimeImeInputAdmission};

pub(in crate::entry) use config::RuntimeEntryAppConfig;
pub(in crate::entry) use failure::RuntimeEntryAppFailureState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeImeHostPublicationReceipt {
    pub(super) window_generation: u64,
    pub(super) candidate_rect: ZrRuntimeImeCandidateRectV2,
}

pub(super) struct RuntimeEntryApp {
    window: Option<Arc<dyn Window>>,
    window_descriptor: WindowDescriptor,
    frame_cadence: RuntimeFrameCadence,
    ime_input_admission: RuntimeImeInputAdmission,
    application_lifecycle: ApplicationLifecycleMachine,
    window_lifecycle_policy: WindowLifecyclePolicy,
    presenter: Option<ReferenceCpuPresenter>,
    reference_cpu_presenter_enabled: bool,
    native_ime_composition_requested: bool,
    /// The most recent checked caret topology published for the live primary window.
    active_candidate_rect: Option<ZrRuntimeImeCandidateRectV2>,
    /// A successful Winit request is a host-thread ownership handoff. It is deliberately kept
    /// separate from `active_candidate_rect`: Winit exposes no native installation ACK, and the
    /// IMM producer is admitted only after this receipt and AppSession configuration succeed.
    native_ime_host_publication: Option<NativeImeHostPublicationReceipt>,
    ime_composition_producer: Option<RuntimeImeCompositionProducer>,
    native_ime_composition_started: bool,
    ime_window_generation: u64,
    surface_present_enabled: bool,
    surface_present_attempted: bool,
    exit_after_presented_frames: Option<NonZeroU64>,
    presented_frame_count: u64,
    first_frame_capture_path: Option<ResolvedProjectPath>,
    require_persisted_scene_diagnostics: bool,
    first_frame_capture_written: bool,
    first_frame_product_diagnostics_emitted: bool,
    mvp_input_probe_submitted: bool,
    unhandled_ui_action_count: u64,
    unhandled_ui_host_request_count: u64,
    // Event callbacks set an atomic stop gate and append failures to the shared cold-path ledger.
    failure_state: RuntimeEntryAppFailureState,
    session: RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    viewport_size: ZrRuntimeViewportSizeV1,
    last_pointer_position: Option<PhysicalPosition<f64>>,
    #[cfg(feature = "gamepad-gilrs")]
    gamepads: Option<gilrs::Gilrs>,
    #[cfg(feature = "gamepad-gilrs")]
    gamepad_connections_announced: bool,
    #[cfg(feature = "gamepad-gilrs")]
    gamepad_rumble_effects: Option<gamepad::RunningRumbleEffects>,
}

impl RuntimeEntryApp {
    /// Rejects an explicitly requested V2 path before mutating the runtime session or installing
    /// a producer. The Windows Winit backend owns the normal `WM_IME_*` callback and the app's
    /// IMM reader consumes its attributes; other platforms remain V1-only.
    pub(super) fn validate_native_ime_v2_request(&self) -> Result<(), &'static str> {
        if !self.native_ime_composition_requested {
            return Ok(());
        }
        if !self.session.supports_app_session_configuration() {
            return Err("requested native IME V2 requires the AppSession V2 entry point");
        }
        if !ime_input::native_ime_provider_available() {
            return Err("requested native IME V2 has no supported native callback adapter");
        }
        Ok(())
    }

    /// Converts the real UI caret request into the public candidate topology and installs the IMM
    /// producer only after the checked AppSession V2 configuration is accepted. The default V1
    /// route never calls this method.
    pub(super) fn prepare_native_ime_candidate(
        &mut self,
        area: ZrRuntimeImeCursorAreaV1,
    ) -> Result<ZrRuntimeImeCandidateRectV2, &'static str> {
        let candidate = match self.validate_native_ime_candidate(area) {
            Ok(candidate) => candidate,
            Err(error) => {
                self.reject_native_ime_host_publication();
                return Err(error);
            }
        };
        let receipt = NativeImeHostPublicationReceipt {
            window_generation: self.ime_window_generation,
            candidate_rect: candidate,
        };
        if self.native_ime_host_publication != Some(receipt) {
            self.reject_native_ime_host_publication();
            return Err("native IME host request was not accepted by the live window");
        }
        let configuration =
            zircon_runtime_interface::ZrRuntimeAppSessionConfigurationV2::ime_composition(
                self.ime_window_generation,
                candidate,
            );
        let negotiation = match configuration.negotiation() {
            Ok(negotiation) => negotiation,
            Err(_) => {
                self.reject_native_ime_host_publication();
                return Err("runtime AppSession V2 rejected the checked caret descriptor");
            }
        };
        if self.session.configure_app_session(configuration).is_err() {
            // The Winit request has already been handed to the host, but no producer may be
            // admitted after a rejected runtime configuration. Fence a queued callback before
            // the caller can retry the same window binding.
            self.reject_native_ime_host_publication();
            return Err("runtime AppSession V2 rejected the checked caret descriptor");
        }
        let producer_generation = self
            .ime_composition_producer
            .as_ref()
            .map(|producer| producer.context().window_generation);
        if producer_generation != Some(self.ime_window_generation) {
            if self
                .set_native_ime_composition_context(
                    negotiation,
                    self.ime_window_generation,
                    candidate,
                )
                .is_err()
            {
                self.reject_native_ime_host_publication();
                return Err("native IME callback adapter could not retain the window generation");
            }
            self.native_ime_composition_started = false;
        } else if let Some(producer) = self.ime_composition_producer.as_mut() {
            producer.update_candidate_rect(candidate);
        }
        self.active_candidate_rect = Some(candidate);
        Ok(candidate)
    }

    pub(super) fn validate_native_ime_candidate(
        &self,
        area: ZrRuntimeImeCursorAreaV1,
    ) -> Result<ZrRuntimeImeCandidateRectV2, &'static str> {
        if !self.native_ime_composition_requested {
            return Err("native candidate publication was requested without native V2 opt-in");
        }
        if !self.ime_input_admission.window_focused() {
            return Err("native IME candidate publication requires the primary window focus");
        }
        if self.ime_window_generation == 0 {
            return Err("native IME window generation is exhausted");
        }
        let candidate =
            crate::entry::runtime_entry_app::host_requests::ime::geometry::ime_candidate_rect(area)
                .ok_or(
                    "runtime IME cursor area is not representable by the native candidate ABI",
                )?;
        Ok(candidate)
    }

    /// Records that the current-thread Winit request was accepted by the live host. This is an
    /// ownership receipt, not an installation ACK: the platform callback is admitted only after
    /// `prepare_native_ime_candidate` completes the runtime configuration and producer install.
    pub(super) fn record_native_ime_host_publication(
        &mut self,
        candidate_rect: ZrRuntimeImeCandidateRectV2,
    ) {
        self.native_ime_host_publication = Some(NativeImeHostPublicationReceipt {
            window_generation: self.ime_window_generation,
            candidate_rect,
        });
    }

    /// Fences every producer and host receipt after a failed request or rejected configuration.
    /// Advancing the generation prevents an already queued Winit callback from entering a retry.
    pub(super) fn reject_native_ime_host_publication(&mut self) {
        self.clear_native_ime_candidate();
        self.advance_native_ime_generation();
    }

    pub(super) fn advance_native_ime_generation(&mut self) {
        self.ime_window_generation = self.ime_window_generation.checked_add(1).unwrap_or(0);
    }

    pub(super) fn clear_native_ime_candidate(&mut self) {
        self.active_candidate_rect = None;
        self.native_ime_host_publication = None;
        self.ime_composition_producer = None;
        self.native_ime_composition_started = false;
    }
}
