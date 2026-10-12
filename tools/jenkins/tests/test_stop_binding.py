import base64,hashlib,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.stop_binding import bind_owned_stop,capture_stop_owner

class StopBindingTests(unittest.TestCase):
    def setUp(self):
        temporary=tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.state=Path(temporary.name)
        self.controller=dict(pid=20,creationTime="200",executable="java",commandDigest="a"*64)
        self.operation=dict(state="running",operationId="start-one",generation="start-one",
            pid=10,hostPid=10,creationTime="100",executable="python",commandDigest="b"*64,
            homePath=str(self.state/"home"),controlPlaneOnly=True)
        self.host=dict(status="running",operationId="start-one",generation="start-one",
            hostPid=10,identity=self.controller,controllerIdentity=self.controller,
            agentIdentity=None,agentLaunchAttempted=False,controlPlaneOnly=True)
        self.proof=dict(identity=self.controller,complete=True,childrenGone=True,activeProcesses=0,
            stdoutEof=True,stderrEof=True,processExitCode=1)
        self.result=dict(status="stopped",operationId="start-one",generation="start-one",
            identity=self.controller,nativeTerminationProof=self.proof,agentTerminationProof=None)
        self.command=self.state/"deployment/commands/stop.json"
        self.command.parent.mkdir(parents=True)
        self.command.write_text(json.dumps(dict(action="stop",operationId="start-one",
            generation="start-one",hostPid=10,controllerIdentity=self.controller)),encoding="utf-8")
        self.response=self.command.with_suffix(".result.json")
        self.host_path=self.state/"deployment/host.json"
        self.journal=self.state/"deployment-operation.json"
        self.journal.write_text(json.dumps(self.operation),encoding="utf-8")
        self.captured=dict(operation=self.operation,host=self.host,homePath=self.operation["homePath"],liveIdentities=[])
        self.departure=dict(status="absent",pid=10,expectedCreationTime="100")
    def bind(self):
        raw=(json.dumps(self.result,indent=3)+"\r\n").encode()
        self.response.write_bytes(raw)
        self.host_path.write_bytes(raw)
        with patch("tools.jenkins.deployment.stop_binding.observe_departed_identity",return_value=self.departure):
            value=bind_owned_stop(self.state,self.captured,self.command,self.response)
        return value,raw
    def test_legacy_binding_preserves_exact_native_and_raw_proof(self):
        value,raw=self.bind()
        projected=json.loads(self.host_path.read_bytes())
        self.assertEqual(self.proof,projected["nativeTerminationProof"])
        self.assertEqual(self.controller,projected["controllerIdentity"])
        self.assertEqual(10,projected["hostPid"])
        self.assertIs(False,projected["agentLaunchAttempted"])
        saved=json.loads(Path(value["stopBinding"]["path"]).read_bytes())
        self.assertEqual(raw,base64.b64decode(saved["rawStopResult"]["base64"]))
        self.response.write_text('{"status":"ignored"}',encoding="utf-8")
        self.assertEqual(raw,Path(value["stopResult"]["path"]).read_bytes())
        self.assertEqual(hashlib.sha256(raw).hexdigest(),value["stopResult"]["sha256"])
    def test_conflicting_metadata_is_rejected(self):
        for field,value in (("hostPid",99),("generation","foreign"),("agentLaunchAttempted",True),
                            ("controllerIdentity",{**self.controller,"creationTime":"500"})):
            with self.subTest(field=field):
                previous=self.result.get(field)
                self.result[field]=value
                with self.assertRaises(JenkinsError): self.bind()
                self.assertEqual(self.result,json.loads(self.host_path.read_bytes()))
                if previous is None: del self.result[field]
                else: self.result[field]=previous
    def test_incomplete_native_tree_or_eof_is_rejected(self):
        for field,value in (("complete",False),("childrenGone",False),("activeProcesses",1),
                            ("stdoutEof",False),("stderrEof",False)):
            self.result["nativeTerminationProof"]={**self.proof,field:value}
            with self.subTest(field=field),self.assertRaises(JenkinsError): self.bind()
    def test_pid_only_departure_is_rejected(self):
        self.departure=dict(status="absent",pid=10)
        with self.assertRaises(JenkinsError): self.bind()
    def test_exact_pid_reuse_observation_is_accepted(self):
        self.departure=dict(status="pid-reused",pid=10,expectedCreationTime="100",
            observedIdentity=dict(pid=10,creationTime="500"))
        self.assertEqual("stopped",self.bind()[0]["state"])
    def test_concurrent_journal_is_preserved(self):
        changed={**self.operation,"operationId":"newer"}
        self.journal.write_text(json.dumps(changed),encoding="utf-8")
        with self.assertRaises(JenkinsError): self.bind()
        self.assertEqual(changed,json.loads(self.journal.read_bytes()))
    def test_inaccessible_owner_is_rejected_before_stop(self):
        with patch("tools.jenkins.deployment.stop_binding.observe_live_identity",side_effect=JenkinsError("inaccessible","foreign")):
            with self.assertRaises(JenkinsError):
                capture_stop_owner(self.operation,self.host,self.operation["homePath"])

    def test_repeated_stop_checks_terminal_proof_without_replaying_command(self):
        from types import SimpleNamespace
        from tools.jenkins.deployment.manager import DeploymentManager
        value, unused = self.bind()
        manager = object.__new__(DeploymentManager)
        manager.paths = SimpleNamespace(state=self.state, home=self.state/"home")
        manager._read = lambda: value
        manager._control_plane_drain = lambda *args: self.fail("Repeated stop drained a terminal owner")
        manager._write = lambda *args: self.fail("Repeated stop rewrote terminal evidence")
        with patch("tools.jenkins.deployment.startup_observations.observe_departed_identity",
                   return_value=self.departure):
            self.assertEqual(value, manager._stop_locked())
        self.assertEqual(1, len(list(self.command.parent.glob("stop.json"))))
        self.assertFalse(any(self.command.parent.glob("stop-*.json")))
