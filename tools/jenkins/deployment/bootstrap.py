from __future__ import annotations

import json
from pathlib import Path
from xml.sax.saxutils import escape

from .spec import JenkinsError


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
        node_xml = ("<hudson.slaves.DumbSlave>"
                    f"<name>{escape(str(node['name']))}</name><nodeDescription>Zircon Jenkins agent</nodeDescription>"
                    f"<remoteFS>{escape(str(self.paths.agent))}</remoteFS><numExecutors>{int(node.get('executors', 1))}</numExecutors>"
                    f"<label>{escape(str(node.get('label', 'zircon-windows')))}</label><mode>NORMAL</mode><retentionStrategy class='hudson.slaves.RetentionStrategy$Always'/>"
                    "<launcher class='hudson.slaves.JNLPLauncher'><webSocket>true</webSocket></launcher></hudson.slaves.DumbSlave>")
        self._post_node(node["name"], node_xml)
        return {"bootstrapApplied": True, "executors": 0, "agentConfigured": True, "authenticated": True}

    def _post_script(self, script: str) -> None:
        import urllib.parse
        marker = "ZR_JENKINS_BOOTSTRAP_OK"
        status, body = self.manager._request("/scriptText", method="POST",
            data=urllib.parse.urlencode({"script": script + "; println('" + marker + "')"}).encode(),
            headers={"Content-Type": "application/x-www-form-urlencoded"})
        if status == 403:
            _, crumb_body = self.manager._request("crumbIssuer/api/json")
            crumb = json.loads(crumb_body)
            status, body = self.manager._request("/scriptText", method="POST",
                data=urllib.parse.urlencode({"script": script + "; println('" + marker + "')"}).encode(),
                headers={"Content-Type": "application/x-www-form-urlencoded",
                         crumb["crumbRequestField"]: crumb["crumb"]})
        if status not in (200, 201, 302) or marker.encode() not in body:
            raise JenkinsError("bootstrap_failed", "Jenkins script bootstrap did not return its verification marker")

    def _post_node(self, name: str, xml: str) -> None:
        import urllib.parse
        path = "/computer/" + urllib.parse.quote(name, safe="") + "/config.xml"
        status, _ = self.manager._request(path, method="POST", data=xml.encode(), headers={"Content-Type": "application/xml"})
        if status == 404:
            path = "/computer/doCreateItem?name=" + urllib.parse.quote(name) + "&type=hudson.slaves.DumbSlave"
            status, _ = self.manager._request(path, method="POST", data=xml.encode(), headers={"Content-Type": "application/xml"})
        if status not in (200, 201, 302):
            raise JenkinsError("agent_bootstrap_failed", "Jenkins rejected agent configuration")

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
