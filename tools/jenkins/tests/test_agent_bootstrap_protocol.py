"""Protect the distinct Jenkins XML and submitted-form node endpoints."""
from types import SimpleNamespace
import unittest

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.bootstrap import JenkinsBootstrap


class XmlNodeEndpoint:
    def __init__(self, *, exists=False, create_status=200):
        self.exists = exists
        self.create_status = create_status
        self.requests = []

    def _request(self, path, **kwargs):
        self.requests.append((path, kwargs))
        if path.endswith('/config.xml'):
            return (200 if self.exists else 404), b''
        if path.startswith('/computer/createItem?'):
            if kwargs.get('headers', {}).get('Content-Type') == 'application/xml':
                return self.create_status, b''
        # The submitted-form endpoint rejects an XML-only request.
        return 400, b'Expected a submitted form'


class AgentBootstrapProtocolTests(unittest.TestCase):
    def test_new_agent_uses_xml_creation_endpoint(self):
        endpoint = XmlNodeEndpoint()
        bootstrap = JenkinsBootstrap(SimpleNamespace(), SimpleNamespace(), endpoint)
        xml = '<hudson.slaves.DumbSlave><name>runtime agent</name></hudson.slaves.DumbSlave>'
        bootstrap._post_node('runtime agent', xml)
        self.assertEqual(len(endpoint.requests), 2)
        path, request = endpoint.requests[-1]
        self.assertEqual(path, '/computer/createItem?name=runtime%20agent')
        self.assertEqual(request['method'], 'POST')
        self.assertEqual(request['data'], xml.encode())

    def test_existing_agent_is_updated_without_creating_another_node(self):
        endpoint = XmlNodeEndpoint(exists=True)
        bootstrap = JenkinsBootstrap(SimpleNamespace(), SimpleNamespace(), endpoint)
        bootstrap._post_node('runtime agent', '<slave/>')
        self.assertEqual(len(endpoint.requests), 1)
        self.assertEqual(endpoint.requests[0][0], '/computer/runtime%20agent/config.xml')

    def test_rejected_creation_remains_a_bootstrap_failure(self):
        endpoint = XmlNodeEndpoint(create_status=403)
        bootstrap = JenkinsBootstrap(SimpleNamespace(), SimpleNamespace(), endpoint)
        with self.assertRaises(JenkinsError) as rejected:
            bootstrap._post_node('runtime-agent', '<slave/>')
        self.assertEqual(rejected.exception.code, 'agent_bootstrap_failed')
        self.assertEqual(rejected.exception.details['httpStatus'], 403)


if __name__ == '__main__':
    unittest.main()
