// BEGIN ZIRCON BRANCH QUEUE
if (params.ACTION in ['commit-branch', 'check-branch-queue']) {
    for (String name : ['SESSION_ID', 'REQUEST_ID', 'OPERATION_ID']) {
        if (!(params[name] ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/)) error("Invalid ${name}")
    }
    node('zircon-windows-agent') {
        dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-management') {
            def base = "E:/Git/ZirconEngine/.jenkins/state/controls/branch-${params.OPERATION_ID}.${env.BUILD_NUMBER}"
            writeFile(file: base + '.json', text: params.INPUT_JSON, encoding: 'UTF-8')
            int status
            withEnv(['INPUT_JSON=', "ZR_BRANCH_ACTION=${params.ACTION}",
                     "ZR_BRANCH_SESSIONID=${params.SESSION_ID}", "ZR_BRANCH_REQUESTID=${params.REQUEST_ID}",
                     "ZR_BRANCH_OPERATIONID=${params.OPERATION_ID}", "ZR_BRANCH_DRIVER=${params.EXPECTED_DRIVER_DIGEST}",
                     "ZR_BRANCH_RUNTIME=${params.RUNTIME_OPERATION_ID}"]) {
                status = bat(returnStatus: true, script:
                    '@echo off\r\n"E:/Git/ZirconEngine/.jenkins/runtime/python/python.exe" -B -X utf8 ' +
                    '"E:/Git/ZirconEngine/tools/jenkins-management/branch_integration.py" run ' +
                    '--input-file "' + base + '.json" --output-file "' + base + '.response.json"\r\nexit /b %ERRORLEVEL%')
            }
            if (!fileExists(base + '.response.json')) error('No branch receipt; reconcile this original request')
            writeFile(file: "branch-${params.OPERATION_ID}.${env.BUILD_NUMBER}.receipt.json",
                      text: readFile(file: base + '.response.json', encoding: 'UTF-8'), encoding: 'UTF-8')
            archiveArtifacts(artifacts: "branch-${params.OPERATION_ID}.${env.BUILD_NUMBER}.receipt.json", fingerprint: false)
            currentBuild.description = "${params.ACTION}: ${params.REQUEST_ID}; main integration uses zircon-merge-queue"
            if (status != 0) error('Scoped branch operation did not complete; inspect its retained receipt')
        }
    }
    return
}
// END ZIRCON BRANCH QUEUE
