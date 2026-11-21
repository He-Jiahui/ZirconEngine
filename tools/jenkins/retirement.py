"""Terminal, stateless notice for retired workflow commands.

Do not load deployment state, credentials, queues or service modules here.
Historical data and already-running host processes are deliberately untouched.
"""
from __future__ import annotations

import json

MESSAGE = (
    "This workflow is retired. Use native Git and the scoped local development tools. "
    "No command was executed; host services and historical data are unchanged."
)


def retired_cli() -> int:
    print(json.dumps({
        "status": "retired", "reasonCode": "jenkins_workflow_retired",
        "message": MESSAGE, "executionOwner": "Agent", "commandExecuted": False,
        "hostServicesChanged": False, "historicalDataChanged": False,
        "formalAcceptance": False, "m8Accepted": False,
    }, sort_keys=True))
    return 2
