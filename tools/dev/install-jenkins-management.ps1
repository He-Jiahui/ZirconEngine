# Retired public workflow. Ignore obsolete arguments; do not inspect host state.
[Console]::Out.WriteLine('{"commandExecuted": false, "executionOwner": "Agent", "formalAcceptance": false, "historicalDataChanged": false, "hostServicesChanged": false, "m8Accepted": false, "message": "This workflow is retired. Use native Git and the scoped local development tools. No command was executed; host services and historical data are unchanged.", "reasonCode": "jenkins_workflow_retired", "status": "retired"}')
exit 2
