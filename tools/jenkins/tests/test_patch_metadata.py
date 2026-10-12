import tempfile,unittest
from pathlib import Path
from unittest.mock import patch
from tools.jenkins.contracts import JenkinsError,digest
from tools.jenkins.state import State
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.resources.paths import repository_build_root
from tools.jenkins.patches import _ensure_patch_metadata
from tools.jenkins.source import SealedManifest

class PatchMetadataTests(unittest.TestCase):
    def setUp(self):
        temp=tempfile.TemporaryDirectory(); self.addCleanup(temp.cleanup)
        self.context=Path(temp.name); self.source=self.context/"source"; self.source.mkdir()
        self.state=State(self.context/"support.sqlite3")
        self.build=str(repository_build_root()); self.objects=str(canonical_build_root(self.build).namespace())
        self.identity=dict(repositoryId="repo",sessionId="s",requestId="r",attemptId="a",generation=1,
                           stageImplementationDigest="d"*64)
        self.intent=dict(identity=self.identity,buildRoot=self.build)
        self.source_digest="a"*64
        self.sealed=dict(status="sealed",sourceDigest=self.source_digest,repositoryId="repo",repositoryRoot=str(self.source),
                         buildRoot=self.build,objectRoot=self.objects)
        self.state.put("sealed_input",self.source_digest,self.sealed)
        self.policy={"kind":"verified-test-policy"}; self.state.put("environment_policy","default",self.policy)
        self.evidence=dict(sourceDigest=self.source_digest,manifestDigest=self.source_digest,
            repositoryId="repo",buildRoot=self.build,environmentPolicyDigest=digest(self.policy),
            toolchainDigest="b"*64,lockDigest="c"*64,
            cargoMetadata={"packages":[],"workspace_members":[]},
            workspaceRoot=str(Path(self.objects)/"inputs/runs"/self.source_digest))
        self.manifest={"mock":"sealed"}
        self.mock_manifest=patch("tools.jenkins.patches.get_sealed_manifest",return_value=self.manifest).start()
        self.mock_policy=patch("tools.jenkins.patches._verified_metadata_inputs",
            return_value=({"PATH":"trusted"},self.policy,"cargo.exe","rustc.exe")).start()
        self.collector=patch("tools.jenkins.patches.collect_cargo_evidence",return_value=self.evidence).start()
        def persist(state,evidence,**kwargs):
            state.put("sealed_cargo_evidence",self.source_digest,{**evidence,"repositoryId":kwargs["repository_id_value"]})
        self.persist=patch("tools.jenkins.patches.persist_cargo_evidence",side_effect=persist).start()
        self.addCleanup(patch.stopall)
    def ensure(self, intent=None, sealed=None):
        _ensure_patch_metadata(self.state,self.source,intent or self.intent,sealed or self.sealed,
                               policy_root=self.context)
    def test_first_collection_then_sealed_evidence_reuse(self):
        self.ensure(); self.ensure()
        self.assertEqual(1,self.collector.call_count); self.assertEqual(1,self.persist.call_count)
        self.mock_policy.assert_called_with(self.state,self.context)
        self.assertEqual(digest(self.policy),self.collector.call_args.kwargs["environment_policy_digest"])
    def test_first_collection_accepts_the_authoritys_real_sealed_manifest(self):
        manifest=SealedManifest({"entries":[],"external":[]},self.source_digest)
        self.mock_manifest.return_value=manifest
        self.ensure()
        self.assertIs(manifest,self.collector.call_args.args[0])
        self.assertEqual(1,self.persist.call_count)
    def test_real_external_manifest_requires_the_sibling_bundle_workspace(self):
        dependency={"entries":[]}
        manifest=SealedManifest({"entries":[],"preparationKind":"current-source","external":[{
            "kind":"sealed-external-source","name":"zr_vm","manifest":dependency,
            "sourceDigest":digest(dependency)}]},self.source_digest)
        self.mock_manifest.return_value=manifest
        with self.assertRaises(JenkinsError) as caught:self.ensure()
        self.assertEqual("cargo_workspace_identity_mismatch",caught.exception.code)
        self.assertEqual(0,self.persist.call_count)
        self.evidence["workspaceRoot"]=str(Path(self.evidence["workspaceRoot"])/"source")
        self.ensure()
        self.assertEqual(1,self.persist.call_count)
    def test_real_sealed_manifest_does_not_allow_another_workspace(self):
        self.mock_manifest.return_value=SealedManifest({"entries":[]},self.source_digest)
        self.evidence["workspaceRoot"]=str(Path(self.objects)/"inputs/runs"/("f"*64))
        with self.assertRaises(JenkinsError) as caught:self.ensure()
        self.assertEqual("cargo_workspace_identity_mismatch",caught.exception.code)
        self.assertEqual(0,self.persist.call_count)
    def test_shared_sealed_input_is_bound_to_new_consumer_without_recollection(self):
        self.ensure()
        identity={**self.identity,"sessionId":"other","attemptId":"other-owner"}
        self.ensure({**self.intent,"identity":identity})
        self.assertEqual(1,self.collector.call_count)
        self.assertEqual("other",self.mock_manifest.call_args.kwargs["session_id"])
        self.assertEqual("other-owner",self.mock_manifest.call_args.kwargs["owner"])
    def test_wrong_environment_evidence_is_rejected(self):
        self.state.put("sealed_cargo_evidence",self.source_digest,
                       {**self.evidence,"environmentPolicyDigest":"f"*64})
        with self.assertRaises(JenkinsError): self.ensure()
        self.assertEqual(0,self.collector.call_count)
    def test_sealed_object_root_cannot_change(self):
        self.state.put("sealed_input",self.source_digest,{**self.sealed,"objectRoot":str(self.source)})
        with self.assertRaises(JenkinsError): self.ensure()
        self.assertEqual(0,self.collector.call_count)
    def test_sealed_checkout_root_must_match_selected_source(self):
        other=self.context / "other-source"
        other.mkdir()
        self.state.put("sealed_input",self.source_digest,{**self.sealed,"repositoryRoot":str(other)})
        with self.assertRaises(JenkinsError) as caught: self.ensure()
        self.assertEqual("sealed_repository_mismatch",caught.exception.code)
        self.assertEqual(0,self.collector.call_count)

    def test_sealed_checkout_rejects_a_physical_alias(self):
        alias=self.context / "SOURCE"
        try:
            alias.symlink_to(self.source,target_is_directory=True)
        except (OSError,NotImplementedError):
            self.skipTest("directory aliases are unavailable in this environment")
        self.state.put("sealed_input",self.source_digest,{**self.sealed,"repositoryRoot":str(alias)})
        with self.assertRaises(JenkinsError) as caught: self.ensure()
        self.assertEqual("sealed_repository_mismatch",caught.exception.code)
        self.assertEqual(0,self.collector.call_count)

    def test_collector_result_must_retain_the_source_identity(self):
        self.collector.return_value={**self.evidence,"sourceDigest":"f"*64}
        with self.assertRaises(JenkinsError): self.ensure()
        self.assertEqual(0,self.persist.call_count)
    def test_formal_marker_cannot_bypass_missing_policy(self):
        marker=self.context/".jenkins/state/deployment/driver.json"; marker.parent.mkdir(parents=True)
        marker.write_text("{}",encoding="utf8")
        self.state.delete("environment_policy","default")
        self.mock_policy.side_effect=JenkinsError("environment_policy_missing","missing")
        with self.assertRaises(JenkinsError): self.ensure()
        self.assertEqual(0,self.collector.call_count)
    def test_existing_metadata_requires_an_approved_workspace_path(self):
        self.state.put("sealed_cargo_evidence",self.source_digest,
                       {**self.evidence,"workspaceRoot":str(self.source)})
        with self.assertRaises(JenkinsError): self.ensure()
        self.assertEqual(0,self.collector.call_count)
