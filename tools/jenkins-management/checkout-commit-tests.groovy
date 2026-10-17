// BEGIN ZIRCON SCOPED COMMIT TESTS
if (params.RUN_ID != null && params.RUN_ID.toString().startsWith('checkout-commit-')) {
    def commitRun = params.RUN_ID.toString()
    def commitSuite = commitRun.contains('-cache-') ? 'cache' : (commitRun.contains('-support-') ? 'support' : (commitRun.contains('-delivery-') ? 'delivery' : 'all'))
    if (!(commitRun ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/)) error('Invalid commit regression run ID')
    node('zircon-windows-agent') {
        dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-management-tests') {
            def status = bat(returnStatus: true, script:
                '@echo off\r\n"E:/Git/ZirconEngine/.jenkins/runtime/python/python.exe" -B -X utf8 ' +
                '"E:/Git/ZirconEngine/tools/jenkins-management/run_commit_checks.py" --run-id "' + commitRun +
                '" --suite ' + commitSuite + '\r\nexit /b %ERRORLEVEL%')
            def output = 'E:/cargo-targets/zircon-local/reports/jenkins-commit-audit-20261017/tests/' + commitRun
            for (String name : ['artifacts.json', 'summary.json', 'stdout.txt', 'stderr.txt']) {
                if (fileExists(output + '/' + name)) {
                    writeFile(file: name, text: readFile(file: output + '/' + name, encoding: 'UTF-8'), encoding: 'UTF-8')
                    archiveArtifacts(artifacts: name, fingerprint: false)
                }
            }
            currentBuild.description = 'scoped checkout commit regression ' + commitRun
            if (status != 0) error('Scoped commit regression failed; inspect archived results')
        }
    }
    return
}
// END ZIRCON SCOPED COMMIT TESTS
