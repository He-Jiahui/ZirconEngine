// Short maintenance controls release the agent between inventory and GC.
def invokeControl(String action, Map payload) {
    def result
    node('zircon-windows') {
        if (!env.ZIRCON_SEALED_DRIVER || !env.ZIRCON_DRIVER_LAUNCHER || !env.ZIRCON_DRIVER_DIGEST) error('sealed driver environment is required')
        result = load(env.ZIRCON_SEALED_DRIVER).control('maintenance', action, payload)
    }
    return result
}
if (!params.REPOSITORY_ID || !params.SESSION_ID || !params.REQUEST_ID || !params.OPERATION_ID) error('maintenance request identity is required')
if (!params.CLEANUP_SCOPE_JSON) error('bounded cleanup scope is required')
def identity = [repositoryId: params.REPOSITORY_ID, sessionId: params.SESSION_ID,
                requestId: params.REQUEST_ID]
@NonCPS
def cleanupScope(String text) {
    def value = new groovy.json.JsonSlurper().parseText(text)
    return [digests: value.digests.collect { it.toString() },
            maxBytes: value.maxBytes as long, maxObjects: value.maxObjects as int]
}
def scope = cleanupScope(params.CLEANUP_SCOPE_JSON)
if (!params.RUNTIME_OPERATION_ID || !params.STAGE_IMPLEMENTATION_DIGEST || !env.ZIRCON_AGENT_LABEL || params.RUNTIME_OPERATION_ID != env.ZIRCON_RUNTIME_OPERATION_ID || params.STAGE_IMPLEMENTATION_DIGEST != env.ZIRCON_DRIVER_DIGEST) error('runtime identity fence failed')
def payload = [identity: identity, runtimeOperationId: params.RUNTIME_OPERATION_ID, buildRoot: params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT,
               operationId: params.OPERATION_ID, cleanupScope: scope]
stage('Inventory') { invokeControl('inventory', payload) }
stage('Bounded garbage collection') { invokeControl('gc', payload) }
