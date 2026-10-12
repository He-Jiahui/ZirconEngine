def agentLabel = '@@AGENT@@'
def repository = '@@REPOSITORY@@'
def python = '@@PYTHON@@'
def codex = '@@CODEX@@'
def driver = '@@DRIVER@@'
def runtime = '@@RUNTIME@@'

if (agentLabel != 'zircon-windows-agent') error('Unexpected regression-test agent')
if (repository.replace('\\', '/').equalsIgnoreCase('E:/Git/ZirconEngine') == false) {
    error('Regression-test repository pin differs')
}
if (!(driver ==~ /[0-9a-f]{64}/) || !(runtime ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/)) {
    error('Regression-test runtime pins are malformed')
}
if (!python.replace('\\', '/').equalsIgnoreCase(repository.replace('\\', '/') + '/.jenkins/runtime/python/python.exe')) {
    error('Regression-test Python runtime path differs')
}
if (!codex || codex.contains('"') || codex.contains('\r') || codex.contains('\n')) {
    error('Official Codex sandbox executable path is invalid')
}

def suppliedRunId = params.RUN_ID == null ? '' : params.RUN_ID.toString().trim()
def runId = suppliedRunId ? suppliedRunId : "jenkins-${env.BUILD_NUMBER}"
if (!(runId ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/)) error('Invalid Jenkins run identifier')
def reportRoot = 'E:/cargo-targets/zircon-local/reports/jenkins-parallel-activation-20261011/ordinary-management-regression'
def reportPath = "${reportRoot}/${runId}"
def runner = "${repository}/tools/jenkins-management/run_checks.py"

node(agentLabel) {
    if (env.NODE_NAME != agentLabel) error('Regression requires the sole fixed node')
    dir("${repository}/.jenkins/agent/workspace/zircon-management-tests") {
        // The fixed agent inherits the verified official strict runtime Job.
        def command = "\"${python}\" -B -Xutf8 \"${runner}\" " +
            "--expected-driver \"${driver}\" --expected-runtime \"${runtime}\" " +
            "--run-id \"${runId}\" --repo-root \"${repository}\""
        def status
        withEnv([
            'USERPROFILE=C:/Users/HeJiahui',
            'LOCALAPPDATA=C:/Users/HeJiahui/AppData/Local',
            'APPDATA=C:/Users/HeJiahui/AppData/Roaming',
            "TEMP=${repository}/.jenkins/tmp",
            "TMP=${repository}/.jenkins/tmp",
            "TMPDIR=${repository}/.jenkins/tmp"
        ]) {
            status = bat(returnStatus: true, script: "@echo off\r\n${command}\r\nexit /b %ERRORLEVEL%")
        }

        if (fileExists("${reportPath}/summary.json")) {
            dir(reportPath) {
                archiveArtifacts artifacts: 'summary.json,pytest-junit.xml',
                    allowEmptyArchive: true, fingerprint: true
                junit testResults: 'pytest-junit.xml', allowEmptyResults: true
            }
        } else {
            echo("No regression receipt was produced for ${runId}")
        }
        currentBuild.description = "ordinary management regression ${runId}"
        if (status != 0) error("Ordinary management regression ended with status ${status}")
    }
}
