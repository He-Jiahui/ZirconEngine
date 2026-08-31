use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::core::gateway::{EditorRuntimeFrame, GatewayError, GatewaySessionIdentity};

use super::{PlayInstanceId, PlayModeKind};

#[derive(Clone, Debug)]
pub struct PlayPreviewFrame {
    identity: PlayPreviewFrameIdentity,
    width: u32,
    height: u32,
    generation: u64,
    rgba: Arc<[u8]>,
}

/// Complete provenance for the runtime product currently displayed by a Play viewport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayPreviewFrameIdentity {
    instance: PlayInstanceId,
    gateway: GatewaySessionIdentity,
    width: u32,
    height: u32,
    generation: u64,
}

impl PlayPreviewFrameIdentity {
    pub const fn instance(&self) -> PlayInstanceId {
        self.instance
    }

    pub fn gateway(&self) -> &GatewaySessionIdentity {
        &self.gateway
    }

    pub const fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn resource_scope(&self, prefix: &str) -> String {
        let gateway = self.gateway();
        let project = gateway
            .project()
            .map(|project| format!("some:{project}"))
            .unwrap_or_else(|| "none".to_string());
        let play_instance = gateway
            .play_instance()
            .map(|instance| format!("some:{instance}"))
            .unwrap_or_else(|| "none".to_string());
        format!(
            "{prefix}:{}:{}:{}:{}:{}:{play_instance}:{}:{}:{}:{project}",
            self.instance().raw(),
            gateway.runtime_instance(),
            gateway.runtime_session().raw(),
            gateway.transport_epoch(),
            gateway.gateway_generation(),
            self.width,
            self.height,
            self.generation
        )
    }
}

impl PlayPreviewFrame {
    pub(crate) fn copy_and_release(
        instance: PlayInstanceId,
        gateway: GatewaySessionIdentity,
        frame: EditorRuntimeFrame,
    ) -> Result<Self, PlayPreviewCaptureError> {
        let width = frame.width();
        let height = frame.height();
        let generation = frame.generation();
        let rgba = Arc::<[u8]>::from(frame.rgba());
        zircon_runtime::profile_counter!("editor", "play.preview.copy_bytes", rgba.len());
        frame.release().map_err(PlayPreviewCaptureError::Release)?;
        Ok(Self {
            identity: PlayPreviewFrameIdentity {
                instance,
                gateway,
                width,
                height,
                generation,
            },
            width,
            height,
            generation,
            rgba,
        })
    }

    pub const fn instance(&self) -> PlayInstanceId {
        self.identity.instance
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn rgba(&self) -> &Arc<[u8]> {
        &self.rgba
    }

    pub fn identity(&self) -> &PlayPreviewFrameIdentity {
        &self.identity
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        instance: PlayInstanceId,
        gateway: GatewaySessionIdentity,
        width: u32,
        height: u32,
        generation: u64,
        rgba: Vec<u8>,
    ) -> Self {
        Self {
            identity: PlayPreviewFrameIdentity {
                instance,
                gateway,
                width,
                height,
                generation,
            },
            width,
            height,
            generation,
            rgba: rgba.into(),
        }
    }
}

#[derive(Debug)]
pub enum PlayPreviewCaptureError {
    GatewayUnavailable { mode: PlayModeKind },
    Capture(GatewayError),
    Release(GatewayError),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime_interface::{
        GatewaySessionIdentity, ZrRuntimeSessionHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
    };

    use super::*;

    #[test]
    fn preview_frame_identity_retains_the_complete_gateway_origin() {
        let gateway = GatewaySessionIdentity::new(
            7,
            ZrRuntimeSessionHandle::new(11),
            13,
            Some(Arc::from("res://project")),
        )
        .with_gateway_generation(17)
        .with_play_instance(Some(19));
        let frame =
            EditorRuntimeFrame::new(ZIRCON_RUNTIME_ABI_VERSION_V1, 1, 1, 23, vec![0, 0, 0, 255]);

        let preview = PlayPreviewFrame::copy_and_release(
            PlayInstanceId::for_test(19),
            gateway.clone(),
            frame,
        )
        .expect("preview copy");

        assert_eq!(preview.identity().gateway(), &gateway);
        assert_eq!(preview.identity().instance(), PlayInstanceId::for_test(19));
        assert_eq!(preview.identity().generation(), 23);
        assert_eq!(preview.identity().size(), (1, 1));
        assert_eq!(
            preview.identity().resource_scope("preview-test"),
            "preview-test:19:7:11:13:17:some:19:1:1:23:some:res://project"
        );
    }
}

impl Display for PlayPreviewCaptureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GatewayUnavailable { mode } => write!(
                formatter,
                "play preview gateway is unavailable while mode is {mode:?}"
            ),
            Self::Capture(source) => write!(formatter, "failed to capture play preview: {source}"),
            Self::Release(source) => {
                write!(formatter, "failed to release play preview output: {source}")
            }
        }
    }
}

impl Error for PlayPreviewCaptureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GatewayUnavailable { .. } => None,
            Self::Capture(source) | Self::Release(source) => Some(source),
        }
    }
}
