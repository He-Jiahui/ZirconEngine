// Sealed workflow glue.  Source payloads are data; only this fixed file is loaded.
def control(String domain, String action, Map payload) {
    def runtimeOperationId = env.ZIRCON_RUNTIME_OPERATION_ID
    def driverDigest = env.ZIRCON_DRIVER_DIGEST
    if (!runtimeOperationId || !driverDigest) error('sealed driver runtime binding is required')
    def suppliedRuntime = payload.get('runtimeOperationId')
    if (suppliedRuntime && suppliedRuntime != runtimeOperationId) error('runtime operation mismatch')
    def suppliedDriver = payload.get('driverDigest')
    if (suppliedDriver && suppliedDriver != driverDigest) error('driver digest mismatch')
    // Bind every control request to the sealed driver selected for this node.
    // Clone the caller payload so its nested workflow identity remains intact.
    def boundPayload = new LinkedHashMap(payload)
    boundPayload.put('runtimeOperationId', runtimeOperationId)
    boundPayload.put('driverDigest', driverDigest)
    def body = groovy.json.JsonOutput.toJson(boundPayload)
    def nonce = "control-${env.BUILD_TAG}-${UUID.randomUUID().toString()}"
    def input = "${env.ZIRCON_REPO_ROOT}/.jenkins/state/controls/${nonce}.json"
    def output = "${env.ZIRCON_REPO_ROOT}/.jenkins/state/responses/${nonce}.json"
    writeFile(file: input, text: body)
    def exitCode = bat(returnStatus: true, label: "Jenkins control ${domain}/${action}", script: """@echo off
"%JENKINS_PYTHON%" -B "%ZIRCON_DRIVER_LAUNCHER%" --repository "%ZIRCON_REPO_ROOT%" ${domain} ${action} --input-file "${input}" --output-file "${output}"
""")
    def value = parseControlResult(readFile(output))
    if (!(value.status instanceof String) || !value.status) {
        error("authoritative control operation ${action} returned no valid status")
    }
    if (value.status in ['failed', 'blocked']) {
        // The execution lock rejects the call before doing any work. Only its
        // explicit contention response may be retried by the job outside node.
        if (value.reasonCode == 'execution_busy' && value.retryable == true && exitCode in [0, 2]) return value
        error("authoritative control operation ${action} failed: ${value}")
    }
    if (exitCode != 0) error("authoritative control operation ${action} exited ${exitCode}: ${value}")
    return value
}

// The controller's script-security whitelist permits JsonSlurper.parseText.
// JsonSlurper returns LazyMaps for objects; copy only JSON values that Jenkins
// can serialize safely before a result crosses a suspended Pipeline continuation.
@NonCPS
def plainJsonValue(Object value) {
    if (value == null || value instanceof String || value instanceof Boolean || value instanceof Number) {
        return value
    }
    if (value instanceof Map) {
        def result = new LinkedHashMap()
        for (def entry : value.entrySet()) {
            if (!(entry.getKey() instanceof String)) {
                throw new IllegalArgumentException('JSON object keys must be strings')
            }
            result.put(entry.getKey(), plainJsonValue(entry.getValue()))
        }
        return result
    }
    if (value instanceof List) {
        def result = new ArrayList(value.size())
        for (def item : value) {
            result.add(plainJsonValue(item))
        }
        return result
    }
    throw new IllegalArgumentException('Unsupported JSON value type')
}

@NonCPS
def parseControlResult(String raw) {
    def parsed = new groovy.json.JsonSlurper().parseText(raw.trim())
    if (!(parsed instanceof Map)) {
        throw new IllegalArgumentException('Control result must be a JSON object')
    }

    // Keep the Pipeline result contract bounded.  Nested identity/receipt/stage
    // values are copied recursively, so no LazyMap or LazyList reaches CPS.
    def boundedFields = [
        'status', 'reasonCode', 'executionId', 'recipeRef', 'template', 'receiptRef',
        'receipt', 'stages', 'generation', 'observedGeneration', 'retryable', 'notifyEligible',
        'identity', 'runtimeOperationId', 'driverDigest', 'inventoryDigest', 'result',
        'sealedInputRef', 'patchOperationRef', 'patchRequestRef',
        'currentSourceRef', 'sourcePreparationRef', 'buildRoot',
        'sourceInputDigest', 'coverageDigest', 'stageImplementationDigest'
    ]
    def result = new LinkedHashMap()
    for (def field : boundedFields) {
        result.put(field, plainJsonValue(parsed.get(field)))
    }
    return result
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
