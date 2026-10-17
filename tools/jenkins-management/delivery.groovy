// BEGIN ZIRCON VALIDATED DELIVERY
if (params.ACTION == 'deliver-task') {
    for (String name : ['SESSION_ID', 'REQUEST_ID', 'OPERATION_ID']) {
        if (!(params[name] ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/)) error("Invalid ${name}")
    }
    node('zircon-windows-agent') {
        dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-management') {
            def base = "E:/Git/ZirconEngine/.jenkins/state/controls/delivery-${params.OPERATION_ID}.${env.BUILD_NUMBER}"
            writeFile(file: base + '.json', text: params.INPUT_JSON, encoding: 'UTF-8')
            def status
            withEnv(['INPUT_JSON=', "ZR_DELIVERY_SESSION=${params.SESSION_ID}",
                     "ZR_DELIVERY_REQUEST=${params.REQUEST_ID}", "ZR_DELIVERY_OPERATION=${params.OPERATION_ID}",
                     "ZR_DELIVERY_DRIVER=${params.EXPECTED_DRIVER_DIGEST}",
                     "ZR_DELIVERY_RUNTIME=${params.RUNTIME_OPERATION_ID}"]) {
                status = bat(returnStatus: true, script:
                    '@echo off\r\n"E:/Git/ZirconEngine/.jenkins/runtime/python/python.exe" -B -X utf8 ' +
                    '"E:/Git/ZirconEngine/tools/jenkins-management/delivery.py" run ' +
                    '--input-file "' + base + '.json" --output-file "' + base + '.response.json"\r\nexit /b %ERRORLEVEL%')
            }
            if (!fileExists(base + '.response.json')) error('Delivery has no receipt; reconcile this operation')
            def response = readFile(file: base + '.response.json', encoding: 'UTF-8')
            def receipt = "delivery-${params.OPERATION_ID}.${env.BUILD_NUMBER}.receipt.json"
            writeFile(file: receipt, text: response, encoding: 'UTF-8')
            archiveArtifacts(artifacts: receipt, fingerprint: false)
            def result = parseJson(response)
            currentBuild.description = "deliver-task: ${params.REQUEST_ID} (${result.status})"
            if (status != 0) error('Delivery did not complete; inspect the archived receipt and continue the same operation')
        }
    }
    return
}
// END ZIRCON VALIDATED DELIVERY
