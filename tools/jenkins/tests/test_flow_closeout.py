import tempfile, unittest
from pathlib import Path
from tools.jenkins.state import State
from tools.jenkins.contracts import JenkinsError
from tools.jenkins.notifications import NotificationOutbox
from tools.jenkins.workflow.handler import handle

class FlowCloseoutTests(unittest.TestCase):
    def setUp(self):
        temp=tempfile.TemporaryDirectory(); self.addCleanup(temp.cleanup)
        self.root=Path(temp.name); self.state=State(self.root/"support.sqlite3")
        self.identity=dict(repositoryId="repo",sessionId="s",requestId="r",attemptId="a",generation=1,
            sourceInputDigest="a"*64,coverageDigest="b"*64,stageImplementationDigest="c"*64)
        self.key="s:r:a"; self.event="s:r:a:complete"
        self.flow=dict(identity=self.identity,repositoryId="repo",status="accepted",executionId="e",
            ownedPaths=["src"],template="module_unit",acceptance={"status":"accepted"},
            stages=[{"name":"acceptance","status":"passed"},{"name":"commit","status":"pending"},
                    {"name":"notify","status":"pending"}])
        self.state.put("workflow",self.key,self.flow)
    def authorize(self, actions):
        self.state.authorize_session("repo","s","user",["src"],actions,{"source":"user","task":"mock closeout"})
    def finalize(self, **extra):
        return handle("finalize-flow",dict(identity=self.identity,**extra),self.state,self.root)
    def notify(self):
        return next(s for s in self.state.get("workflow",self.key)["payload"]["stages"] if s["name"]=="notify")
    def test_commit_intent_does_not_infer_notification(self):
        self.authorize(["commit","notify"])
        self.finalize(commitRequested=True)
        self.assertEqual([],self.state.list("notification_outbox"))
        self.assertEqual("skipped",self.notify()["status"])
    def test_ungranted_notification_does_not_queue(self):
        self.authorize(["validation"])
        result=self.finalize(notifyRequested=True,notifyDestination="fixture")
        self.assertFalse(result["notifyEligible"])
        self.assertEqual("skipped",self.notify()["status"])
        self.assertEqual([],self.state.list("notification_outbox"))
    def test_notify_only_authority_queues_before_delivery(self):
        self.authorize(["notify"])
        result=self.finalize(notifyRequested=True,notifyDestination="fixture")
        self.assertTrue(result["notifyEligible"])
        self.assertEqual("pending",NotificationOutbox(self.state).get(self.event,"fixture").status)
        stages=self.state.get("workflow",self.key)["payload"]["stages"]
        self.assertEqual("action_not_requested",next(s for s in stages if s["name"]=="commit")["reasonCode"])
    def test_comments_template_cannot_enqueue(self):
        self.authorize(["notify"])
        self.state.put("workflow",self.key,{**self.flow,"template":"comments_only"})
        self.finalize(notifyRequested=True,notifyDestination="fixture")
        self.assertEqual([],self.state.list("notification_outbox"))
    def test_missing_notify_stage_cannot_enqueue(self):
        self.authorize(["notify"])
        self.state.put("workflow",self.key,{**self.flow,"stages":[self.flow["stages"][0]]})
        self.finalize(notifyRequested=True,notifyDestination="fixture")
        self.assertEqual([],self.state.list("notification_outbox"))
    def test_explicit_destination_required_before_queue(self):
        self.authorize(["notify"])
        with self.assertRaises(JenkinsError):
            self.finalize(notifyRequested=True)
        self.assertEqual([],self.state.list("notification_outbox"))
    def test_delivered_notification_is_projected_without_resending(self):
        self.authorize(["notify"])
        self.finalize(notifyRequested=True,notifyDestination="fixture")
        outbox=NotificationOutbox(self.state)
        outbox.deliver(self.event,"fixture",lambda payload: True)
        result=self.finalize(notifyRequested=True,notifyDestination="fixture")
        self.assertFalse(result["notifyEligible"]); self.assertEqual("passed",self.notify()["status"])
        self.assertEqual(1,outbox.get(self.event,"fixture").attempts)
    def test_ambiguous_notification_is_not_deliverable_until_reconciled(self):
        self.authorize(["notify"])
        self.finalize(notifyRequested=True,notifyDestination="fixture")
        def fail(payload): raise OSError("ambiguous mock")
        outbox=NotificationOutbox(self.state); outbox.deliver(self.event,"fixture",fail)
        result=self.finalize(notifyRequested=True,notifyDestination="fixture")
        self.assertFalse(result["notifyEligible"]); self.assertEqual("unknown",outbox.get(self.event,"fixture").status)
    def test_job_declares_notify_parameters_and_flow_finalizes_first(self):
        import xml.etree.ElementTree as ET
        from tools.jenkins.deployment.job_provisioning import _config
        env={key:"fixture" for key in ["ZIRCON_AGENT_LABEL","ZIRCON_RUNTIME_OPERATION_ID",
            "ZIRCON_DRIVER_DIGEST","ZIRCON_REPO_ROOT","JENKINS_PYTHON","ZIRCON_SEALED_DRIVER",
            "ZIRCON_DRIVER_LAUNCHER","ZIRCON_BUILD_ROOT"]}
        xml=ET.fromstring(_config("fixture",env))
        names={item.text for item in xml.findall(".//parameterDefinitions/*/name")}
        self.assertTrue({"ALLOW_NOTIFY","NOTIFY_DESTINATION"}<=names)
        source=(Path(__file__).absolute().parents[3]/".jenkins/pipeline/zircon-flow.groovy").read_text()
        self.assertLess(source.index("stage('Finalize authorized actions')"),source.index("stage('Notify')"))
        self.assertIn("finalized.notifyEligible == true",source)
        self.assertIn("repositoryId: identity.repositoryId, sessionId: identity.sessionId",source)
