"""Control tests for proposed bootstrap bytes; no HTTP or service execution."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import sys
from types import SimpleNamespace
import unittest

ROOT = Path(__file__).absolute().parents[3]
sys.path.insert(0, str(ROOT))
from tools.jenkins.contracts import JenkinsError

from tools.jenkins.deployment import bootstrap as DRAFT

NAME = "zircon-runtime-actual"
XML = ("<hudson.slaves.DumbSlave><name>" + NAME + "</name><nodeDescription>Zircon Jenkins runtime agent</nodeDescription>"
       "<remoteFS>E:\\Git\\ZirconEngine\\.jenkins\\agent</remoteFS><numExecutors>2</numExecutors>"
       "<label>owned-runtime-label</label><mode>EXCLUSIVE</mode>"
       "<retentionStrategy class='hudson.slaves.RetentionStrategy$Always'/>"
       "<launcher class='hudson.slaves.JNLPLauncher'><webSocket>true</webSocket></launcher></hudson.slaves.DumbSlave>")


class Endpoint:
    def __init__(self, replies):
        self.replies = list(replies)
        self.calls = []

    def _request(self, path, **kwargs):
        self.calls.append((path, kwargs))
        if not self.replies:
            raise AssertionError("Unexpected retry")
        reply = self.replies.pop(0)
        if isinstance(reply, Exception):
            raise reply
        return reply


def timeout():
    return JenkinsError("controller_unreachable", "transport timeout")


class BootstrapTimeoutControls(unittest.TestCase):
    def bootstrap(self, replies):
        endpoint = Endpoint(replies)
        return DRAFT.JenkinsBootstrap(SimpleNamespace(), SimpleNamespace(), endpoint), endpoint

    def test_success_has_explicit_bounded_maintenance_timeout(self):
        bootstrap, endpoint = self.bootstrap([(200, b"")])
        bootstrap._post_node(NAME, XML)
        self.assertEqual(60, endpoint.calls[0][1]["timeout"])
        self.assertEqual(1, len(endpoint.calls))

    def test_update_timeout_reconciles_complete_owned_configuration_without_retry(self):
        actual = XML.replace("hudson.slaves.DumbSlave", "slave").replace("><", ">\n  <").encode()
        bootstrap, endpoint = self.bootstrap([timeout(), (200, actual)])
        bootstrap._post_node(NAME, XML)
        self.assertEqual(["POST", "GET"], [call[1].get("method", "GET") for call in endpoint.calls])
        self.assertEqual(endpoint.calls[0][0], endpoint.calls[1][0])
        self.assertEqual(20, endpoint.calls[1][1]["timeout"])

    def test_create_timeout_only_reads_exact_node_after_known_absence(self):
        bootstrap, endpoint = self.bootstrap([(404, b""), timeout(), (200, XML.encode())])
        bootstrap._post_node(NAME, XML)
        self.assertEqual(["POST", "POST", "GET"], [call[1].get("method", "GET") for call in endpoint.calls])
        self.assertEqual("/computer/createItem?name=" + NAME, endpoint.calls[1][0])
        self.assertEqual(endpoint.calls[0][0], endpoint.calls[2][0])

    def test_unknown_configuration_and_failed_reads_never_retry_or_create(self):
        cases = [(404, b""), (403, XML.encode()), timeout(), (200, b"<slave/>")]
        for reply in cases:
            with self.subTest(reply=type(reply).__name__):
                bootstrap, endpoint = self.bootstrap([timeout(), reply])
                with self.assertRaises(JenkinsError) as rejected:
                    bootstrap._post_node(NAME, XML)
                self.assertEqual("agent_bootstrap_unknown", rejected.exception.code)
                self.assertEqual(2, len(endpoint.calls))
                self.assertEqual(1, sum(call[1].get("method") == "POST" for call in endpoint.calls))

    def test_foreign_and_incomplete_semantics_remain_unknown(self):
        cases = [XML.replace(NAME, "foreign-node"), XML.replace("EXCLUSIVE", "NORMAL"),
                 XML.replace("owned-runtime-label", "foreign-label"), XML.replace("numExecutors>2", "numExecutors>1"), XML.replace("numExecutors>2", "numExecutors>3"),
                 XML.replace(".jenkins\\agent", ".jenkins\\foreign"), XML.replace("<webSocket>true", "<webSocket>false"),
                 XML.replace("</launcher>", "<tunnel>foreign:123</tunnel></launcher>"),
                 XML.replace("</hudson.slaves.DumbSlave>", "<nodeProperties><foreign/></nodeProperties></hudson.slaves.DumbSlave>"),
                 XML.replace("<mode>EXCLUSIVE</mode>", ""), XML.replace("<label>", "<label duplicate='true'>"),
                 XML.replace("<remoteFS>E:", "<remoteFS> E:"), XML.replace("<name>", "<name>extra</name><name>"),
                 '<!DOCTYPE slave [<!ENTITY a "x">]>' + XML]
        for actual in cases:
            with self.subTest(actual=actual[:60]):
                self.assertFalse(DRAFT.JenkinsBootstrap._node_config_matches(NAME, XML.encode(), actual.encode()))

    def test_known_rejection_does_not_become_unknown_or_retry(self):
        bootstrap, endpoint = self.bootstrap([(403, b"")])
        with self.assertRaises(JenkinsError) as rejected:
            bootstrap._post_node(NAME, XML)
        self.assertEqual("agent_bootstrap_failed", rejected.exception.code)
        self.assertEqual(1, len(endpoint.calls))

    def test_script_timeout_has_one_post_no_crumb_or_automatic_retry(self):
        bootstrap, endpoint = self.bootstrap([timeout()])
        with self.assertRaises(JenkinsError) as rejected:
            bootstrap._post_script("owned settings")
        self.assertEqual("bootstrap_unknown", rejected.exception.code)
        self.assertEqual(1, len(endpoint.calls))
        self.assertEqual(60, endpoint.calls[0][1]["timeout"])

    def test_script_known_csrf_rejection_retains_verified_crumb_retry(self):
        bootstrap, endpoint = self.bootstrap([(403, b""), (200, b'{"crumbRequestField":"X-Crumb","crumb":"value"}'),
                                               (200, b"ZR_JENKINS_BOOTSTRAP_OK")])
        bootstrap._post_script("owned settings")
        self.assertEqual(3, len(endpoint.calls))
        self.assertEqual([60, 20, 60], [kwargs["timeout"] for _, kwargs in endpoint.calls])


if __name__ == "__main__":
    unittest.main()
