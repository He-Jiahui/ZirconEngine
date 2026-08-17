use zircon_runtime_host::foreign_output::{
    profile_control_response_item_count, RuntimeForeignOutputKind, PROFILE_RESPONSE_OUTPUT_BUDGET,
};
use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrByteSlice, ZrOwnedResultV2,
};

use super::super::GatewayError;
use super::gateway::SessionGateway;

impl SessionGateway {
    pub(super) fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, GatewayError> {
        self.ensure_output_available(RuntimeForeignOutputKind::ProfileResponse)?;
        let Some(profile) = self.api.profile_control else {
            return Ok(None);
        };
        let request = serde_json::to_vec(request).map_err(|error| GatewayError::Protocol {
            message: format!("encode runtime profile request: {error}"),
        })?;
        let mut output = ZrOwnedResultV2::empty();
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
        self.decode_output(
            status,
            output,
            RuntimeForeignOutputKind::ProfileResponse,
            PROFILE_RESPONSE_OUTPUT_BUDGET,
            "control runtime profiling",
            "free runtime profile response",
            |response: &ProfileControlResponse| {
                Ok::<usize, GatewayError>(profile_control_response_item_count(response))
            },
        )
    }
}
