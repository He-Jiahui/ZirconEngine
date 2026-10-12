import tempfile,unittest
from pathlib import Path
from unittest.mock import patch
from tools.jenkins.contracts import JenkinsError,digest
from tools.jenkins.patch_context import register_acceptance_checkout,resolve_patch_checkout,patch_checkout_for_registration
from tools.jenkins.source import repository_id
from tools.jenkins.state import State

class AcceptanceCheckoutTests(unittest.TestCase):
    def setUp(self):
        temp=tempfile.TemporaryDirectory(); self.addCleanup(temp.cleanup)
        self.root=Path(temp.name)
        self.fixture=self.root/".jenkins/tmp/acceptance/original"
        self.fixture.mkdir(parents=True)
        self.state=State(self.root/".jenkins/state/support.sqlite3")
        self.identity=dict(repositoryId=repository_id(self.fixture),sessionId="s",requestId="r",
            attemptId="a",generation=1,stageImplementationDigest="d"*64)
        self.state.authorize_session(self.identity["repositoryId"],"s","user",["src","Cargo.toml"],
            ["implementation","validation","deployment"],{"source":"user","task":"original Cargo fixture"})
        self.binding=dict(digest="d"*64,runtimeOperationId="start-one")
        mock=patch("tools.jenkins.patch_context.active_driver_binding",return_value=self.binding)
        mock.start(); self.addCleanup(mock.stop)
        self.payload=dict(identity=self.identity,checkoutRoot=str(self.fixture),ownedPaths=["src","Cargo.toml"])
        self.intent=dict(identity=self.identity,repositoryRoot=str(self.fixture),
            beforeHashes={"src/lib.rs":"a"*64},declaredDependencies=["Cargo.toml"])
    def test_bound_original_checkout_resolves_idempotently(self):
        result=register_acceptance_checkout(self.state,self.root,self.payload)
        self.assertEqual(result,register_acceptance_checkout(self.state,self.root,self.payload))
        self.assertEqual(self.fixture,resolve_patch_checkout(self.state,self.root,self.intent))
        self.assertIs(False,result["productRepositoryAcceptance"])
    def test_unregistered_checkout_is_rejected(self):
        with self.assertRaises(JenkinsError): resolve_patch_checkout(self.state,self.root,self.intent)
    def test_legacy_json_path_digest_cannot_register_as_a_source_repository(self):
        legacy = digest(str(self.fixture).casefold())
        self.assertNotEqual(repository_id(self.fixture), legacy)
        with self.assertRaises(JenkinsError) as raised:
            register_acceptance_checkout(self.state, self.root,
                {**self.payload, "identity": {**self.identity, "repositoryId": legacy}})
        self.assertEqual("acceptance_checkout_identity", raised.exception.code)
        self.assertEqual([], self.state.list("acceptance_checkout"))
    def test_resolver_rejects_legacy_identity_even_when_old_binding_exists(self):
        from tools.jenkins.patch_context import _key
        result = register_acceptance_checkout(self.state, self.root, self.payload)
        record = self.state.get("acceptance_checkout", result["checkoutBindingRef"])["payload"]
        legacy_identity = {**self.identity, "repositoryId": digest(str(self.fixture).casefold())}
        self.state.put("acceptance_checkout", _key(legacy_identity),
            {**record, "identity": legacy_identity}, expected_version=0)
        with self.assertRaises(JenkinsError) as raised:
            resolve_patch_checkout(self.state, self.root, {**self.intent, "identity": legacy_identity})
        self.assertEqual("acceptance_checkout_conflict", raised.exception.code)
    def test_arbitrary_path_cannot_be_registered(self):
        outside=self.root/"outside"; outside.mkdir()
        with self.assertRaises(JenkinsError):
            register_acceptance_checkout(self.state,self.root,{**self.payload,"checkoutRoot":str(outside)})
    def test_runtime_replacement_cannot_adopt_the_binding(self):
        register_acceptance_checkout(self.state,self.root,self.payload)
        self.binding["runtimeOperationId"]="different"
        with self.assertRaises(JenkinsError): resolve_patch_checkout(self.state,self.root,self.intent)
    def test_foreign_session_cannot_use_the_binding(self):
        register_acceptance_checkout(self.state,self.root,self.payload)
        with self.assertRaises(JenkinsError):
            resolve_patch_checkout(self.state,self.root,{**self.intent,"identity":{**self.identity,"sessionId":"foreign"}})
    def test_patch_scope_cannot_expand(self):
        register_acceptance_checkout(self.state,self.root,self.payload)
        with self.assertRaises(JenkinsError):
            resolve_patch_checkout(self.state,self.root,{**self.intent,"declaredDependencies":["secrets/token"]})

    def test_registration_resolves_only_the_matching_binding_reference(self):
        result = register_acceptance_checkout(self.state, self.root, self.payload)
        registration = {**self.intent, "checkoutBindingRef": result["checkoutBindingRef"]}
        self.assertEqual(self.fixture, patch_checkout_for_registration(self.state, self.root, registration))
        with self.assertRaises(JenkinsError):
            patch_checkout_for_registration(self.state, self.root, {**registration, "checkoutBindingRef": "foreign"})
    def test_registration_cannot_supply_an_unregistered_root(self):
        with self.assertRaises(JenkinsError):
            patch_checkout_for_registration(self.state, self.root, self.intent)
    def test_registration_preserves_the_default_checkout(self):
        self.assertEqual(self.root, patch_checkout_for_registration(self.state, self.root, {"identity": self.identity}))

    def test_control_intake_prepares_only_bound_original_checkout(self):
        import hashlib
        from tools.jenkins.patches import handle, prepare_patch
        from tools.jenkins.resources.paths import repository_build_root
        source = self.fixture/"src/lib.rs"
        source.parent.mkdir()
        source.write_text("fn value() -> u32 { 1 }\n", encoding="utf8")
        manifest = self.fixture/"Cargo.toml"
        manifest.write_text('[package]\nname="bound-fixture"\nversion="0.1.0"\n', encoding="utf8")
        binding = register_acceptance_checkout(self.state, self.root, self.payload)
        patch_text = ("diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n"
                      "@@ -1 +1 @@\n-fn value() -> u32 { 1 }\n+fn value() -> u32 { 2 }\n")
        request = dict(identity=self.identity, checkoutBindingRef=binding["checkoutBindingRef"],
                       unifiedPatch=patch_text, beforeHashes={"src/lib.rs":hashlib.sha256(source.read_bytes()).hexdigest()},
                       declaredDependencies=["Cargo.toml"], coverage={"template":"module_unit"},
                       buildRoot=str(repository_build_root()))
        with patch("tools.jenkins.workflow.handler._trusted_driver", return_value="d"*64):
            registered = handle("register", request, self.state, self.root)
        self.assertIn("{ 1 }", source.read_text())
        with patch("tools.jenkins.patches._ensure_patch_metadata") as metadata:
            result = prepare_patch(self.state, self.root,
                {"identity":self.identity,"patchRequestRef":registered["patchRequestRef"]},
                trusted_driver_digest="d"*64)
        self.assertEqual("prepared", result["status"])
        self.assertIn("{ 2 }", source.read_text())
        self.assertEqual(self.fixture, metadata.call_args.args[1])
        self.assertEqual(self.root, metadata.call_args.kwargs["policy_root"])
        sealed = self.state.get("sealed_input", result["sealedInputRef"])["payload"]
        self.assertEqual(repository_id(self.fixture), self.identity["repositoryId"])
        self.assertEqual(self.identity["repositoryId"], sealed["repositoryId"])
        self.assertEqual(self.identity["repositoryId"], result["identity"]["repositoryId"])
        self.assertEqual(self.fixture, resolve_patch_checkout(self.state, self.root, self.intent))
        self.assertEqual(self.fixture, patch_checkout_for_registration(self.state, self.root,
            {**self.intent, "checkoutBindingRef": binding["checkoutBindingRef"]}))
        self.assertFalse((self.root/"src/lib.rs").exists())
    def test_control_registration_rejects_a_root_without_binding_before_intake(self):
        from tools.jenkins.patches import handle
        with patch("tools.jenkins.workflow.handler._trusted_driver", return_value="d"*64):
            with self.assertRaises(JenkinsError):
                handle("register", self.intent, self.state, self.root)
        self.assertEqual([], self.state.list("patch_intent"))

    def test_applied_patch_intake_keeps_controller_context_and_derives_runtime(self):
        import hashlib
        import subprocess
        from tools.jenkins.patches import handle
        from tools.jenkins.resources.paths import repository_build_root
        before = b"pub fn value() -> u32 { 1 }\n"
        after = b"pub fn value() -> u32 { 2 }\n"
        manifest = b'[package]\nname="bound-applied"\nversion="0.1.0"\nedition="2021"\n'
        source = self.fixture / "src/lib.rs"
        source.parent.mkdir()
        source.write_bytes(before)
        (self.fixture / "Cargo.toml").write_bytes(manifest)
        def git(*args):
            return subprocess.run(["git", "-C", str(self.fixture), "-c", "user.name=Fixture",
                                   "-c", "user.email=fixture@example.invalid", *args],
                                   capture_output=True, check=True).stdout.decode().strip()
        git("init", "-q")
        git("config", "core.autocrlf", "false")
        git("add", "src/lib.rs", "Cargo.toml")
        git("commit", "-qm", "original applied-patch fixture")
        head = git("rev-parse", "HEAD")
        source.write_bytes(after)
        checkout = register_acceptance_checkout(self.state, self.root, self.payload)
        marker = self.root / ".jenkins/state/deployment/driver.json"
        marker.parent.mkdir(parents=True)
        marker.write_text("{}", encoding="utf-8")
        request = {"identity": self.identity, "checkoutBindingRef": checkout["checkoutBindingRef"],
                   "inputMode": "already-applied", "baseHead": head,
                   "unifiedPatch": "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-pub fn value() -> u32 { 1 }\n+pub fn value() -> u32 { 2 }\n",
                   "beforeHashes": {"src/lib.rs": hashlib.sha256(before).hexdigest()},
                   "afterHashes": {"src/lib.rs": hashlib.sha256(after).hexdigest()},
                   "declaredDependencies": ["Cargo.toml"],
                   "dependencyHashes": {"Cargo.toml": hashlib.sha256(manifest).hexdigest()},
                   "coverage": {"template": "module_unit"}, "buildRoot": str(repository_build_root())}
        with patch("tools.jenkins.workflow.handler._trusted_driver", return_value="d" * 64), \
             patch("tools.jenkins.deployment.driver.active_driver_binding", return_value=self.binding):
            registered = handle("register", request, self.state, self.root)
            intent = self.state.get("patch_intent", registered["patchRequestRef"])["payload"]
            self.assertEqual(str(self.root), intent["contextRoot"])
            self.assertEqual(str(self.fixture), intent["repositoryRoot"])
            self.assertEqual("start-one", intent["runtimeOperationId"])
            with self.assertRaises(JenkinsError) as error:
                handle("register", {**request, "runtimeOperationId": "foreign-start"}, self.state, self.root)
            self.assertEqual("runtime_binding_mismatch", error.exception.code)
        self.assertEqual(after, source.read_bytes())
        self.assertEqual([], self.state.list("sealed_input"))
