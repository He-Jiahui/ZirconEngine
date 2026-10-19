import groovy.json.JsonOutput
import groovy.json.JsonSlurper
import com.cloudbees.groovy.cps.NonCPS

@NonCPS
def plainJson(value) {
    if (value instanceof Map) {
        def result = [:]
        value.each { key, item -> result[key.toString()] = plainJson(item) }
        return result
    }
    if (value instanceof List) return value.collect { plainJson(it) }
    if (value instanceof CharSequence) return value.toString()
    return value
}

@NonCPS
def parseJson(String text) {
    return plainJson(new JsonSlurper().parseText(text))
}

def agentLabel = '@@AGENT@@'
def repository = '@@REPOSITORY@@'
def python = '@@PYTHON@@'
def codex = '@@CODEX@@'
def capsule = '@@CAPSULE@@'
def driver = '@@DRIVER@@'
def runtime = '@@RUNTIME@@'
def allowed = ['create-branch', 'create-worktree', 'prepare-merge', 'apply-patch', 'commit-worktree', 'release-worktree', 'recover-worktree', 'validate', 'generate', 'prepare-flow', 'submit-flow', 'prepare-worktree', 'inspect-worktree', 'retire-worktree-reader', 'preview-current-editor-r34']
if (!(params.ACTION in allowed)) error('Unsupported management action')
for (String name : ['SESSION_ID', 'REQUEST_ID', 'OPERATION_ID']) {
    if (!(params[name] ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/)) error("Invalid ${name}")
}
if (params.EXPECTED_DRIVER_DIGEST != driver || params.RUNTIME_OPERATION_ID != runtime) {
    error('Management request differs from the pinned formal runtime')
}
def input = parseJson(params.INPUT_JSON)
if (!(input instanceof Map)) error('INPUT_JSON must be an object')
def authoritative = [sessionId: params.SESSION_ID, requestId: params.REQUEST_ID,
    operationId: params.OPERATION_ID, action: params.ACTION, driverDigest: driver,
    runtimeOperationId: runtime, observationOnly: params.QUERY_ONLY]
authoritative.each { key, value ->
    if (input.containsKey(key) && input[key] != value) error("Cannot replace ${key}")
    input[key] = value
}
def result
for (int poll = 0; poll < 240; poll++) {
    node(agentLabel) {
        dir(repository + '/.jenkins/agent/workspace/zircon-management') {
            def inputPath = "${repository}/.jenkins/state/controls/management-${params.OPERATION_ID}.${env.BUILD_NUMBER}.${poll}.json"
            def outputPath = "${repository}/.jenkins/state/controls/management-${params.OPERATION_ID}.${env.BUILD_NUMBER}.${poll}.response.json"
            writeFile(file: inputPath, text: JsonOutput.toJson(input), encoding: 'UTF-8')
            def status
            // The exact request lives in the control file, not the child environment.
            withEnv(['INPUT_JSON=', 'XDG_CONFIG_HOME=E:/Git/ZirconEngine/.jenkins']) {
                status = bat(returnStatus: true, script:
                    "\"${python}\" -B -X utf8 \"${capsule}/runner.py\" --capsule \"${capsule}\" " +
                    "--input-file \"${inputPath}\" --output-file \"${outputPath}\"")
            }
            if (!fileExists(outputPath)) error("Pinned management runner rejected the control; exit ${status}")
            def receiptName = "management-${params.OPERATION_ID}.${env.BUILD_NUMBER}.receipt.json"
            // Keep complete receipt bytes on the agent; only this bounded projection enters CPS.
            def receiptProgram =
                "import hashlib,json,sys; from pathlib import Path; " +
                "raw=Path(sys.argv[1]).read_bytes(); " +
                "Path(sys.argv[2]).write_bytes(raw); Path(sys.argv[3]).write_bytes(raw); " +
                "value=json.loads(raw.decode('utf-8-sig')); " +
                "assert isinstance(value,dict),'Management receipt must be an object'; " +
                "keys=('status','reasonCode','operationKey','sessionId','requestId','operationId','worktreeId','worktreePreparationRef','currentSourceRef'); " +
                "summary={key:value[key][:256] for key in keys if isinstance(value.get(key),str)}; " +
                "summary['message']=value['message'][:1024] if isinstance(value.get('message'),str) else ''; " +
                "summary['receipt']={'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'controlPath':sys.argv[2],'archive':sys.argv[3]}; " +
                "print(json.dumps(summary,ensure_ascii=True,separators=(',',':')))"
            def receiptProjection
            try {
                receiptProjection = bat(returnStdout: true, encoding: 'UTF-8', script:
                    "@\"${python}\" -B -X utf8 -c \"${receiptProgram}\" " +
                    "\"${outputPath}\" \"${repository}/.jenkins/state/controls/${receiptName}\" \"${receiptName}\"").trim()
            } finally {
                archiveArtifacts(artifacts: receiptName, fingerprint: true)
            }
            result = parseJson(receiptProjection)
            currentBuild.description = "${params.ACTION}: ${params.REQUEST_ID}"
            if (status != 0 && result.status != 'waiting') error("Management operation requires review: ${result}")
        }
    }
    if (result.status != 'waiting') break
    if (poll == 239) error('Resource admission remains pending; query this same request')
    sleep(time: 5, unit: 'SECONDS')
}
if (result.status in ['unknown', 'failed', 'conflict']) error("Management operation retained: ${result}")
echo(JsonOutput.toJson(result))
