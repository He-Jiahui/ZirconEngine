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
def allowed = ['create-branch', 'create-worktree', 'prepare-merge', 'apply-patch', 'commit-worktree', 'release-worktree', 'validate', 'generate', 'submit-flow']
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
            def status = bat(returnStatus: true, script:
                "\"${python}\" -B -X utf8 \"${capsule}/runner.py\" --capsule \"${capsule}\" " +
                "--input-file \"${inputPath}\" --output-file \"${outputPath}\"")
            if (!fileExists(outputPath)) error("Pinned management runner rejected the control; exit ${status}")
            result = parseJson(readFile(file: outputPath, encoding: 'UTF-8'))
            def receiptName = "management-${params.OPERATION_ID}.${env.BUILD_NUMBER}.receipt.json"
            writeFile(file: "${repository}/.jenkins/state/controls/${receiptName}", text: JsonOutput.toJson(result), encoding: 'UTF-8')
            writeFile(file: receiptName, text: JsonOutput.toJson(result), encoding: 'UTF-8')
            archiveArtifacts(artifacts: receiptName, fingerprint: true)
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
