// BEGIN ZIRCON SCOPED CHECKOUT COMMIT
if (params.ACTION == 'commit-checkout') {
    for (String name : ['SESSION_ID', 'REQUEST_ID', 'OPERATION_ID']) {
        if (!(params[name] ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/)) error("Invalid ${name}")
    }
    def commitResult
    for (int poll = 0; poll < 120; poll++) {
      node('zircon-windows-agent') {
        if (env.NODE_NAME != 'zircon-windows-agent') error('Unexpected commit node')
        dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-management') {
            def base = "E:/Git/ZirconEngine/.jenkins/state/controls/checkout-commit-${params.OPERATION_ID}.${env.BUILD_NUMBER}"
            writeFile(file: base + '.json', text: params.INPUT_JSON, encoding: 'UTF-8')
            def status
            withEnv(['INPUT_JSON=', "ZR_COMMIT_SESSION=${params.SESSION_ID}",
                     "ZR_COMMIT_REQUEST=${params.REQUEST_ID}", "ZR_COMMIT_OPERATION=${params.OPERATION_ID}",
                     "ZR_COMMIT_QUERY=${params.QUERY_ONLY}"]) {
                status = bat(returnStatus: true, script:
                    '@echo off\r\n"E:/Git/ZirconEngine/.jenkins/runtime/python/python.exe" -B -X utf8 ' +
                    '"E:/Git/ZirconEngine/tools/jenkins-management/checkout_commit.py" run ' +
                    '--input-file "' + base + '.json" --output-file "' + base + '.response.json"\r\nexit /b %ERRORLEVEL%')
            }
            if (!fileExists(base + '.response.json')) error('Commit runner produced no receipt')
            def response = readFile(file: base + '.response.json', encoding: 'UTF-8')
            commitResult = parseJson(response)
            def receipt = "checkout-commit-${params.OPERATION_ID}.${env.BUILD_NUMBER}.receipt.json"
            writeFile(file: receipt, text: response, encoding: 'UTF-8')
            archiveArtifacts(artifacts: receipt, fingerprint: false)
            currentBuild.description = "commit-checkout ${commitResult.status}: ${commitResult.commit ?: params.REQUEST_ID}"
            if (status != 0 && commitResult.status != 'waiting') error('Scoped commit failed; inspect the archived receipt')
        }
      }
      if (commitResult.status != 'waiting') break
      if (poll == 119) error('Commit is still waiting for an active Git/source owner; query the same request')
      sleep(time: 5, unit: 'SECONDS')
    }
    return
}
// END ZIRCON SCOPED CHECKOUT COMMIT
