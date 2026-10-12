// Short maintenance controls release the agent between inventory and GC.
def invokeControl(String action, Map payload) {
    def result
    if (!params.RUNTIME_OPERATION_ID || !params.STAGE_IMPLEMENTATION_DIGEST) error('runtime identity parameters are required')
    if (!env.ZIRCON_AGENT_LABEL || params.RUNTIME_OPERATION_ID != env.ZIRCON_RUNTIME_OPERATION_ID || params.STAGE_IMPLEMENTATION_DIGEST != env.ZIRCON_DRIVER_DIGEST) error('runtime identity fence failed')
    // Reuse the exact operation and payload. A busy lock performed no work;
    // all other failures keep their original error/reconciliation behavior.
    for (int attempt = 0; attempt < 12; attempt++) {
        node(env.ZIRCON_AGENT_LABEL) {
            if (!env.ZIRCON_SEALED_DRIVER || !env.ZIRCON_DRIVER_LAUNCHER || !env.ZIRCON_DRIVER_DIGEST) error('sealed driver environment is required')
            result = load(env.ZIRCON_SEALED_DRIVER).control('maintenance', action, payload)
        }
        if (!(result.status in ['failed', 'blocked'] && result.reasonCode == 'execution_busy' && result.retryable == true)) return result
        if (attempt == 11) error("maintenance/${action} exceeded execution contention retry budget")
        // Release the executor before each wait; at most eleven five-second waits.
        sleep(time: 5, unit: 'SECONDS')
    }
}

def collectBuildCache(Map payload) {
    try {
        for (int chunk = 0; chunk < 12; chunk++) {
            def response = invokeControl('cache-maintenance', payload)
            def result = response.result
            if (!(result instanceof Map) || result.phase != 'gc' ||
                result.operationId != payload.operationId || response.inventoryDigest != payload.inventoryDigest) {
                error('build-cache GC returned a mismatched operation or inventory')
            }
            // A clean yield has no unresolved deletion intent. Missing evidence
            // is unknown, so only an explicit null and zero unknown files allow
            // another chunk of this exact operation.
            if (!result.containsKey('pendingFile') || result.pendingFile != null || result.unknownFileCount != 0) {
                error('build-cache GC has an unknown deletion outcome; reconcile the original operation')
            }
            if (response.status == 'accepted' && result.status == 'complete') return response
            if (response.status != 'waiting' || result.status != 'yielded') {
                error('build-cache GC did not complete or yield safely; reconcile the original operation')
            }
            if (chunk == 11) {
                currentBuild.description = "GC ${payload.operationId} pending after chunk budget"
                error('build-cache GC remains pending after its chunk budget; reconcile the original operation')
            }
            // invokeControl has left node; the same operation, inventory, request
            // and runtime binding continue after the executor has been released.
            sleep(time: 1, unit: 'SECONDS')
        }
    } catch (err) {
        try {
            invokeControl('cache-maintenance', new LinkedHashMap(payload) + [phase: 'reconcile'])
        } catch (reconcileErr) {
            echo "build-cache reconcile failed after original error: ${reconcileErr}"
        }
        throw err
    }
}
if (!params.REPOSITORY_ID || !params.SESSION_ID || !params.REQUEST_ID || !params.OPERATION_ID) error('maintenance request identity is required')
if (!params.CLEANUP_SCOPE_JSON) error('bounded cleanup scope is required')
def identity = [repositoryId: params.REPOSITORY_ID, sessionId: params.SESSION_ID,
                requestId: params.REQUEST_ID]
@NonCPS
def cleanupScope(String text) {
    def value = new groovy.json.JsonSlurper().parseText(text)
    if (!(value instanceof Map)) throw new IllegalArgumentException('Cleanup scope must be a JSON object')
    if (value.candidatePath) {
        if (value.keySet().any { !(it in ['candidatePath', 'maxBytes', 'maxFiles', 'maxSeconds', 'relativeSubtree']) }) {
            throw new IllegalArgumentException('Build-cache cleanup scope contains an unknown field')
        }
        def result = [candidatePath: value.candidatePath.toString(),
                      maxBytes: value.maxBytes as long, maxFiles: value.maxFiles as int]
        if (value.containsKey('maxSeconds')) {
            if (!(value.maxSeconds instanceof Number)) throw new IllegalArgumentException('Build-cache maxSeconds must be numeric')
            result.put('maxSeconds', value.maxSeconds)
        }
        if (value.containsKey('relativeSubtree')) {
            if (!(value.relativeSubtree instanceof String)) throw new IllegalArgumentException('Build-cache relativeSubtree must be a string')
            result.put('relativeSubtree', value.relativeSubtree)
        }
        return result
    }
    return [digests: value.digests.collect { it.toString() },
            maxBytes: value.maxBytes as long, maxObjects: value.maxObjects as int]
}

@NonCPS
def plainReviewValue(Object value, int depth, List budget) {
    budget[0] = budget[0] - 1
    if (depth > 8 || budget[0] < 0) throw new IllegalArgumentException('Protection review exceeds JSON structure bounds')
    if (value instanceof String) {
        if (value.length() > 8192) throw new IllegalArgumentException('Protection review string exceeds its bound')
        return value
    }
    if (value == null || value instanceof Boolean || value instanceof Number) return value
    if (value instanceof Map) {
        def result = new LinkedHashMap()
        for (def entry : value.entrySet()) {
            if (!(entry.getKey() instanceof String) || entry.getKey().length() > 256) {
                throw new IllegalArgumentException('Protection review requires bounded string keys')
            }
            result.put(entry.getKey(), plainReviewValue(entry.getValue(), depth + 1, budget))
        }
        return result
    }
    if (value instanceof List) {
        def result = new ArrayList(value.size())
        for (def item : value) result.add(plainReviewValue(item, depth + 1, budget))
        return result
    }
    throw new IllegalArgumentException('Protection review contains an unsupported JSON value')
}

@NonCPS
def protectionReview(String text) {
    if (!text || text.length() > 65536) throw new IllegalArgumentException('Protection review exceeds its JSON size bound')
    def value = new groovy.json.JsonSlurper().parseText(text)
    if (!(value instanceof Map)) throw new IllegalArgumentException('Protection review must be a JSON object')
    // Preserve the supplied values exactly. The backend checks the registered
    // review and its pins; ordinary collections keep the Pipeline CPS-safe.
    return plainReviewValue(value, 0, [1024])
}
def scope = cleanupScope(params.CLEANUP_SCOPE_JSON)
if (!params.RUNTIME_OPERATION_ID || !params.STAGE_IMPLEMENTATION_DIGEST || !env.ZIRCON_AGENT_LABEL || params.RUNTIME_OPERATION_ID != env.ZIRCON_RUNTIME_OPERATION_ID || params.STAGE_IMPLEMENTATION_DIGEST != env.ZIRCON_DRIVER_DIGEST) error('runtime identity fence failed')
def payload = [identity: identity, runtimeOperationId: params.RUNTIME_OPERATION_ID, buildRoot: params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT,
               operationId: params.OPERATION_ID, cleanupScope: scope]
if (params.CACHE_KIND == 'existing-build-cache') {
    if (!params.ATTEMPT_ID || !params.GENERATION || !params.SOURCE_INPUT_DIGEST || !params.COVERAGE_DIGEST) error('complete build-cache request identity is required')
    identity.put('attemptId', params.ATTEMPT_ID)
    identity.put('generation', params.GENERATION as int)
    identity.put('sourceInputDigest', params.SOURCE_INPUT_DIGEST)
    identity.put('coverageDigest', params.COVERAGE_DIGEST)
    identity.put('stageImplementationDigest', params.STAGE_IMPLEMENTATION_DIGEST)
    payload.put('cacheKind', params.CACHE_KIND)
    if (params.PROTECTION_REVIEW_JSON) payload.put('reviewedProtection', protectionReview(params.PROTECTION_REVIEW_JSON))
    stage('Register bounded build cache') {
        invokeControl('cache-maintenance', new LinkedHashMap(payload) + [phase: 'intake'])
    }
    def inventory
    stage('Inventory') {
        inventory = invokeControl('cache-maintenance', new LinkedHashMap(payload) + [phase: 'inventory'])
    }
    if (!inventory.inventoryDigest) error('authoritative build-cache inventory digest is required')
    stage('Bounded garbage collection') {
        collectBuildCache(new LinkedHashMap(payload) + [phase: 'gc', inventoryDigest: inventory.inventoryDigest])
    }
} else {
    if (params.CACHE_KIND) error('unsupported cache kind')
    stage('Inventory') { invokeControl('inventory', payload) }
    stage('Bounded garbage collection') { invokeControl('gc', payload) }
}
