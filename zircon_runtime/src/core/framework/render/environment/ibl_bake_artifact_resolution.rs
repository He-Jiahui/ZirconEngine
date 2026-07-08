use super::{
    select_ibl_bake_artifact, IblBakeArtifactBlob, IblBakeArtifactCandidate,
    IblBakeArtifactDescriptor, IblBakeArtifactPayload, IblBakeArtifactRequest,
    IblBakeArtifactSelection, IblBakeArtifactSource,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactBlobCandidate {
    source: IblBakeArtifactSource,
    blob: IblBakeArtifactBlob,
}

impl IblBakeArtifactBlobCandidate {
    pub fn asset_derived(blob: IblBakeArtifactBlob) -> Self {
        Self {
            source: IblBakeArtifactSource::AssetDerivedArtifact,
            blob,
        }
    }

    pub fn runtime_cache(blob: IblBakeArtifactBlob) -> Self {
        Self {
            source: IblBakeArtifactSource::RuntimeCache,
            blob,
        }
    }

    pub const fn source(&self) -> IblBakeArtifactSource {
        self.source
    }

    pub fn blob(&self) -> &IblBakeArtifactBlob {
        &self.blob
    }

    pub const fn descriptor(&self) -> IblBakeArtifactDescriptor {
        self.blob.descriptor()
    }

    pub fn payload(&self) -> &IblBakeArtifactPayload {
        self.blob.payload()
    }

    pub fn descriptor_candidate(&self) -> IblBakeArtifactCandidate {
        match self.source {
            IblBakeArtifactSource::AssetDerivedArtifact => {
                IblBakeArtifactCandidate::asset_derived(self.descriptor())
            }
            IblBakeArtifactSource::RuntimeCache => {
                IblBakeArtifactCandidate::runtime_cache(self.descriptor())
            }
            IblBakeArtifactSource::RuntimeCompute => {
                unreachable!("runtime compute is not a blob-backed artifact candidate")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactResolvedPayload {
    selection: IblBakeArtifactSelection,
    blob: Option<IblBakeArtifactBlob>,
}

impl IblBakeArtifactResolvedPayload {
    pub const fn selection(&self) -> IblBakeArtifactSelection {
        self.selection
    }

    pub const fn source(&self) -> IblBakeArtifactSource {
        self.selection.source()
    }

    pub const fn descriptor(&self) -> Option<IblBakeArtifactDescriptor> {
        self.selection.descriptor()
    }

    pub fn blob(&self) -> Option<&IblBakeArtifactBlob> {
        self.blob.as_ref()
    }

    pub fn payload(&self) -> Option<&IblBakeArtifactPayload> {
        self.blob().map(IblBakeArtifactBlob::payload)
    }

    pub const fn rejected_candidate_count(&self) -> usize {
        self.selection.rejected_candidate_count()
    }

    pub const fn environment_compute_dispatch_count(&self) -> u32 {
        self.selection.environment_compute_dispatch_count()
    }

    pub const fn requires_runtime_compute(&self) -> bool {
        self.selection.requires_runtime_compute()
    }
}

pub fn resolve_ibl_bake_artifact_payload(
    request: &IblBakeArtifactRequest,
    candidates: &[IblBakeArtifactBlobCandidate],
) -> IblBakeArtifactResolvedPayload {
    let descriptor_candidates = candidates
        .iter()
        .map(IblBakeArtifactBlobCandidate::descriptor_candidate)
        .collect::<Vec<_>>();
    let selection = select_ibl_bake_artifact(request, &descriptor_candidates);
    let blob = selection.descriptor().and_then(|descriptor| {
        candidates
            .iter()
            .find(|candidate| {
                candidate.source() == selection.source()
                    && candidate.descriptor() == descriptor
                    && descriptor.is_current_for(request)
            })
            .map(|candidate| candidate.blob().clone())
    });

    IblBakeArtifactResolvedPayload { selection, blob }
}
