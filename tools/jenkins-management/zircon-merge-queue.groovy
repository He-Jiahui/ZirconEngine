// ZIRCON SCHEDULED BRANCH INTEGRATION
node('zircon-windows-agent') {
    dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-merge-queue') {
        def output = "E:/Git/ZirconEngine/.jenkins/state/controls/scheduled-integration-${env.BUILD_NUMBER}.json"
        int status
        withEnv(['ZR_INTEGRATION_DRIVER=@@driverDigest@@', 'ZR_INTEGRATION_RUNTIME=@@runtimeOperationId@@']) {
            status = bat(returnStatus: true, script:
                '@echo off\r\n"E:/Git/ZirconEngine/.jenkins/runtime/python/python.exe" -B -X utf8 ' +
                '"E:/Git/ZirconEngine/tools/jenkins-management/branch_integration.py" drain --output-file "' + output + '"\r\nexit /b %ERRORLEVEL%')
        }
        if (!fileExists(output)) error('Integration has no receipt; reconcile its original native result')
        def response = readFile(file: output, encoding: 'UTF-8')
        writeFile(file: 'integration-result.json', text: response, encoding: 'UTF-8')
        archiveArtifacts(artifacts: 'integration-result.json', fingerprint: false)
        currentBuild.description = 'Scheduled branch integration; inspect exact per-task results'
        if (status != 0) error('Some branch integrations remain pending; task commits are retained')
    }
}
