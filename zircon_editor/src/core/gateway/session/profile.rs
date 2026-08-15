use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrByteSlice, ZrOwnedByteBuffer,
};

use super::super::GatewayError;
use super::gateway::SessionGateway;
use super::output::{decode_owned_output, validate_output_status};

impl SessionGateway {
    pub(super) fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, GatewayError> {
        let Some(profile) = self.api.profile_control else {
            return Ok(None);
        };
        let request = serde_json::to_vec(request).map_err(|error| GatewayError::Protocol {
            message: format!("encode runtime profile request: {error}"),
        })?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe {
            profile(
                self.session,
                ZrByteSlice {
                    data: request.as_ptr(),
                    len: request.len(),
                },
                &mut output,
            )
        };
        let output = validate_output_status(status, output, "control runtime profiling")?;
        decode_owned_output(output, "control runtime profiling").map(Some)
    }
}
