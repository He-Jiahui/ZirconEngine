from __future__ import annotations

import json
import base64
import hashlib
import os
from pathlib import Path
import re
import urllib.parse
import xml.etree.ElementTree as ET
from xml.sax.saxutils import escape

from .spec import JenkinsError
from .driver import _plain, runtime_agent_label, runtime_agent_name, verify_driver
from ..contracts import canonical_json

_DESCRIPTION = "Zircon Jenkins runtime agent"
_NODE_MARKER = "ZR_OWNED_AGENT_NODES="
_NODE_SNAPSHOT = r'''
def j = jenkins.model.Jenkins.get()
if (j.rootDir.canonicalPath != new File(p.homePath).canonicalPath ||
    java.lang.management.ManagementFactory.runtimeMXBean.name.split('@')[0].toLong() != p.controllerPid) { throw new IllegalStateException('Controller ownership changed') }
def sha = { byte[] raw -> java.security.MessageDigest.getInstance('SHA-256').digest(raw).encodeHex().toString() }
def selector = new File(p.selectorPath)
if (sha(selector.bytes) != p.selectorSha256) { throw new IllegalStateException('Runtime selector changed') }
def names = { -> j.nodes.collect { it.nodeName }.sort() }
def selected = { String name -> name == p.baseName || name ==~ java.util.regex.Pattern.quote(p.baseName) + '-[0-9a-f]{16}' }
def snapshot = { ->
    def queue = j.queue.items.collect { [id: it.id] }
    def nodes = j.nodes.findAll { selected(it.nodeName) }.sort { it.nodeName }.collect { n ->
        def dir = n.rootDir
        def file = j.nodesObject.getConfigFile(n).file
        def files = dir.listFiles()
        if (dir.canonicalFile != dir.absoluteFile || file.canonicalFile != file.absoluteFile ||
            files == null || files.size() != 1 || files[0].name != 'config.xml' || !file.isFile() ||
            file.length() > 1048576) { throw new IllegalStateException('Unreviewed node files or aliases') }
        def out = new java.io.ByteArrayOutputStream()
        jenkins.model.Jenkins.XSTREAM2.toXMLUTF8(n, out)
        def c = n.toComputer()
        def executors = c == null ? [] : c.executors.collect { e -> [idle: e.isIdle(), currentExecutable: e.currentExecutable == null ? null : e.currentExecutable.toString()] }
        def oneOff = c == null ? [] : c.oneOffExecutors.collect { e -> [idle: e.isIdle(), currentExecutable: e.currentExecutable == null ? null : e.currentExecutable.toString()] }
        [name: n.nodeName, configBase64: out.toByteArray().encodeBase64().toString(),
         diskConfigBase64: file.bytes.encodeBase64().toString(),
         metadata: [nodeClass: n.class.name, numExecutors: n.numExecutors, offline: c == null ? n.numExecutors == 0 : c.offline,
                    computerAbsent: c == null, channelConnected: c != null && c.channel != null,
                    executors: executors, oneOffExecutors: oneOff]]
    }
    [homePath: j.rootDir.canonicalPath, allNames: names(), queue: queue, nodes: nodes]
}
'''


def _require(condition, message, code="agent_node_migration_unproven"):
    if not condition:
        raise JenkinsError(code, message, retryable=True)


def _parse_node_config(raw, *, allow_legacy=False):
    if (not isinstance(raw, bytes) or not raw or len(raw) > 1_048_576
            or b"<!DOCTYPE" in raw.upper() or b"<!ENTITY" in raw.upper()):
        raise ValueError("Unknown node XML")
    root = ET.fromstring(raw)
    if root.tag not in {"slave", "hudson.slaves.DumbSlave"} or root.attrib or (root.text or "").strip():
        raise ValueError("Unknown node class")
    children = list(root)
    tags = {child.tag for child in children}
    required = {"name", "remoteFS", "numExecutors", "label", "mode", "retentionStrategy", "launcher"}
    descriptions = tags & {"description", "nodeDescription"}
    if (len(tags) != len(children) or not required <= tags or len(descriptions) > 1
            or not tags <= required | {"description", "nodeDescription", "nodeProperties"}
            or not descriptions and not allow_legacy):
        raise ValueError("Incomplete or duplicate node configuration")
    for child in children:
        if child.tag not in {"launcher", "retentionStrategy"} and (child.attrib or list(child)):
            raise ValueError("Unknown node field")
        if child.tail and not child.tail.isspace():
            raise ValueError("Unknown XML tail")
    retention = root.find("retentionStrategy")
    launcher = root.find("launcher")
    if (retention.attrib != {"class": "hudson.slaves.RetentionStrategy$Always"} or list(retention)
            or (retention.text or "").strip() or launcher.attrib != {"class": "hudson.slaves.JNLPLauncher"}):
        raise ValueError("Unknown node launch strategy")
    launch_fields = list(launcher)
    if (len(launch_fields) != len({child.tag for child in launch_fields})
            or {child.tag for child in launch_fields} not in ({"webSocket"}, {"workDirSettings", "webSocket"})
            or launcher.findtext("webSocket") != "true" or launcher.find("webSocket").attrib
            or list(launcher.find("webSocket")) or (launcher.text or "").strip()):
        raise ValueError("Unknown inbound launch settings")
    work = launcher.find("workDirSettings")
    if work is not None:
        defaults = {"disabled": "true", "internalDir": "remoting", "failIfWorkDirIsMissing": "false"}
        if (work.attrib or len(list(work)) != len(defaults) or (work.text or "").strip()
                or {child.tag: child.text for child in work} != defaults
                or any(child.attrib or list(child) or child.tail and not child.tail.isspace() for child in work)):
            raise ValueError("Unknown remoting work directory settings")
    if any(child.tail and not child.tail.isspace() for child in launch_fields):
        raise ValueError("Unknown launcher tail")
    properties = root.find("nodeProperties")
    if properties is not None and (properties.text or "").strip():
        raise ValueError("Unknown node properties")
    desc = next(iter(descriptions), None)
    return {name: root.findtext(name) or "" for name in required - {"launcher", "retentionStrategy"}} | {
        "descriptionTag": desc, "description": root.findtext(desc) or "" if desc else ""}


class JenkinsBootstrap:
    """Apply controller safety settings through authenticated Jenkins APIs."""

    def __init__(self, spec, paths, manager):
        self.spec, self.paths, self.manager = spec, paths, manager

    def apply(self) -> dict:
        # Use the supported script endpoint to change only the controller
        # executor count.  This preserves the security realm, users and keys.
        script = ("Jenkins.get().setNumExecutors(0); Jenkins.get().save(); "
                  "def l=jenkins.model.JenkinsLocationConfiguration.get(); "
                  "l.setUrl('http://127.0.0.1:%s/'); l.save(); "
                  "def old=Jenkins.get().getItem('zircon-pilot'); if(old!=null){old.setDisabled(true); old.save()}" % self.spec.controller["httpPort"])
        self._post_script(script)
        node = self.spec.agent
        selected = json.loads((self.paths.state / "deployment" / "driver.json").read_text(encoding="utf-8"))
        runtime_operation_id = selected.get("runtimeOperationId")
        agent_name = runtime_agent_name(str(node["name"]), str(runtime_operation_id))
        agent_label = runtime_agent_label(str(node["name"]), str(runtime_operation_id))
        node_xml = ("<hudson.slaves.DumbSlave>"
                    f"<name>{escape(agent_name)}</name><description>{_DESCRIPTION}</description>"
                    f"<remoteFS>{escape(str(self.paths.agent))}</remoteFS><numExecutors>{int(node.get('executors', 1))}</numExecutors>"
                    f"<label>{escape(agent_label)}</label><mode>EXCLUSIVE</mode><retentionStrategy class='hudson.slaves.RetentionStrategy$Always'/>"
                    "<launcher class='hudson.slaves.JNLPLauncher'><workDirSettings><disabled>true</disabled>"
                    "<internalDir>remoting</internalDir><failIfWorkDirIsMissing>false</failIfWorkDirIsMissing>"
                    "</workDirSettings><webSocket>true</webSocket></launcher><nodeProperties/></hudson.slaves.DumbSlave>")
        migration = self._prepare_agent(selected, node_xml)
        return {"bootstrapApplied": True, "executors": 0, "agentConfigured": True, "authenticated": True,
                "agentName": agent_name, "agentLabel": agent_label, "runtimeOperationId": str(runtime_operation_id),
                "agentNodeMigration": migration}

    def _parameters(self, selected):
        from .drain_observations import _identity
        from .live_observations import observe_endpoint_owner, observe_live_identity
        state = self.paths.state.absolute()
        for path in (self.paths.home.absolute(), self.paths.agent.absolute()):
            _plain(path)
        selector = state / "deployment/driver.json"
        _plain(selector)
        raw = selector.read_bytes()
        _require(json.loads(raw) == selected, "Runtime selector changed")
        checksum = selected.get("driverDigest")
        root = self.paths.repo.absolute() / ".jenkins/runtime/drivers" / str(checksum)
        _require(selected.get("root") == str(root), "Runtime driver root changed")
        verify_driver(root, expected_digest=checksum)
        for path in (state / "deployment-operation.json", state / "deployment/host.json"):
            _plain(path)
        operation = json.loads((state / "deployment-operation.json").read_bytes())
        host = json.loads((state / "deployment/host.json").read_bytes())
        runtime = selected.get("runtimeOperationId")
        _require(operation.get("state") in {"starting", "running"} and host.get("status") in {"starting", "running"}
                 and operation.get("operationId") == operation.get("generation") == host.get("operationId")
                    == host.get("generation") == runtime
                 and operation.get("component") == "lifecycle-host"
                 and operation.get("homePath") == str(self.paths.home.absolute())
                 and operation.get("pid") == operation.get("hostPid") == host.get("hostPid"),
                 "Agent configuration requires the exact native runtime owner")
        supervisor = _identity(operation, self.paths.repo / ".jenkins/runtime/python/python.exe")
        controller = _identity(host.get("controllerIdentity") or host.get("identity"), self.manager.java)
        _require(host.get("identity") == controller, "Controller native identities conflict")
        for identity in (supervisor, controller):
            observe_live_identity(identity)
        observe_endpoint_owner(self.spec.controller["listenAddress"], int(self.spec.controller["httpPort"]), controller["pid"])
        _require(self.paths.agent.absolute() == self.paths.repo.absolute() / ".jenkins/agent",
                 "Only the exact repository agent root is owned")
        return {"baseName": str(self.spec.agent["name"]), "homePath": str(self.paths.home.absolute()),
                "controllerPid": controller["pid"], "selectorPath": str(selector),
                "selectorSha256": hashlib.sha256(raw).hexdigest(),
                "runtimeFence": {"operationId": runtime, "driverDigest": checksum,
                                 "supervisor": supervisor, "controller": controller}}

    def _migration_review(self, selected):
        from .recovery_keeper import _reference
        state = self.paths.state.absolute()
        release_ref = selected.get("recoveryRelease")
        if release_ref is None:
            return None
        release = _reference(state, release_ref, release=True)
        _require(release.get("targetDriver", {}).get("driverDigest") == selected.get("driverDigest"),
                 "Agent review release changed its sealed driver")
        marker = release.get("agentNodeMigration")
        if marker is None:
            return None
        review = _reference(state, marker)
        _require(Path(marker["path"]).parent == state / "migration-baseline"
                 and review.get("schemaVersion") == 1 and review.get("kind") == "reviewed-agent-node-migration"
                 and review.get("baseName") == self.spec.agent["name"]
                 and review.get("homePath") == str(self.paths.home.absolute())
                 and review.get("remoteFS") == str(self.paths.agent.absolute())
                 and review.get("legacyLabel") == self.spec.agent.get("label", self.spec.agent["name"])
                 and isinstance(review.get("nodes"), dict), "Agent migration review has a different ownership boundary")
        def original(reference, parent=None):
            _require(isinstance(reference, dict) and set(reference) == {"path", "sha256"}, "Exact original node reference required")
            path = Path(reference["path"]).absolute()
            _require(str(path) == reference["path"] and ".." not in path.parts
                     and path.is_relative_to(state / "migration-baseline")
                     and (parent is None or path.parent == parent), "Original node review escaped its immutable namespace")
            _plain(path)
            raw = path.read_bytes()
            _require(hashlib.sha256(raw).hexdigest() == reference["sha256"], "Original node review bytes changed")
            return raw
        inventory = json.loads(original(review.get("sourceInventory")))
        inventory_parent = Path(review["sourceInventory"]["path"]).parent
        _require(inventory.get("readOnly") is True and inventory.get("serviceLifecycleExecuted") is False
                 and inventory.get("owner", {}).get("homePath") == review["homePath"]
                 and inventory.get("driverDigest") == inventory.get("owner", {}).get("driverDigest")
                 and isinstance(inventory.get("computers"), list), "Original node review inventory is unproven")
        reviewed = {}
        for item in inventory["computers"]:
            name = item.get("name")
            _require(isinstance(name, str) and name not in reviewed, "Original node review contains duplicate identities")
            reviewed[name] = item.get("config")
        _require(set(review["nodes"]) == set(reviewed), "Reviewed node identities changed")
        for name, entry in review["nodes"].items():
            _require(re.fullmatch(re.escape(review["baseName"]) + r"(?:-[0-9a-f]{16})?", name)
                     and isinstance(entry, dict) and entry.get("config") == reviewed[name],
                     "Only exact original repository node configurations may be reviewed")
            _require(Path(entry["config"]["path"]).name == name + ".xml", "Reviewed configuration lost its exact node identity")
            original(entry["config"], inventory_parent)
        return review

    def _node_request(self, script, *, mutation=False):
        payload = urllib.parse.urlencode({"script": script}).encode()
        headers = {"Content-Type": "application/x-www-form-urlencoded"}
        try:
            status, body = self.manager._request("/scriptText", method="POST", data=payload, headers=headers, timeout=60)
            if status == 403:
                crumb_status, crumb_raw = self.manager._request("crumbIssuer/api/json", timeout=20)
                _require(crumb_status == 200, "Authenticated node observation crumb is unavailable")
                crumb = json.loads(crumb_raw)
                status, body = self.manager._request("/scriptText", method="POST", data=payload,
                    headers={**headers, crumb["crumbRequestField"]: crumb["crumb"]}, timeout=60)
        except JenkinsError as error:
            if mutation and error.code == "controller_unreachable":
                raise JenkinsError("agent_node_migration_unknown", "Node maintenance outcome requires read-only reconciliation") from error
            raise
        lines = [line[len(_NODE_MARKER):] for line in body.decode("utf-8").splitlines() if line.startswith(_NODE_MARKER)]
        _require(status == 200 and len(lines) == 1, "Node maintenance did not return its exact observation",
                 "agent_node_migration_unknown" if mutation and status == 200 else "agent_node_observation_failed")
        try:
            return json.loads(lines[0])
        except ValueError as error:
            raise JenkinsError("agent_node_migration_unknown" if mutation else "agent_node_observation_failed",
                               "Node maintenance returned incomplete JSON") from error

    @staticmethod
    def _script(parameters, tail):
        encoded = base64.b64encode(canonical_json(parameters)).decode("ascii")
        return ("def p = new groovy.json.JsonSlurper().parseText(new String('" + encoded
                + "'.decodeBase64(), 'UTF-8'))\n" + _NODE_SNAPSHOT + tail)

    def _snapshot(self, selected=None):
        """Read-only live preflight, including online nodes; never archive or mutate."""
        if selected is None:
            selected = json.loads((self.paths.state / "deployment/driver.json").read_bytes())
        parameters = self._parameters(selected)
        result = self._node_request(self._script(parameters,
            "def value = hudson.model.Queue.withLock({ -> snapshot() } as java.util.concurrent.Callable)\n"
            "println('" + _NODE_MARKER + "' + groovy.json.JsonOutput.toJson(value))"))
        _require(isinstance(result, dict) and result.get("homePath") == parameters["homePath"]
                 and isinstance(result.get("allNames"), list) and isinstance(result.get("nodes"), list)
                 and isinstance(result.get("queue"), list), "Live node inventory is incomplete")
        return result

    def _owned_nodes(self, snapshot, review, *, maintenance=True):
        _require(isinstance(snapshot, dict) and snapshot.get("homePath") == str(self.paths.home.absolute())
                 and isinstance(snapshot.get("allNames"), list) and isinstance(snapshot.get("nodes"), list),
                 "Owned node observation is incomplete")
        _require(isinstance(snapshot.get("queue"), list), "Owned queue observation is incomplete")
        if maintenance:
            _require(snapshot["queue"] == [], "Pending Jenkins queue ownership blocks node maintenance")
        base = self.spec.agent["name"]
        names = snapshot["allNames"]
        _require(all(isinstance(name, str) for name in names) and len(names) == len(set(names)), "Node inventory contains duplicate identities")
        expected = {name for name in names if re.fullmatch(re.escape(base) + r"(?:-[0-9a-f]{16})?", name)}
        owned = {}
        for item in snapshot["nodes"]:
            name = item.get("name")
            _require(name in expected and name not in owned, "Owned node inventory changed")
            try:
                raw = base64.b64decode(item["configBase64"], validate=True)
                disk = base64.b64decode(item["diskConfigBase64"], validate=True)
                config = _parse_node_config(raw, allow_legacy=True)
                _require(_parse_node_config(disk, allow_legacy=True) == config,
                         "Persisted node configuration differs from its exact live identity")
            except (KeyError, ValueError, TypeError, ET.ParseError) as error:
                raise JenkinsError("agent_node_migration_unproven", "Owned node configuration is incomplete") from error
            suffix = name[len(base):]
            normal = (config["descriptionTag"] == "description" and config["description"] == _DESCRIPTION
                      and config["mode"] == "EXCLUSIVE" and config["label"] == name)
            legacy = (review is not None and review["nodes"].get(name, {}).get("config", {}).get("sha256") == hashlib.sha256(raw).hexdigest()
                      and config["descriptionTag"] is None and config["description"] == ""
                      and config["mode"] == ("NORMAL" if name == base else "EXCLUSIVE")
                      and config["label"] == review["legacyLabel"] + suffix)
            metadata = item.get("metadata", {})
            configured_executors = self.spec.agent.get("executors", 1)
            accepted_executors = {"0", "1"}
            if name == base and normal and type(configured_executors) is int and configured_executors > 0:
                accepted_executors.add(str(configured_executors))
            executors, one_off = metadata.get("executors"), metadata.get("oneOffExecutors")
            _require(config["name"] == name and config["remoteFS"] == str(self.paths.agent.absolute())
                     and config["numExecutors"] in accepted_executors and (normal or legacy)
                     and bool(disk) and len(disk) <= 1_048_576
                     and metadata.get("nodeClass") == "hudson.slaves.DumbSlave"
                     and type(metadata.get("numExecutors")) is int and metadata["numExecutors"] == int(config["numExecutors"])
                     and type(metadata.get("offline")) is bool and type(metadata.get("channelConnected")) is bool
                     and type(metadata.get("computerAbsent")) is bool
                     and (not metadata["computerAbsent"] or metadata["numExecutors"] == 0)
                     and isinstance(executors, list) and len(executors) == metadata["numExecutors"]
                     and isinstance(one_off, list) and all(isinstance(executor, dict)
                        and type(executor.get("idle")) is bool for executor in executors + one_off),
                     "Foreign or incompletely owned node must be preserved")
            if maintenance:
                _require(metadata["offline"] is True and metadata["channelConnected"] is False
                         and all(executor["idle"] is True and executor.get("currentExecutable") is None
                                 for executor in executors + one_off),
                         "Online or busy node must be preserved during maintenance")
            owned[name] = {"observation": item, "raw": raw, "disk": disk}
        _require(set(owned) == expected, "Owned node registration inventory is incomplete")
        return owned

    def _archive_node(self, name, evidence, parameters):
        root = self.paths.state.absolute() / "deployment/retired-agent-nodes"
        _plain(root)
        root.mkdir(parents=True, exist_ok=True)
        def archive(prefix, raw, suffix):
            path = root / (name + "-" + prefix + "-" + hashlib.sha256(raw).hexdigest() + suffix)
            _plain(path)
            try:
                with path.open("xb") as stream:
                    stream.write(raw)
                    stream.flush()
                    os.fsync(stream.fileno())
            except FileExistsError:
                _require(path.read_bytes() == raw, "Immutable retired node archive changed")
            _require(path.read_bytes() == raw, "Retired node archive did not retain the original bytes")
            return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest()}
        config = archive("api-config", evidence["raw"], ".xml")
        disk = archive("disk-config", evidence["disk"], ".xml")
        metadata = {"schemaVersion": 1, "kind": "retired-agent-node-evidence", "name": name,
                    "config": config, "diskConfig": disk, "metadata": evidence["observation"]["metadata"],
                    "runtimeFence": parameters["runtimeFence"], "queue": [], "buildHistoryDeleted": False,
                    "foreignNodesModified": False}
        return archive("metadata", canonical_json(metadata) + b"\n", ".json")

    def _prepare_agent(self, selected, desired_xml):
        desired = desired_xml.encode("utf-8")
        base = self.spec.agent["name"]
        configured_executors = self.spec.agent.get("executors", 1)
        desired_executors = _parse_node_config(desired)["numExecutors"]
        _require(type(configured_executors) is int and configured_executors > 0
                 and desired_executors == str(configured_executors),
                 "The fixed agent executor count must match deployment configuration")
        parameters = self._parameters(selected)
        review = self._migration_review(selected)
        before = self._snapshot(selected)
        owned = self._owned_nodes(before, review, maintenance=False)
        retired = sorted(name for name in owned if name != base)
        update = base not in owned or not self._node_config_matches(base, desired, owned[base]["raw"])
        if not retired and not update:
            _require(self._parameters(selected) == parameters, "Native runtime changed during node observation")
            return {"fixedNodeName": base, "retiredNodes": [], "configurationUnchanged": True}
        # Offline migration constraints apply only when registration changes.
        self._owned_nodes(before, review)
        archives = [self._archive_node(name, owned[name], parameters) for name in retired + ([base] if update and base in owned else [])]
        _require(self._parameters(selected) == parameters, "Native runtime changed before node maintenance")
        parameters = {**parameters, "expected": before, "retired": retired, "update": update,
                      "desiredBase64": base64.b64encode(desired).decode("ascii")}
        tail = '''
def value = hudson.model.Queue.withLock({ ->
    if (snapshot() != p.expected || j.queue.items.length != 0) { throw new IllegalStateException('Owned node evidence changed') }
    def replacement = p.update ? jenkins.model.Jenkins.XSTREAM2.fromXML(new String(p.desiredBase64.decodeBase64(), 'UTF-8')) : null
    if (replacement != null && (replacement.class != hudson.slaves.DumbSlave || replacement.nodeName != p.baseName)) { throw new IllegalStateException('Invalid fixed node') }
    p.retired.each { name -> j.removeNode(j.getNode(name)) }
    if (replacement != null) {
        def previous = j.getNode(p.baseName)
        if (previous == null) { j.addNode(replacement) }
        else if (!j.nodesObject.replaceNode(previous, replacement)) { throw new IllegalStateException('Fixed node changed') }
    }
    snapshot()
} as java.util.concurrent.Callable)
println('ZR_OWNED_AGENT_NODES=' + groovy.json.JsonOutput.toJson(value))
'''
        reconciled = False
        try:
            after = self._node_request(self._script(parameters, tail), mutation=True)
        except JenkinsError as error:
            if error.code != "agent_node_migration_unknown":
                raise
            try:
                after = self._snapshot(selected)
                self._verify_node_result(before, after, desired, review, retired)
            except (JenkinsError, KeyError, TypeError, ValueError) as observation_error:
                raise JenkinsError("agent_node_migration_unknown", "Exact node maintenance outcome remains unproven",
                                   details={"archives": archives, "retiredNodes": retired}) from observation_error
            reconciled = True
        self._verify_node_result(before, after, desired, review, retired)
        return {"fixedNodeName": base, "retiredNodes": retired, "originalNodeArchives": archives,
                "configurationUnchanged": not update, "readOnlyReconciled": reconciled, "buildHistoryDeleted": False}

    def _verify_node_result(self, before, after, desired, review, retired):
        base = self.spec.agent["name"]
        _require(isinstance(after, dict) and after.get("homePath") == before["homePath"]
                 and set(after.get("allNames", [])) == set(before["allNames"]) - set(retired) | {base},
                 "Node maintenance changed an unexpected registration", "agent_node_migration_unknown")
        nodes = self._owned_nodes(after, review)
        _require(set(nodes) == {base} and self._node_config_matches(base, desired, nodes[base]["raw"]),
                 "The sole fixed node configuration remains unproven", "agent_node_migration_unknown")

    def _post_script(self, script: str) -> None:
        import urllib.parse
        marker = "ZR_JENKINS_BOOTSTRAP_OK"
        data = urllib.parse.urlencode({"script": script + "; println('" + marker + "')"}).encode()

        def send(headers):
            try:
                return self.manager._request("/scriptText", method="POST", data=data,
                                             headers=headers, timeout=60)
            except JenkinsError as error:
                if error.code != "controller_unreachable":
                    raise
                # The script may already have run. A second POST cannot prove it.
                raise JenkinsError("bootstrap_unknown", "Controller settings POST outcome requires read-only reconciliation") from error

        headers = {"Content-Type": "application/x-www-form-urlencoded"}
        status, body = send(headers)
        if status == 403:
            crumb_status, crumb_body = self.manager._request("crumbIssuer/api/json", timeout=20)
            if crumb_status != 200:
                raise JenkinsError("bootstrap_failed", "Jenkins rejected the authenticated crumb read")
            crumb = json.loads(crumb_body)
            status, body = send({**headers, crumb["crumbRequestField"]: crumb["crumb"]})
        if status not in (200, 201, 302) or marker.encode() not in body:
            raise JenkinsError("bootstrap_failed", "Jenkins script bootstrap did not return its verification marker")

    @staticmethod
    def _node_config_matches(name: str, desired: bytes, observed: bytes) -> bool:
        """Compare complete owned semantics, including known remoting defaults."""
        try:
            expected, actual = _parse_node_config(desired), _parse_node_config(observed)
            return expected["name"] == actual["name"] == name and expected == actual
        except (ET.ParseError, TypeError, ValueError):
            return False

    def _post_node(self, name: str, xml: str) -> None:
        import urllib.parse
        config_path = "/computer/" + urllib.parse.quote(name, safe="") + "/config.xml"
        data = xml.encode("utf-8")

        def post_once(path):
            try:
                status, _ = self.manager._request(path, method="POST", data=data,
                    headers={"Content-Type": "application/xml"}, timeout=60)
                return status
            except JenkinsError as error:
                if error.code != "controller_unreachable":
                    raise
                # Both an update and a create may have committed before timeout.
                # Read exactly this owned node once; never issue another POST.
                try:
                    status, observed = self.manager._request(config_path, timeout=20)
                    matches = status == 200 and self._node_config_matches(name, data, observed)
                except JenkinsError:
                    matches = False
                if matches:
                    return None
                raise JenkinsError("agent_bootstrap_unknown", "Agent configuration POST outcome remains unproven",
                                   details={"endpoint": config_path}) from error

        path = config_path
        status = post_once(path)
        if status is None:
            return
        if status == 404:
            # Only an explicit known 404 permits the established XML create route.
            path = "/computer/createItem?name=" + urllib.parse.quote(name, safe="")
            status = post_once(path)
            if status is None:
                return
        if status not in (200, 201, 302):
            raise JenkinsError("agent_bootstrap_failed", "Jenkins rejected agent configuration",
                               details={"httpStatus": status, "endpoint": path})

    def _post_xml(self, path: str, body: str, *, content_type: str = "application/xml") -> None:
        import base64
        import urllib.request
        user, token = self.manager._credentials()
        req = urllib.request.Request(self.manager.base_url + path, data=body.encode(), method="POST",
                                     headers={"Content-Type": content_type, "Authorization":
                                              "Basic " + base64.b64encode(f"{user}:{token}".encode()).decode()})
        try:
            with urllib.request.urlopen(req, timeout=10) as response:
                if response.status not in (200, 201, 302):
                    raise JenkinsError("bootstrap_failed", f"Jenkins rejected {path}")
        except Exception as exc:
            if isinstance(exc, JenkinsError):
                raise
            # CSRF protection can require a crumb.  Retry once with the
            # authenticated crumb endpoint, without exposing credentials.
            try:
                _, crumb_body = self.manager._request("crumbIssuer/api/json")
                crumb = json.loads(crumb_body)
                req.add_header(crumb["crumbRequestField"], crumb["crumb"])
                with urllib.request.urlopen(req, timeout=10) as response:
                    if response.status not in (200, 201, 302):
                        raise JenkinsError("bootstrap_failed", f"Jenkins rejected {path}")
            except Exception as retry_exc:
                raise JenkinsError("bootstrap_failed", "authenticated Jenkins bootstrap failed") from retry_exc
