use std::collections::BTreeSet;

use super::{
    current_runtime_payload_schema_set_digest, ZrRuntimeArtifactIdentityV1, ZrRuntimeBuildSetId,
    ZrRuntimeDigestV1, ZrRuntimeIdentityEncodingError, ZrRuntimeInterfaceSpecV1,
    ZrRuntimeTargetModelV1,
};

/// Host-side lockstep requirements supplied by the product build, not by the loaded DLL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZrRuntimeBuildSetExpectationV1 {
    build_set_id: ZrRuntimeBuildSetId,
    interface_spec: ZrRuntimeInterfaceSpecV1,
    payload_schema_digest: ZrRuntimeDigestV1,
    target: ZrRuntimeTargetModelV1,
    required_capabilities: BTreeSet<String>,
    host_artifact: Option<ZrRuntimeArtifactIdentityV1>,
}

impl ZrRuntimeBuildSetExpectationV1 {
    /// Host 建立装载前预期：接口和 payload 摘要取自本地冻结源，BuildSet ID
    /// 应取自 Host 自身的构建身份，不能由待验证的 Runtime sidecar 回填。
    pub fn new(
        build_set_id: ZrRuntimeBuildSetId,
        target: ZrRuntimeTargetModelV1,
        required_capabilities: impl IntoIterator<Item = String>,
    ) -> Result<Self, ZrRuntimeIdentityEncodingError> {
        Ok(Self {
            build_set_id,
            interface_spec: ZrRuntimeInterfaceSpecV1::current()?,
            payload_schema_digest: current_runtime_payload_schema_set_digest(),
            target,
            required_capabilities: required_capabilities.into_iter().collect(),
            host_artifact: None,
        })
    }

    pub fn build_set_id(&self) -> &ZrRuntimeBuildSetId {
        &self.build_set_id
    }

    pub fn interface_spec(&self) -> &ZrRuntimeInterfaceSpecV1 {
        &self.interface_spec
    }

    pub fn payload_schema_digest(&self) -> &ZrRuntimeDigestV1 {
        &self.payload_schema_digest
    }

    pub fn target(&self) -> &ZrRuntimeTargetModelV1 {
        &self.target
    }

    pub fn required_capabilities(&self) -> &BTreeSet<String> {
        &self.required_capabilities
    }

    /// 附上当前 Host 可执行文件的实测身份，供装载前核对 sidecar 中的同名条目。
    pub fn with_host_artifact(mut self, host_artifact: ZrRuntimeArtifactIdentityV1) -> Self {
        self.host_artifact = Some(host_artifact);
        self
    }

    pub fn host_artifact(&self) -> Option<&ZrRuntimeArtifactIdentityV1> {
        self.host_artifact.as_ref()
    }
}
