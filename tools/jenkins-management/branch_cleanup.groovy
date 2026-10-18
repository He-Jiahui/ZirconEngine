// BEGIN ZIRCON OLD BRANCH CLEANUP
if (params.ACTION == 'prune-old-branches') {
    if (!(params.OPERATION_ID ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/)) error('Invalid cleanup operation')
    timeout(time: 10, unit: 'MINUTES') {
        node('zircon-windows-agent') {
            dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-management') {
                def input = "E:/Git/ZirconEngine/.jenkins/state/controls/branch-cleanup-${params.OPERATION_ID}.${env.BUILD_NUMBER}.json"
                writeFile(file: input, text: params.INPUT_JSON, encoding: 'UTF-8')
                int status
                withEnv(['INPUT_JSON=', "ZR_BRANCH_CLEANUP_OPERATION=${params.OPERATION_ID}",
                         "ZR_BRANCH_CLEANUP_DRIVER=${params.EXPECTED_DRIVER_DIGEST}",
                         "ZR_BRANCH_CLEANUP_RUNTIME=${params.RUNTIME_OPERATION_ID}"]) {
                    status = bat(returnStatus: true, script:
                        '@echo off\r\n"E:/Git/ZirconEngine/.jenkins/runtime/python/python.exe" -B -X utf8 ' +
                        '"E:/Git/ZirconEngine/tools/jenkins-management/branch_cleanup.py" run --input-file "' + input + '"\r\nexit /b %ERRORLEVEL%')
                }
                for (String name : ['executed-plan.json', 'receipt.json']) {
                    def path = 'E:/cargo-targets/zircon-local/reports/branch-cleanup-before-20261016-20261018/' + name
                    if (fileExists(path)) {
                        writeFile(file: name, text: readFile(file: path, encoding: 'UTF-8'), encoding: 'UTF-8')
                        archiveArtifacts(artifacts: name, fingerprint: false)
                    }
                }
                currentBuild.description = 'Remove explicitly selected old merged local branches; preserve occupied worktrees'
                if (status != 0) error('Branch cleanup did not finish; reconcile the original receipt')
            }
        }
    }
    return
}
// END ZIRCON OLD BRANCH CLEANUP
