"""Fixed owned agents migrate only after exact archival and fresh idle proof."""
from contextlib import ExitStack
from copy import deepcopy
import base64
import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.contracts import JenkinsError, canonical_json
from tools.jenkins.deployment import bootstrap, job_provisioning

ROOT = Path(__file__).absolute().parents[3]
BASE = "zircon-windows-agent"
OLD = BASE + "-0123456789abcdef"


def node_xml(name, remote, *, legacy=False, executors=1):
    suffix = name[len(BASE):]
    return ("<?xml version='1.1' encoding='UTF-8'?><slave><name>" + name + "</name>"
        + ("" if legacy else "<description>Zircon Jenkins runtime agent</description>")
        + "<remoteFS>" + str(remote) + "</remoteFS><numExecutors>" + str(executors) + "</numExecutors>"
        + "<mode>" + ("NORMAL" if legacy and name == BASE else "EXCLUSIVE") + "</mode>"
        + "<retentionStrategy class='hudson.slaves.RetentionStrategy$Always'/>"
        + "<launcher class='hudson.slaves.JNLPLauncher'><workDirSettings><disabled>true</disabled>"
        + "<internalDir>remoting</internalDir><failIfWorkDirIsMissing>false</failIfWorkDirIsMissing>"
        + "</workDirSettings><webSocket>true</webSocket></launcher><label>"
        + ("zircon-windows" + suffix if legacy else name) + "</label><nodeProperties/></slave>").encode()


class Nodes:
    def __init__(self, root):
        self.paths = SimpleNamespace(repo=root, state=root / ".jenkins/state", home=root / ".jenkins/jenkins_home",
                                     agent=root / ".jenkins/agent", build_root=Path("E:/cargo-targets/jenkins"))
        self.spec = SimpleNamespace(agent={"name": BASE, "label": "zircon-windows", "executors": 2},
                                    controller={"httpPort": 18080, "listenAddress": "127.0.0.1"})
        self.manager = Mock()
        self.actor = bootstrap.JenkinsBootstrap(self.spec, self.paths, self.manager)
        self.selected = {"runtimeOperationId": "start-123", "driverDigest": "a" * 64}
        self.parameters = {"baseName": BASE, "homePath": str(self.paths.home), "controllerPid": 20,
            "selectorPath": str(self.paths.state / "deployment/driver.json"), "selectorSha256": "b" * 64,
            "runtimeFence": {"operationId": "start-123", "driverDigest": "a" * 64,
                             "supervisor": {"pid": 10, "creationTime": "100"},
                             "controller": {"pid": 20, "creationTime": "200"}}}
        self.desired = node_xml(BASE, self.paths.agent, executors=2)

    def item(self, name=OLD, *, legacy=True, executors=None):
        if executors is None:
            executors = 2 if name == BASE and not legacy else 1
        raw = node_xml(name, self.paths.agent, legacy=legacy, executors=executors)
        return {"name": name, "configBase64": base64.b64encode(raw).decode(),
                "diskConfigBase64": base64.b64encode(raw + b"\n").decode(),
                "metadata": {"nodeClass": "hudson.slaves.DumbSlave", "numExecutors": executors,
                             "offline": True, "channelConnected": False, "computerAbsent": executors == 0,
                             "executors": [{"idle": True, "currentExecutable": None} for _ in range(executors)],
                             "oneOffExecutors": []}}

    def snapshot(self, *items, foreign=()):
        return {"homePath": str(self.paths.home), "allNames": sorted([item["name"] for item in items] + list(foreign)),
                "queue": [], "nodes": list(items)}

    def review(self, *items):
        return {"legacyLabel": "zircon-windows", "nodes": {item["name"]: {"config": {
            "sha256": hashlib.sha256(base64.b64decode(item["configBase64"])).hexdigest()}} for item in items}}

    def setup(self, before, *, review=None, after=None):
        stack = ExitStack()
        self.fence = stack.enter_context(patch.object(self.actor, "_parameters", return_value=deepcopy(self.parameters)))
        stack.enter_context(patch.object(self.actor, "_migration_review", return_value=review))
        self.observation = stack.enter_context(patch.object(self.actor, "_snapshot", return_value=deepcopy(before)))
        self.post = stack.enter_context(patch.object(self.actor, "_node_request", return_value=after))
        return stack


class AgentNodeMigrationTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory(prefix="agent-migration-test-", dir=ROOT / "tools/jenkins/tests")
        self.addCleanup(self.temp.cleanup)
        self.nodes = Nodes(Path(self.temp.name))

    def test_exact_legacy_nodes_archive_api_disk_and_metadata_before_any_mutation(self):
        n = self.nodes
        items = [n.item(BASE), n.item(OLD)]
        before = n.snapshot(*items, foreign=("foreign-node", BASE + "-0123456789ABCDEG"))
        after = n.snapshot(n.item(BASE, legacy=False), foreign=("foreign-node", BASE + "-0123456789ABCDEG"))
        def post(script, **kwargs):
            archive = n.paths.state / "deployment/retired-agent-nodes"
            configs = [path.read_bytes() for path in archive.glob("*.xml")]
            metadata = [json.loads(path.read_bytes()) for path in archive.glob("*.json")]
            self.assertEqual(4, len(configs))
            self.assertEqual(2, len(metadata))
            for item in items:
                self.assertIn(base64.b64decode(item["configBase64"]), configs)
                self.assertIn(base64.b64decode(item["diskConfigBase64"]), configs)
            for value in metadata:
                self.assertEqual(n.parameters["runtimeFence"], value["runtimeFence"])
                self.assertFalse(value["buildHistoryDeleted"])
                self.assertFalse(value["foreignNodesModified"])
            self.assertTrue(kwargs["mutation"])
            return after
        with n.setup(before, review=n.review(*items), after=after):
            n.post.side_effect = post
            result = n.actor._prepare_agent(n.selected, n.desired.decode())
        self.assertEqual([OLD], result["retiredNodes"])
        self.assertEqual(2, len(result["originalNodeArchives"]))
        self.assertFalse(result["configurationUnchanged"])
        self.assertFalse(result["readOnlyReconciled"])
        self.assertEqual(1, n.post.call_count)

    def test_matching_name_never_authorizes_foreign_or_unreviewed_legacy_nodes(self):
        n = self.nodes
        item = n.item()
        review = n.review(item)
        for problem in ("no-review", "changed-config", "foreign-root", "normal", "description", "label", "launcher", "properties", "disk"):
            with self.subTest(problem=problem):
                bad = deepcopy(item)
                raw = base64.b64decode(bad["configBase64"])
                if problem == "changed-config":
                    raw += b"\n"
                elif problem == "foreign-root":
                    raw = raw.replace(str(n.paths.agent).encode(), str(n.paths.agent.parent / "foreign").encode())
                elif problem == "normal":
                    raw = raw.replace(b"<mode>EXCLUSIVE", b"<mode>NORMAL")
                elif problem == "description":
                    raw = raw.replace(b"</name>", b"</name><description>Foreign node</description>")
                elif problem == "label":
                    raw = raw.replace(b"<label>zircon-windows-", b"<label>foreign-")
                elif problem == "launcher":
                    raw = raw.replace(b"<webSocket>true", b"<webSocket>false")
                elif problem == "properties":
                    raw = raw.replace(b"<nodeProperties/>", b"<nodeProperties><foreign/></nodeProperties>")
                bad["configBase64"] = base64.b64encode(raw).decode()
                if problem == "disk":
                    bad["diskConfigBase64"] = base64.b64encode(raw.replace(b"<mode>EXCLUSIVE", b"<mode>NORMAL")).decode()
                with n.setup(n.snapshot(bad), review=None if problem == "no-review" else review), self.assertRaises(JenkinsError):
                    n.actor._prepare_agent(n.selected, n.desired.decode())
                n.post.assert_not_called()
                self.assertFalse((n.paths.state / "deployment/retired-agent-nodes").exists())

    def test_fresh_offline_idle_queue_and_complete_executor_observations_are_required(self):
        n = self.nodes
        item = n.item()
        for problem in ("online", "connected", "busy", "one-off-busy", "executor-missing", "offline-missing", "queue", "duplicate", "inventory-missing"):
            with self.subTest(problem=problem):
                bad = deepcopy(item)
                if problem == "online":
                    bad["metadata"]["offline"] = False
                elif problem == "connected":
                    bad["metadata"]["channelConnected"] = True
                elif problem == "busy":
                    bad["metadata"]["executors"][0]["currentExecutable"] = "live-build"
                elif problem == "one-off-busy":
                    bad["metadata"]["oneOffExecutors"] = [{"idle": False, "currentExecutable": "live-build"}]
                elif problem == "executor-missing":
                    bad["metadata"]["executors"] = []
                elif problem == "offline-missing":
                    del bad["metadata"]["offline"]
                before = n.snapshot(bad)
                if problem == "queue":
                    before["queue"] = [{"id": 1}]
                elif problem == "duplicate":
                    before["nodes"].append(deepcopy(bad))
                elif problem == "inventory-missing":
                    before["nodes"] = []
                with n.setup(before, review=n.review(item)), self.assertRaises(JenkinsError):
                    n.actor._prepare_agent(n.selected, n.desired.decode())
                n.post.assert_not_called()

    def test_fixed_described_node_is_idempotent_without_legacy_review_or_rewrite(self):
        n = self.nodes
        before = n.snapshot(n.item(BASE, legacy=False), foreign=("foreign-node",))
        with n.setup(before):
            result = n.actor._prepare_agent(n.selected, n.desired.decode())
        n.post.assert_not_called()
        self.assertTrue(result["configurationUnchanged"])
        self.assertEqual([], result["retiredNodes"])
        self.assertFalse((n.paths.state / "deployment/retired-agent-nodes").exists())

    def test_configured_executor_count_is_limited_to_fixed_normal_node(self):
        n = self.nodes
        fixed = n.item(BASE, legacy=False)
        with n.setup(n.snapshot(fixed)):
            result = n.actor._prepare_agent(n.selected, n.desired.decode())
        self.assertTrue(result["configurationUnchanged"])
        n.post.assert_not_called()

        for legacy in (False, True):
            with self.subTest(legacy=legacy):
                item = n.item(executors=2, legacy=legacy)
                review = n.review(item) if legacy else None
                with n.setup(n.snapshot(item), review=review), self.assertRaises(JenkinsError):
                    n.actor._prepare_agent(n.selected, n.desired.decode())
                n.post.assert_not_called()

    def test_online_fixed_node_and_pending_queue_need_no_migration(self):
        n = self.nodes
        fixed = n.item(BASE, legacy=False)
        fixed["metadata"].update(offline=False, channelConnected=True)
        fixed["metadata"]["executors"][0].update(idle=False, currentExecutable="job/active/1/")
        before = n.snapshot(fixed)
        before["queue"] = [{"id": 123}]
        with n.setup(before):
            result = n.actor._prepare_agent(n.selected, n.desired.decode())
        self.assertTrue(result["configurationUnchanged"])
        self.assertEqual([], result["retiredNodes"])
        n.post.assert_not_called()
        self.assertFalse(n.paths.state.exists())

    def test_busy_or_queued_fixed_node_still_rejects_actual_configuration_changes(self):
        n = self.nodes
        for online in (False, True):
            with self.subTest(online=online):
                fixed = n.item(BASE, legacy=False, executors=1)
                if online:
                    fixed["metadata"].update(offline=False, channelConnected=True)
                    fixed["metadata"]["executors"][0].update(idle=False, currentExecutable="job/active/1/")
                before = n.snapshot(fixed)
                if not online:
                    before["queue"] = [{"id": 123}]
                with n.setup(before), self.assertRaises(JenkinsError):
                    n.actor._prepare_agent(n.selected, n.desired.decode())
                n.post.assert_not_called()
                self.assertFalse(n.paths.state.exists())

    def test_absent_reviewed_legacy_nodes_do_not_block_later_fixed_startup(self):
        n = self.nodes
        old = n.item()
        with n.setup(n.snapshot(n.item(BASE, legacy=False)), review=n.review(old)):
            n.actor._prepare_agent(n.selected, n.desired.decode())
        n.post.assert_not_called()

    def test_zero_executor_offline_legacy_node_can_be_retired_with_exact_review(self):
        n = self.nodes
        old = n.item(executors=0)
        after = n.snapshot(n.item(BASE, legacy=False))
        with n.setup(n.snapshot(old), review=n.review(old), after=after):
            result = n.actor._prepare_agent(n.selected, n.desired.decode())
        self.assertEqual([OLD], result["retiredNodes"])

    def test_unknown_mutation_reconciles_once_without_another_mutation(self):
        n = self.nodes
        old = n.item()
        before, after = n.snapshot(old), n.snapshot(n.item(BASE, legacy=False))
        for outcome in (after, before):
            with self.subTest(committed=outcome == after), n.setup(before, review=n.review(old)):
                n.post.side_effect = JenkinsError("agent_node_migration_unknown", "timeout")
                n.observation.side_effect = [before, outcome]
                if outcome == after:
                    result = n.actor._prepare_agent(n.selected, n.desired.decode())
                    self.assertTrue(result["readOnlyReconciled"])
                else:
                    with self.assertRaises(JenkinsError) as raised:
                        n.actor._prepare_agent(n.selected, n.desired.decode())
                    self.assertEqual("agent_node_migration_unknown", raised.exception.code)
                self.assertEqual(1, n.post.call_count)
                self.assertEqual(2, n.observation.call_count)

    def test_runtime_drift_or_archive_collision_never_reaches_mutation(self):
        n = self.nodes
        old = n.item()
        with n.setup(n.snapshot(old), review=n.review(old)):
            n.fence.side_effect = [n.parameters, {**n.parameters, "controllerPid": 21}]
            with self.assertRaises(JenkinsError):
                n.actor._prepare_agent(n.selected, n.desired.decode())
            n.post.assert_not_called()
        config = next((n.paths.state / "deployment/retired-agent-nodes").glob("*api-config*.xml"))
        config.write_bytes(b"foreign preexisting archive")
        with n.setup(n.snapshot(old), review=n.review(old)), self.assertRaises(JenkinsError):
            n.actor._prepare_agent(n.selected, n.desired.decode())
        n.post.assert_not_called()
        self.assertEqual(b"foreign preexisting archive", config.read_bytes())

    def test_unexpected_foreign_registration_changes_keep_outcome_unknown(self):
        n = self.nodes
        old = n.item()
        after = n.snapshot(n.item(BASE, legacy=False), foreign=("new-foreign",))
        with n.setup(n.snapshot(old), review=n.review(old), after=after), self.assertRaises(JenkinsError) as raised:
            n.actor._prepare_agent(n.selected, n.desired.decode())
        self.assertEqual("agent_node_migration_unknown", raised.exception.code)
        self.assertEqual(1, n.post.call_count)

    def test_snapshot_allows_online_read_only_preflight_without_archive_or_mutation(self):
        n = self.nodes
        item = n.item()
        item["metadata"]["offline"] = False
        before = n.snapshot(item)
        with patch.object(n.actor, "_parameters", return_value=n.parameters), patch.object(n.actor, "_node_request", return_value=before) as post:
            result = n.actor._snapshot(n.selected)
        self.assertEqual(before, result)
        self.assertNotIn("mutation", post.call_args.kwargs)
        self.assertNotIn("removeNode", post.call_args.args[0])
        self.assertNotIn("replaceNode", post.call_args.args[0])
        self.assertFalse(n.paths.state.exists())

    def test_actual_serialization_defaults_match_but_unknown_settings_never_do(self):
        n = self.nodes
        minimal = n.desired.replace(b"<workDirSettings><disabled>true</disabled><internalDir>remoting</internalDir>"
            b"<failIfWorkDirIsMissing>false</failIfWorkDirIsMissing></workDirSettings>", b"").replace(b"<nodeProperties/>", b"")
        self.assertTrue(n.actor._node_config_matches(BASE, minimal, n.desired))
        for field in (b"<disabled>true", b"<internalDir>remoting", b"<failIfWorkDirIsMissing>false"):
            self.assertFalse(n.actor._node_config_matches(BASE, minimal, n.desired.replace(field, field + b"unknown")))
        self.assertFalse(n.actor._node_config_matches(BASE, minimal,
                         n.desired.replace(b"</launcher>", b"<tunnel>foreign:123</tunnel></launcher>")))

    def test_bootstrap_uses_the_exact_node_name_and_real_description_property(self):
        n = self.nodes
        selector = n.paths.state / "deployment/driver.json"
        selector.parent.mkdir(parents=True)
        selector.write_bytes(canonical_json(n.selected))
        with patch.object(n.actor, "_post_script"), patch.object(n.actor, "_prepare_agent", return_value={}) as prepare:
            result = n.actor.apply()
        self.assertEqual(BASE, result["agentName"])
        self.assertEqual(BASE, result["agentLabel"])
        xml = prepare.call_args.args[1]
        self.assertIn("<description>Zircon Jenkins runtime agent</description>", xml)
        self.assertNotIn("nodeDescription", xml)
        self.assertIn("<label>" + BASE + "</label>", xml)

    def test_job_node_selector_uses_name_while_preserving_runtime_and_digest_guards(self):
        n = self.nodes
        n.spec.raw = {"pipeline": {"jobs": ["build"]}}
        selector = n.paths.state / "deployment/driver.json"
        selector.parent.mkdir(parents=True)
        selector.write_bytes(canonical_json(n.selected))
        n.manager._driver_env.return_value = {"ZIRCON_REPO_ROOT": str(n.paths.repo), "JENKINS_PYTHON": "owned-python",
            "ZIRCON_SEALED_DRIVER": "sealed-root", "ZIRCON_DRIVER_LAUNCHER": "sealed-launcher", "ZIRCON_DRIVER_DIGEST": "a" * 64}
        n.manager._credentials.return_value = ("user", "token")
        n.manager.base_url = "http://127.0.0.1:18080"
        response = Mock()
        response.__enter__ = Mock(return_value=SimpleNamespace(status=200))
        response.__exit__ = Mock(return_value=False)
        with patch.object(job_provisioning, "_driver_snapshot", return_value=(ROOT, "a" * 64)), \
             patch.object(job_provisioning, "_pipeline_source", return_value="node(env.ZIRCON_AGENT_LABEL) {}"), \
             patch.object(job_provisioning.urllib.request, "urlopen", return_value=response) as post:
            job_provisioning.provision_jobs(n.spec, n.paths, n.manager)
        payload = post.call_args.args[0].data.decode()
        self.assertIn('env.ZIRCON_AGENT_LABEL="' + BASE + '"', payload)
        self.assertIn('env.ZIRCON_RUNTIME_OPERATION_ID="start-123"', payload)
        self.assertIn('env.ZIRCON_DRIVER_DIGEST="' + "a" * 64 + '"', payload)
        self.assertNotIn('env.ZIRCON_AGENT_LABEL="zircon-windows"', payload)

    def write_review(self):
        n = self.nodes
        root = n.paths.state / "migration-baseline"
        originals = root / "readonly-node-inventory"
        originals.mkdir(parents=True)
        def save(path, value):
            raw = canonical_json(value) + b"\n" if isinstance(value, dict) else value
            path.write_bytes(raw)
            return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest()}
        item = n.item()
        original = save(originals / (OLD + ".xml"), base64.b64decode(item["configBase64"]))
        inventory = {"readOnly": True, "serviceLifecycleExecuted": False,
            "owner": {"homePath": str(n.paths.home), "driverDigest": "c" * 64}, "driverDigest": "c" * 64,
            "computers": [{"name": OLD, "config": original}]}
        inventory_ref = save(originals / "inventory.json", inventory)
        review = {"schemaVersion": 1, "kind": "reviewed-agent-node-migration", "baseName": BASE,
            "homePath": str(n.paths.home), "remoteFS": str(n.paths.agent), "legacyLabel": "zircon-windows",
            "sourceInventory": inventory_ref, "nodes": {OLD: {"config": original}}}
        review_ref = save(root / "reviewed-agent-node-migration.json", review)
        release = {"targetDriver": {"driverDigest": n.selected["driverDigest"]}, "agentNodeMigration": review_ref}
        deployment = n.paths.state / "deployment"
        deployment.mkdir()
        n.selected["recoveryRelease"] = save(deployment / "release.json", release)
        return review, inventory, save

    def test_legacy_review_is_bound_to_exact_release_inventory_and_original_xml_bytes(self):
        n = self.nodes
        review, inventory, _ = self.write_review()
        self.assertEqual(review, n.actor._migration_review(n.selected))
        original = Path(review["nodes"][OLD]["config"]["path"])
        original.write_bytes(original.read_bytes() + b"\n")
        with self.assertRaises(JenkinsError):
            n.actor._migration_review(n.selected)

    def test_reviewed_inventory_never_substitutes_for_fresh_offline_state(self):
        n = self.nodes
        review, inventory, _ = self.write_review()
        self.assertEqual(review, n.actor._migration_review(n.selected))
        live = n.item()
        live["metadata"]["offline"] = False
        with self.assertRaises(JenkinsError):
            n.actor._owned_nodes(n.snapshot(live), review)

    def test_review_marker_rejects_source_identity_and_inventory_namespace_drift(self):
        n = self.nodes
        review, inventory, save = self.write_review()
        source = Path(review["sourceInventory"]["path"])
        source.write_bytes(source.read_bytes() + b"\n")
        with self.assertRaises(JenkinsError):
            n.actor._migration_review(n.selected)
        save(source, inventory)
        outside = n.paths.state / "foreign-inventory.json"
        review["sourceInventory"] = save(outside, inventory)
        marker = Path(n.paths.state / "migration-baseline/reviewed-agent-node-migration.json")
        release = {"targetDriver": {"driverDigest": "a" * 64}, "agentNodeMigration": save(marker, review)}
        n.selected["recoveryRelease"] = save(n.paths.state / "deployment/release.json", release)
        with self.assertRaises(JenkinsError):
            n.actor._migration_review(n.selected)

    def test_node_script_timeout_is_unknown_without_any_automatic_post_retry(self):
        n = self.nodes
        n.manager._request.side_effect = JenkinsError("controller_unreachable", "transport timeout")
        with self.assertRaises(JenkinsError) as raised:
            n.actor._node_request("owned mutation", mutation=True)
        self.assertEqual("agent_node_migration_unknown", raised.exception.code)
        self.assertEqual(1, n.manager._request.call_count)


if __name__ == "__main__":
    unittest.main()
