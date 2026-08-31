use zircon_runtime_interface::runtime_build_set::{
    ZrRuntimeDigestV1, ZrRuntimeModuleCompositionReceiptV1, ZrRuntimeModuleCompositionTargetV1,
    ZrRuntimeModuleProfileV1, ZrRuntimeSessionProfileV1,
};
use zircon_runtime_interface::ProfileControlResponse;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::RuntimeProfileId;

use super::{RuntimeDynamicSession, RuntimeDynamicSessionProfile};

pub(super) fn module_composition_receipt_response(
    session: &RuntimeDynamicSession,
) -> ProfileControlResponse {
    let identity = &session.module_composition_identity;
    let Some(catalog_generation) = identity.catalog_generation() else {
        return ProfileControlResponse::error(
            "runtime module composition identity is missing catalog provenance",
        );
    };
    let Some(source_manifest_fingerprint) = identity.source_manifest_fingerprint() else {
        return ProfileControlResponse::error(
            "runtime module composition identity is missing manifest provenance",
        );
    };
    let composition_hash = match ZrRuntimeDigestV1::parse(identity.composition_hash_hex()) {
        Ok(composition_hash) => composition_hash,
        Err(error) => {
            return ProfileControlResponse::error(format!(
                "runtime module composition identity produced an invalid digest: {error}"
            ));
        }
    };

    let receipt = ZrRuntimeModuleCompositionReceiptV1::new(
        catalog_generation.get(),
        source_manifest_fingerprint,
        target_mode(identity.target_mode()),
        identity.runtime_profile().map(module_profile),
        session_profile(session.profile),
        composition_hash,
    );
    let mut response = ProfileControlResponse::ok("runtime module composition receipt captured");
    response.module_composition_receipt = Some(receipt);
    response
}

fn target_mode(target: RuntimeTargetMode) -> ZrRuntimeModuleCompositionTargetV1 {
    match target {
        RuntimeTargetMode::ClientRuntime => ZrRuntimeModuleCompositionTargetV1::ClientRuntime,
        RuntimeTargetMode::ServerRuntime => ZrRuntimeModuleCompositionTargetV1::ServerRuntime,
        RuntimeTargetMode::EditorHost => ZrRuntimeModuleCompositionTargetV1::EditorHost,
    }
}

fn module_profile(profile: RuntimeProfileId) -> ZrRuntimeModuleProfileV1 {
    match profile {
        RuntimeProfileId::Minimal => ZrRuntimeModuleProfileV1::Minimal,
        RuntimeProfileId::Client2d => ZrRuntimeModuleProfileV1::Client2d,
        RuntimeProfileId::Client3d => ZrRuntimeModuleProfileV1::Client3d,
        RuntimeProfileId::Editor => ZrRuntimeModuleProfileV1::Editor,
        RuntimeProfileId::Dev => ZrRuntimeModuleProfileV1::Dev,
        RuntimeProfileId::Server => ZrRuntimeModuleProfileV1::Server,
    }
}

fn session_profile(profile: RuntimeDynamicSessionProfile) -> ZrRuntimeSessionProfileV1 {
    match profile {
        RuntimeDynamicSessionProfile::Runtime => ZrRuntimeSessionProfileV1::Runtime,
        RuntimeDynamicSessionProfile::RuntimePipelined => {
            ZrRuntimeSessionProfileV1::RuntimePipelined
        }
        RuntimeDynamicSessionProfile::Editor => ZrRuntimeSessionProfileV1::Editor,
        RuntimeDynamicSessionProfile::Dev => ZrRuntimeSessionProfileV1::Dev,
        RuntimeDynamicSessionProfile::Minimal => ZrRuntimeSessionProfileV1::Minimal,
        RuntimeDynamicSessionProfile::Headless => ZrRuntimeSessionProfileV1::Headless,
    }
}
