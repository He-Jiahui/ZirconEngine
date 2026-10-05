// Sealed workflow glue.  Source payloads are data; only this fixed file is loaded.
def control(String domain, String action, Map payload) {
    def body = groovy.json.JsonOutput.toJson(payload)
    def nonce = "control-${env.BUILD_TAG}-${UUID.randomUUID().toString()}"
    def input = "${env.ZIRCON_REPO_ROOT}/.jenkins/state/controls/${nonce}.json"
    def output = "${env.ZIRCON_REPO_ROOT}/.jenkins/state/responses/${nonce}.json"
    writeFile(file: input, text: body)
    bat(label: "Jenkins control ${domain}/${action}", script: """@echo off
"%JENKINS_PYTHON%" -B "%ZIRCON_DRIVER_LAUNCHER%" --repository "%ZIRCON_REPO_ROOT%" ${domain} ${action} --input-file "${input}" --output-file "${output}"
""")
    def value = parseControlResult(readFile(output))
    if (!value.status || value.status in ['failed', 'blocked']) {
        error("authoritative control operation ${action} failed: ${value}")
    }
    return value
}

// The controller's script-security whitelist permits JsonSlurper.parseText.
// Convert the small response to ordinary strings before returning across CPS;
// never expose Groovy's LazyMap to a suspended Pipeline continuation.
@NonCPS
def parseControlResult(String raw) {
    def parsed = new groovy.json.JsonSlurper().parseText(raw.trim())
    return [status: parsed.status == null ? null : parsed.status.toString(),
            reasonCode: parsed.reasonCode == null ? null : parsed.reasonCode.toString(),
            executionId: parsed.executionId == null ? null : parsed.executionId.toString(),
            recipeRef: parsed.recipeRef == null ? null : parsed.recipeRef.toString(),
            template: parsed.template == null ? null : parsed.template.toString(),
            receiptRef: parsed.receiptRef == null ? null : parsed.receiptRef.toString()]
}

def withControl(Closure body) {
    if (!env.ZIRCON_AGENT_LABEL || !env.ZIRCON_RUNTIME_OPERATION_ID) error('runtime agent fence is required')
    if (env.RUNTIME_OPERATION_ID && env.RUNTIME_OPERATION_ID != env.ZIRCON_RUNTIME_OPERATION_ID) error('runtime operation mismatch')
    node(env.ZIRCON_AGENT_LABEL) {
        if (!env.ZIRCON_SEALED_DRIVER || !env.ZIRCON_DRIVER_DIGEST) error('sealed driver environment is required')
        body(load(env.ZIRCON_SEALED_DRIVER))
    }
}

def waitForControl(String domain, String action, Map payload, int maxPolls = 120) {
    for (int attempt = 0; attempt < maxPolls; attempt++) {
        def result = control(domain, action, payload)
        if (result.status in ['passed', 'reused', 'complete', 'accepted', 'ready', 'cancelled']) return result
        if (result.status == 'failed' || result.status == 'blocked') error("control operation ${action} blocked")
        sleep(time: Math.min(30, 2 + attempt / 4), unit: 'SECONDS')
    }
    error("control operation ${action} exceeded polling budget")
}

return this
