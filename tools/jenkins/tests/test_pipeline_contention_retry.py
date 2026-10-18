"""Exercise canonical Groovy control/job code with an in-memory Pipeline DSL.

No Jenkins service, native control command, source fixture or compiled file is
created. The local Jenkins Groovy jar is optional on hosts without Jenkins; a
skipped test is not Pipeline acceptance evidence.
"""
from pathlib import Path
import json
import os
import subprocess
import unittest


GROOVY_HARNESS = r'''
import groovy.json.JsonOutput
import groovy.json.JsonSlurper

// The plain Groovy runner does not supply Jenkins' default annotation import.
// This annotation only lets the real source compile; it does not emulate CPS.
@interface NonCPS {}

class PipelineFixture {
    File root
    Binding binding
    private static final GroovyClassLoader compiler = new GroovyClassLoader(PipelineFixture.class.classLoader)
    private static final Map compiledScripts = [:]
    def driver
    List plan
    List calls = []
    List writes = []
    List sleeps = []
    List labels = []
    List stages = []
    List messages = []
    int position = 0
    int nodeDepth = 0
    def current

    PipelineFixture(File repository, List responses) {
        root = repository
        plan = responses
        def environment = [ZIRCON_RUNTIME_OPERATION_ID: 'runtime-test',
            ZIRCON_DRIVER_DIGEST: 'd' * 64, ZIRCON_AGENT_LABEL: 'runtime-specific-agent',
            ZIRCON_SEALED_DRIVER: 'sealed-driver', ZIRCON_DRIVER_LAUNCHER: 'sealed-launcher',
            ZIRCON_REPO_ROOT: root.absolutePath, BUILD_TAG: 'test']
        binding = new Binding([env: environment,
            params: [RUNTIME_OPERATION_ID: 'runtime-test', STAGE_IMPLEMENTATION_DIGEST: 'd' * 64],
            currentBuild: [:]])
        binding.setVariable('error', { message -> throw new IllegalStateException(message.toString()) })
        binding.setVariable('echo', { message -> messages << message.toString() })
        binding.setVariable('stage', { label, Closure body ->
            assert nodeDepth == 0
            stages << label
            return body()
        })
        binding.setVariable('node', { label, Closure body ->
            assert nodeDepth == 0 : 'nested executor allocation'
            assert label == environment.ZIRCON_AGENT_LABEL : 'unfenced agent label'
            labels << label
            nodeDepth++
            try { return body() } finally { nodeDepth-- }
        })
        binding.setVariable('sleep', { Map options ->
            assert nodeDepth == 0 : 'retry retained an executor'
            assert options.unit == 'SECONDS'
            sleeps << options.time
        })
        binding.setVariable('load', { path ->
            assert nodeDepth == 1
            assert path == environment.ZIRCON_SEALED_DRIVER
            return driver
        })
        binding.setVariable('writeFile', { Map options ->
            assert nodeDepth == 1
            writes << new LinkedHashMap(options)
        })
        binding.setVariable('bat', { Map options ->
            assert nodeDepth == 1
            assert options.returnStatus == true : 'batch exit prevented response parsing'
            assert options.script.contains('%ZIRCON_DRIVER_LAUNCHER%')
            assert position < plan.size() : 'control exceeded planned attempt bound'
            calls << [label: options.label, payload: new JsonSlurper().parseText(writes.last().text)]
            current = plan[position++]
            if (current.exception) throw new IllegalStateException(current.exception)
            return current.exitCode
        })
        binding.setVariable('readFile', { path ->
            assert nodeDepth == 1
            if (current.missing) throw new IllegalStateException('missing control response')
            return current.containsKey('raw') ? current.raw : JsonOutput.toJson(current.response)
        })
        driver = freshScript(new File(root, '.jenkins/pipeline/steps.groovy'), 'canonical_steps.groovy')
    }

    def freshScript(File path, String logicalName) {
        // Compile the canonical bytes once, then give each invocation its own
        // Script and Binding. Retrying still exercises every real control path.
        def source = path.getText('UTF-8')
        def key = [path.absolutePath, source]
        def scriptClass = compiledScripts[key]
        if (scriptClass == null) {
            scriptClass = compiler.parseClass(source, logicalName)
            compiledScripts[key] = scriptClass
        }
        def script = scriptClass.newInstance()
        script.binding = binding
        return script
    }

    def jobScript(String job) {
        // Use a legal logical name for Jenkins' hyphenated Pipeline filenames.
        return freshScript(new File(root, ".jenkins/pipeline/zircon-${job}.groovy"), "zircon_${job}.groovy")
    }

    def invoke(String job, String action, Map payload) {
        def script = jobScript(job)
        return job == 'flow' ? script.invokeControl('flow', action, payload) : script.invokeControl(action, payload)
    }

    def collectCache(Map payload) {
        return jobScript('maintenance').collectBuildCache(payload)
    }

    def runMaintenance(Map parameters) {
        binding.getVariable('params').putAll(parameters)
        return jobScript('maintenance').run()
    }

    def control(Map payload = [:]) {
        nodeDepth++
        try { return driver.control('execution', 'observe-execution', payload) }
        finally { nodeDepth-- }
    }
}

def rejected = { Closure operation ->
    try { operation() } catch (Exception failure) { return failure }
    throw new AssertionError('operation unexpectedly succeeded')
}
def root = new File(args[0])
def busy = [status: 'blocked', reasonCode: 'execution_busy', retryable: true]
def complete = [status: 'complete', reasonCode: 'ok', retryable: false,
    identity: [requestId: 'original', generation: 7], stages: [[status: 'passed']],
    receipt: [executionId: 'original-execution'], unknownField: 'must be dropped']

// The authoritative retry response must survive both documented CLI exits.
for (status in ['blocked', 'failed']) {
    for (exitCode in [0, 2]) {
        def fixture = new PipelineFixture(root, [[exitCode: exitCode, response: busy + [status: status]]])
        def result = fixture.control()
        assert result.status == status && result.reasonCode == 'execution_busy' && result.retryable == true
        assert fixture.position == 1 && fixture.sleeps.empty
    }
}
def success = new PipelineFixture(root, [[exitCode: 0, response: complete]])
def parsed = success.control()
assert parsed.identity == complete.identity && parsed.receipt == complete.receipt && parsed.stages == complete.stages
assert !parsed.containsKey('unknownField')
assert parsed instanceof LinkedHashMap && parsed.identity instanceof LinkedHashMap
assert parsed.stages instanceof ArrayList && parsed.stages[0] instanceof LinkedHashMap

// Retryability alone is insufficient. Unknown delivery outcomes, source/native
// failures, bad JSON, absent response files and unexpected exits fail closed.
def refusals = [
    [exitCode: 2, response: busy + [retryable: false]],
    [exitCode: 2, response: busy + [retryable: 'true']],
    [exitCode: 2, response: busy + [retryable: null]],
    [exitCode: 2, response: busy + [reasonCode: 'state_busy']],
    [exitCode: 2, response: busy + [reasonCode: 'delivery_unknown']],
    [exitCode: 2, response: busy + [reasonCode: 'source_failed']],
    [exitCode: 2, response: busy + [reasonCode: 'native_termination_unproven']],
    [exitCode: 2, response: busy + [reasonCode: 'execution_identity_corrupt']],
    [exitCode: 2, response: busy + [status: 'failed', retryable: false]],
    [exitCode: 2, response: complete],
    [exitCode: 7, response: busy],
    [exitCode: 0, response: [:]],
    [exitCode: 0, response: [status: [bad: 'type']]],
    [exitCode: 2, raw: 'invalid json'],
    [exitCode: 0, raw: '[]'],
    [exitCode: 2, missing: true],
    [exitCode: 2, exception: 'batch interrupted']
]
for (response in refusals) {
    def fixture = new PipelineFixture(root, [response])
    rejected { fixture.control() }
    assert fixture.position == 1 && fixture.sleeps.empty && fixture.nodeDepth == 0
}

// Compile each real job, then call its actual wrapper with the actual adapter.
// The no-op DSL records executor scopes instead of contacting Jenkins.
for (job in ['flow', 'execution', 'maintenance']) {
    def action = job == 'flow' ? 'observe-flow' : job == 'execution' ? 'observe-execution' : 'inventory'
    def payload = [identity: [sessionId: 'session', requestId: 'request', attemptId: 'attempt', generation: 7],
        executionId: 'execution', recipeRef: 'recipe', operationId: 'operation']
    def before = JsonOutput.toJson(payload)
    def fixture = new PipelineFixture(root, [
        [exitCode: 2, response: busy], [exitCode: 2, response: busy], [exitCode: 0, response: complete]])
    def result = fixture.invoke(job, action, payload)
    assert result.status == 'complete'
    assert fixture.position == 3 && fixture.sleeps == [5, 5] && fixture.labels.size() == 3
    assert fixture.nodeDepth == 0 && JsonOutput.toJson(payload) == before
    assert fixture.calls.every { it.label == "Jenkins control ${job}/${action}" }
    assert fixture.calls.every { it.payload == fixture.calls[0].payload }
    assert fixture.calls[0].payload.identity == payload.identity
    assert fixture.calls[0].payload.executionId == 'execution'
    assert fixture.calls[0].payload.recipeRef == 'recipe'
    assert fixture.calls[0].payload.operationId == 'operation'
    assert fixture.calls[0].payload.runtimeOperationId == 'runtime-test'
    assert fixture.calls[0].payload.driverDigest == 'd' * 64

    def exhausted = new PipelineFixture(root, (1..12).collect { [exitCode: 2, response: busy] })
    def exhaustion = rejected { exhausted.invoke(job, action, payload) }
    assert exhaustion.message.contains('contention retry budget')
    assert exhausted.position == 12 && exhausted.sleeps == ([5] * 11) && exhausted.nodeDepth == 0

    for (response in refusals) {
        def refusal = new PipelineFixture(root, [response])
        rejected { refusal.invoke(job, action, payload) }
        assert refusal.position == 1 && refusal.sleeps.empty && refusal.nodeDepth == 0
    }

    for (badFence in ['runtime', 'driver', 'label', 'launcher']) {
        def fenced = new PipelineFixture(root, [[exitCode: 0, response: complete]])
        if (badFence == 'runtime') fenced.binding.getVariable('params').RUNTIME_OPERATION_ID = 'foreign-runtime'
        if (badFence == 'driver') fenced.binding.getVariable('params').STAGE_IMPLEMENTATION_DIGEST = 'foreign-driver'
        if (badFence == 'label') fenced.binding.getVariable('env').ZIRCON_AGENT_LABEL = null
        if (badFence == 'launcher') fenced.binding.getVariable('env').ZIRCON_DRIVER_LAUNCHER = null
        rejected { fenced.invoke(job, action, payload) }
        assert fenced.position == 0 && fenced.sleeps.empty && fenced.nodeDepth == 0
    }
}

// A yielded cache chunk is explicitly safe to continue, and carries its
// original operation and inventory. Completion requires both status levels.
def cacheDigest = 'b' * 64
def suppliedReview = [kind: 'reviewed-legacy-single-blob-v1',
    candidatePath: 'D:/cargo-targets/zircon-local/cache-candidate', blobDigest: 'e' * 64,
    blobByteLength: 3800, oldPath: 'D:/cargo-targets/legacy-objects/blob',
    producer: [path: 'producer.py', sha256: 'f' * 64], archive: [path: 'archive.json', sha256: 'a' * 64],
    rows: [[domain: 'artifact_ref', key: 'retained-ref', version: 1, payloadDigest: 'b' * 64]],
    review: [source: 'user', task: 'the supplied review remains unchanged']]
def cachePayload = [identity: [repositoryId: 'repo', sessionId: 'session', requestId: 'request',
        attemptId: 'attempt', generation: 7, sourceInputDigest: 'a' * 64, coverageDigest: 'c' * 64,
        stageImplementationDigest: 'd' * 64], operationId: 'cache-operation',
    runtimeOperationId: 'runtime-test', buildRoot: 'D:/cargo-targets', cacheKind: 'existing-build-cache',
    cleanupScope: [candidatePath: 'D:/cargo-targets/zircon-local/cache-candidate', maxBytes: 20, maxFiles: 2, maxSeconds: 7],
    reviewedProtection: suppliedReview,
    phase: 'gc', inventoryDigest: cacheDigest]
def cacheResponse = { String status, String detail ->
    [status: status, inventoryDigest: cacheDigest,
     result: [phase: 'gc', status: detail, operationId: 'cache-operation', pendingFile: null,
              unknownFileCount: 0, reclaimedBytes: 10, unreclaimedBytes: detail == 'complete' ? 0 : 10,
              removedFileCount: 1, JenkinsAcceptance: false, milestoneAcceptance: false]]
}
def yielded = cacheResponse('waiting', 'yielded')
def collected = cacheResponse('accepted', 'complete')
def reconciled = cacheResponse('waiting', 'yielded')
reconciled.result.phase = 'reconcile'
def cache = new PipelineFixture(root, [[exitCode: 0, response: yielded],
    [exitCode: 0, response: yielded], [exitCode: 0, response: collected]])
def cacheBefore = JsonOutput.toJson(cachePayload)
assert cache.collectCache(cachePayload).result.status == 'complete'
assert cache.position == 3 && cache.sleeps == [1, 1] && cache.nodeDepth == 0
assert JsonOutput.toJson(cachePayload) == cacheBefore
assert cache.calls.every { it.payload == cache.calls[0].payload }
assert cache.calls.every { it.payload.operationId == 'cache-operation' && it.payload.inventoryDigest == cacheDigest }
assert cache.calls.every { it.payload.reviewedProtection == suppliedReview && it.payload.cleanupScope.maxSeconds == 7 }

def cacheExhausted = new PipelineFixture(root,
    (1..12).collect { [exitCode: 0, response: yielded] } + [[exitCode: 0, response: reconciled]])
def cacheExhaustion = rejected { cacheExhausted.collectCache(cachePayload) }
assert cacheExhaustion.message.contains('pending after its chunk budget')
assert cacheExhausted.position == 13 && cacheExhausted.sleeps == ([1] * 11) && cacheExhausted.nodeDepth == 0
assert cacheExhausted.calls[0..11].every { it.payload.phase == 'gc' && it.payload == cacheExhausted.calls[0].payload }
assert cacheExhausted.calls.last().payload == cacheExhausted.calls[0].payload + [phase: 'reconcile']
assert cacheExhausted.binding.getVariable('currentBuild').description.contains('cache-operation pending')

def cacheRefusals = [
    yielded + [result: yielded.result + [pendingFile: [path: 'unacknowledged']]],
    yielded + [result: yielded.result.findAll { key, value -> key != 'pendingFile' }],
    yielded + [result: yielded.result + [unknownFileCount: 1]],
    yielded + [result: yielded.result.findAll { key, value -> key != 'unknownFileCount' }],
    yielded + [result: yielded.result + [operationId: 'foreign-operation']],
    yielded + [inventoryDigest: 'f' * 64],
    yielded + [result: yielded.result + [phase: 'inventory']],
    cacheResponse('accepted', 'yielded'),
    cacheResponse('waiting', 'complete'),
    cacheResponse('accepted', 'partial'),
    cacheResponse('accepted', 'unknown'),
    cacheResponse('accepted', 'running'),
    cacheResponse('blocked', 'unknown') + [reasonCode: 'cache_deletion_unknown'],
    collected + [result: collected.result + [pendingFile: [path: 'unacknowledged']]]
]
for (response in cacheRefusals) {
    def fixture = new PipelineFixture(root, [[exitCode: 0, response: response], [exitCode: 0, response: reconciled]])
    rejected { fixture.collectCache(cachePayload) }
    assert fixture.position == 2 && fixture.sleeps.empty && fixture.nodeDepth == 0
    assert fixture.calls*.payload*.phase == ['gc', 'reconcile']
    assert fixture.calls.last().payload == fixture.calls[0].payload + [phase: 'reconcile']
}
def reconcileFailure = new PipelineFixture(root, [[exitCode: 0, response: cacheRefusals[0]],
    [exitCode: 2, exception: 'reconcile interrupted']])
def original = rejected { reconcileFailure.collectCache(cachePayload) }
assert original.message.contains('unknown deletion outcome')
assert reconcileFailure.messages.any { it.contains('reconcile failed after original error') }
assert reconcileFailure.sleeps.empty && reconcileFailure.nodeDepth == 0

// Run the actual maintenance entry, not just its helper, to verify inventory
// binding and completion across intake, inventory and multiple GC chunks.
def maintenance = new PipelineFixture(root, [[exitCode: 0, response: [status: 'accepted', result: [phase: 'intake']]],
    [exitCode: 0, response: [status: 'accepted', inventoryDigest: cacheDigest, result: [phase: 'inventory']]],
    [exitCode: 0, response: yielded], [exitCode: 0, response: collected]])
maintenance.runMaintenance([REPOSITORY_ID: 'repo', SESSION_ID: 'session', REQUEST_ID: 'request',
    ATTEMPT_ID: 'attempt', GENERATION: '7', SOURCE_INPUT_DIGEST: 'a' * 64, COVERAGE_DIGEST: 'c' * 64,
    OPERATION_ID: 'cache-operation', CACHE_KIND: 'existing-build-cache', BUILD_ROOT: 'D:/cargo-targets',
    CLEANUP_SCOPE_JSON: JsonOutput.toJson(cachePayload.cleanupScope),
    PROTECTION_REVIEW_JSON: JsonOutput.toJson(suppliedReview)])
assert maintenance.position == 4 && maintenance.sleeps == [1] && maintenance.nodeDepth == 0
assert maintenance.stages == ['Register bounded build cache', 'Inventory', 'Bounded garbage collection']
assert maintenance.calls*.payload*.phase == ['intake', 'inventory', 'gc', 'gc']
assert maintenance.calls[2].payload == maintenance.calls[3].payload
assert maintenance.calls[2].payload.inventoryDigest == cacheDigest
assert maintenance.calls[2].payload.identity == cachePayload.identity
assert maintenance.calls.every { it.payload.reviewedProtection == suppliedReview && it.payload.cleanupScope.maxSeconds == 7 }

def reviewParser = maintenance.jobScript('maintenance')
def ordinaryReview = reviewParser.protectionReview(JsonOutput.toJson(suppliedReview))
assert ordinaryReview == suppliedReview
assert ordinaryReview instanceof LinkedHashMap && ordinaryReview.producer instanceof LinkedHashMap
assert ordinaryReview.rows instanceof ArrayList && ordinaryReview.rows[0] instanceof LinkedHashMap
assert ordinaryReview.review instanceof LinkedHashMap
for (badReview in ['[]', 'null', 'invalid JSON', '{"x":"' + ('a' * 8193) + '"}',
                   '[' * 10 + '0' + ']' * 10, '{"rows":[' + ([0] * 1025).join(',') + ']}',
                   '{"x":"' + ('a' * 65536) + '"}']) {
    rejected { reviewParser.protectionReview(badReview) }
}
def nestedReview = [value: 0]
for (int depth = 0; depth < 10; depth++) nestedReview = [child: nestedReview]
rejected { reviewParser.protectionReview(JsonOutput.toJson(nestedReview)) }
rejected { reviewParser.plainReviewValue([nonJson: new Object()], 0, [1024]) }

// Other maintenance kinds do not parse or forward protection review. A legacy
// artifact request therefore remains independent of the new cache parameter.
def legacy = new PipelineFixture(root, [[exitCode: 0, response: [status: 'accepted']],
    [exitCode: 0, response: [status: 'accepted']]])
legacy.runMaintenance([REPOSITORY_ID: 'repo', SESSION_ID: 'session', REQUEST_ID: 'request',
    OPERATION_ID: 'artifact-operation', BUILD_ROOT: 'D:/cargo-targets',
    CLEANUP_SCOPE_JSON: JsonOutput.toJson([digests: ['a' * 64], maxBytes: 20, maxObjects: 2]),
    PROTECTION_REVIEW_JSON: 'malformed but irrelevant'])
assert legacy.position == 2 && legacy.sleeps.empty
assert legacy.calls.every { !it.payload.containsKey('reviewedProtection') }
// Run the real terminal finally path and verify maintenance authority and errors.
def flowParameters = [REPOSITORY_ID:'repo', SESSION_ID:'session', REQUEST_ID:'request',
    ATTEMPT_ID:'attempt', GENERATION:'7', SOURCE_INPUT_DIGEST:'a' * 64,
    COVERAGE_DIGEST:'c' * 64, SEALED_INPUT_REF:'a' * 64,
    PATCH_OPERATION_REF:'original-patch', BUILD_ROOT:'D:/cargo-targets',
    ALLOW_COMMIT:'false', ALLOW_NOTIFY:'false']
def successfulLight = [
    [exitCode:0,response:[status:'pending',executionId:'light']],
    [exitCode:0,response:[status:'ready',template:'comments_only',
        stages:['patch','syntax','format','acceptance','commit']]],
    [exitCode:0,response:[status:'ready']],
    [exitCode:0,response:[status:'ready']],
    [exitCode:0,response:[status:'ready']],
    [exitCode:0,response:[status:'accepted']],
    [exitCode:0,response:[status:'accepted',notifyEligible:false]]]
for (cacheCase in [
    [assessment:[cleanupRequired:false,gcAuthorized:true,scratch:[status:'empty']],gc:null],
    [assessment:[cleanupRequired:true,gcAuthorized:false,scratch:[status:'owned']],gc:null],
    [assessment:[cleanupRequired:true,gcAuthorized:true,scratch:[status:'owned']],
        gc:[exitCode:0,response:[status:'observed',result:[status:'complete',reclaimedBytes:262144]]]],
    [assessment:[cleanupRequired:true,gcAuthorized:true,scratch:[status:'owned']],
        gc:[exitCode:2,response:[status:'failed',reasonCode:'storage_unknown']]]]) {
    def responses = new ArrayList(successfulLight)
    responses << [exitCode:0,response:[status:'observed',result:cacheCase.assessment]]
    if (cacheCase.gc) responses << cacheCase.gc
    def flowFixture = new PipelineFixture(root,responses)
    flowFixture.binding.getVariable('params').putAll(flowParameters)
    flowFixture.jobScript('flow').run()
    assert flowFixture.position == responses.size() && flowFixture.nodeDepth == 0
    assert flowFixture.calls.findAll { it.label == 'Jenkins control flow/run-stage' }*.payload*.stage ==
        ['patch','syntax','format','acceptance']
    assert !flowFixture.calls.any { it.label in ['Jenkins control flow/submit-execution',
        'Jenkins control flow/ensure-execution-job', 'Jenkins control integration/commit-flow'] }
    def maintenanceCalls = flowFixture.calls.findAll { it.label == 'Jenkins control maintenance/post-task-cache' }
    assert maintenanceCalls.size() == (cacheCase.gc ? 2 : 1)
    assert maintenanceCalls.first().payload.phase == 'assess'
    assert maintenanceCalls.every { it.payload.identity.requestId == 'request' &&
        it.payload.identity.attemptId == 'attempt' && it.payload.identity.generation == 7 &&
        !it.payload.containsKey('candidatePath') && !it.payload.containsKey('buildRoot') }
    if (cacheCase.gc) assert maintenanceCalls.last().payload.phase == 'gc'
    if (cacheCase.gc?.exitCode == 2) assert flowFixture.messages.any { it.contains('independent retry') }
}
def failedSyntaxResponses = new ArrayList(successfulLight.take(3))
failedSyntaxResponses << [exitCode:2,response:[status:'failed',reasonCode:'original_syntax_failure']]
failedSyntaxResponses << [exitCode:0,response:[status:'cancelled']]
failedSyntaxResponses << [exitCode:0,response:[status:'observed',result:[cleanupRequired:false,
    gcAuthorized:true,scratch:[status:'native_terminal_pending']]]]
def failedSyntax = new PipelineFixture(root,failedSyntaxResponses)
failedSyntax.binding.getVariable('params').putAll(flowParameters)
def originalFailure = rejected { failedSyntax.jobScript('flow').run() }
assert originalFailure.message.contains('original_syntax_failure')
assert failedSyntax.position == failedSyntaxResponses.size() && failedSyntax.nodeDepth == 0
assert failedSyntax.calls.last().payload.phase == 'assess'

// A pending native guard is nonterminal. Reconcile the original execution;
// do not launch it again or retain an executor while waiting for proof.
def guardParameters = flowParameters + [EXECUTION_ID:'guard-execution', RECIPE_REF:'guard-recipe']
def pendingGuard = [exitCode:0, response:[status:'reconciling', executionId:'guard-execution']]
def passedGuard = [exitCode:0, response:[status:'passed', executionId:'guard-execution',
    terminalProof:[complete:true, activeProcesses:0, childrenGone:true]]]
def guardReady = new PipelineFixture(root, [pendingGuard, pendingGuard, passedGuard, passedGuard])
guardReady.binding.getVariable('params').putAll(guardParameters)
guardReady.jobScript('execution').run()
assert guardReady.position == 4 && guardReady.sleeps == [2] && guardReady.nodeDepth == 0
assert guardReady.stages == ['Managed execution', 'Finalize execution']
assert guardReady.calls*.label == ['Jenkins control execution/run-execution',
    'Jenkins control execution/reconcile-execution', 'Jenkins control execution/observe-execution',
    'Jenkins control execution/finalize-execution']
assert guardReady.calls.every { it.payload == guardReady.calls.first().payload }
assert guardReady.calls.first().payload.identity == [sessionId:'session', requestId:'request',
    repositoryId:'repo', attemptId:'attempt', generation:7, sourceInputDigest:'a' * 64,
    coverageDigest:'c' * 64, stageImplementationDigest:'d' * 64]
assert guardReady.calls.every { it.payload.executionId == 'guard-execution' && it.payload.recipeRef == 'guard-recipe' }

// Failed and unknown outcomes still fail immediately; finalization cannot
// turn either outcome into success or authorize a second run.
for (status in ['failed', 'unknown']) {
    def terminalGuard = new PipelineFixture(root,
        [[exitCode:0, response:[status:status, executionId:'guard-execution']], passedGuard])
    terminalGuard.binding.getVariable('params').putAll(guardParameters)
    def failure = rejected { terminalGuard.jobScript('execution').run() }
    assert failure.message.contains(status == 'failed' ? 'authoritative control operation run-execution failed' : 'execution did not complete') && failure.message.contains(status)
    assert terminalGuard.position == 2 && terminalGuard.sleeps.empty && terminalGuard.nodeDepth == 0
    assert terminalGuard.calls*.label == ['Jenkins control execution/run-execution',
        'Jenkins control execution/finalize-execution']
}

def guardBudgetPlan = [pendingGuard]
120.times { guardBudgetPlan.addAll([pendingGuard, pendingGuard]) }
guardBudgetPlan << passedGuard
def guardBudget = new PipelineFixture(root, guardBudgetPlan)
guardBudget.binding.getVariable('params').putAll(guardParameters)
def guardExhaustion = rejected { guardBudget.jobScript('execution').run() }
assert guardExhaustion.message.contains('execution did not complete') &&
    guardExhaustion.message.contains('reconciling')
assert guardBudget.position == 242 && guardBudget.sleeps.size() == 120 && guardBudget.nodeDepth == 0
assert guardBudget.calls.count { it.label == 'Jenkins control execution/run-execution' } == 1
assert guardBudget.calls.count { it.label == 'Jenkins control execution/reconcile-execution' } == 120
assert guardBudget.calls.count { it.label == 'Jenkins control execution/observe-execution' } == 120
assert guardBudget.calls.last().label == 'Jenkins control execution/finalize-execution'
println 'PASS canonical Pipeline contention, exit, identity, executor and fence contracts'
'''


class PipelineContentionRetryTests(unittest.TestCase):
    def test_actual_cleanup_scope_parser_preserves_fixed_leaf_and_rejects_invalid_fields(self):
        from tools.jenkins.workflow.build_cache_gc import CANDIDATE, RELATIVE_SUBTREE

        root = Path(__file__).resolve().parents[3]
        java = Path(os.environ.get("JENKINS_GROOVY_JAVA", str(root / ".jenkins/runtime/jdk/bin/java.exe")))
        libraries = sorted((root / ".jenkins/cache/war/WEB-INF/lib").glob("groovy-all-*.jar"))
        classpath = os.environ.get("JENKINS_GROOVY_CLASSPATH") or (str(libraries[-1]) if libraries else None)
        if not java.is_file() or not classpath:
            self.skipTest("Local Jenkins Java/Groovy runtime unavailable; behavioral acceptance remains pending")
        scope = {"candidatePath": CANDIDATE, "maxBytes": 1048576, "maxFiles": 17,
                 "maxSeconds": 7, "relativeSubtree": RELATIVE_SUBTREE}
        harness = r'''
import groovy.json.JsonOutput
import groovy.json.JsonSlurper
def source = new File(args[0], '.jenkins/pipeline/zircon-maintenance.groovy').getText('UTF-8')
def method = source =~ /(?s)def cleanupScope\(String text\) \{.*?\n\}/
assert method.find() : 'actual maintenance cleanupScope method is missing'
def parser = new GroovyShell().parse(method.group(), 'actual_cleanup_scope.groovy')
def expected = new JsonSlurper().parseText(args[1])
def parsed = parser.cleanupScope(JsonOutput.toJson(expected))
assert parsed == expected : 'registered scope changed before control dispatch'
assert parsed instanceof LinkedHashMap && parsed.relativeSubtree instanceof String
assert new JsonSlurper().parseText(JsonOutput.toJson(parsed)) == expected
def base = new LinkedHashMap(expected)
base.remove('relativeSubtree')
base.remove('maxSeconds')
assert parser.cleanupScope(JsonOutput.toJson(base)) == base : 'optional fields were invented'
for (bad in [null, true, 17, [], [path: expected.relativeSubtree]]) {
    def invalid = new LinkedHashMap(expected)
    invalid.relativeSubtree = bad
    boolean rejected = false
    try { parser.cleanupScope(JsonOutput.toJson(invalid)) }
    catch (IllegalArgumentException error) { rejected = true }
    assert rejected : 'non-string relativeSubtree was accepted'
}
def unknown = new LinkedHashMap(expected)
unknown.recursiveDelete = true
boolean rejected = false
try { parser.cleanupScope(JsonOutput.toJson(unknown)) }
catch (IllegalArgumentException error) { rejected = true }
assert rejected : 'unknown registered scope field was silently dropped'
println 'PASS actual cleanup scope parser'
'''
        completed = subprocess.run(
            [str(java), "-cp", classpath, "groovy.ui.GroovyMain", "-e", harness, str(root), json.dumps(scope)],
            cwd=root, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=60,
        )
        self.assertEqual(0, completed.returncode, completed.stdout + completed.stderr)
        self.assertIn("PASS actual cleanup scope parser", completed.stdout)

    def test_actual_groovy_adapter_and_jobs(self):
        root = Path(__file__).resolve().parents[3]
        default_java = root / ".jenkins/runtime/jdk/bin/java.exe"
        java = Path(os.environ.get("JENKINS_GROOVY_JAVA", str(default_java)))
        classpath = os.environ.get("JENKINS_GROOVY_CLASSPATH")
        if not classpath:
            libraries = sorted((root / ".jenkins/cache/war/WEB-INF/lib").glob("groovy-all-*.jar"))
            if libraries:
                classpath = str(libraries[-1])
        if not java.is_file() or not classpath:
            self.skipTest("Local Jenkins Java/Groovy runtime unavailable; behavioral acceptance remains pending")
        branch_harness = r'''
def branchParameters = [ACTION: 'commit-branch', SESSION_ID: 'branch-session',
    REQUEST_ID: 'branch-request', OPERATION_ID: 'branch-operation',
    EXPECTED_DRIVER_DIGEST: 'd' * 64, RUNTIME_OPERATION_ID: 'runtime-test',
    QUERY_ONLY: 'false', INPUT_JSON: JsonOutput.toJson([action: 'deliver-task',
        sessionId: 'branch-session', requestId: 'branch-request', operationId: 'branch-operation',
        sourceRoot: 'E:/Git/ZirconEngine/.jenkins/worktrees/branch-session-w',
        paths: ['owned.py'], dependencies: [], targetBranch: 'main', message: 'Scoped task'])]
def branchWaiting = [exitCode: 1, response: [status: 'waiting',
    reason: 'Another management operation owns Git; continue the same request']]
def branchCases = [
    [plan: [branchWaiting, branchWaiting, [exitCode: 0, response: [status: 'committed']]],
     attempts: 3, sleeps: [5, 5], terminal: 'committed'],
    [plan: [[exitCode: 1, response: [status: 'failed', reason: 'real failure']]],
     attempts: 1, sleeps: [], terminal: 'failed', failure: 'Scoped branch operation did not complete'],
    [plan: Collections.nCopies(120, branchWaiting), attempts: 120,
     sleeps: Collections.nCopies(119, 5), terminal: 'waiting',
     failure: 'Branch operation is still waiting'],
    [plan: [branchWaiting, [exitCode: 1, missingResponse: true]],
     attempts: 2, sleeps: [5], terminal: 'waiting', archivedAttempts: 1,
     failure: 'No branch receipt; reconcile this original request']
]
for (def branchCase in branchCases) {
    def branchFixture = new PipelineFixture(root, branchCase.plan)
    def environment = branchFixture.binding.getVariable('env')
    environment.ZIRCON_AGENT_LABEL = 'zircon-windows-agent'
    environment.BUILD_NUMBER = 'branch-build'
    branchFixture.binding.setVariable('params', new LinkedHashMap(branchParameters))
    def archives = []
    def responses = [:]
    def responseLookups = []
    branchFixture.binding.setVariable('dir', { path, Closure body ->
        assert branchFixture.nodeDepth == 1
        assert path == 'E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-management'
        body()
    })
    branchFixture.binding.setVariable('withEnv', { List values, Closure body ->
        assert branchFixture.nodeDepth == 1
        assert values*.toString() == ['INPUT_JSON=', 'ZR_BRANCH_ACTION=commit-branch',
            'ZR_BRANCH_SESSIONID=branch-session', 'ZR_BRANCH_REQUESTID=branch-request',
            'ZR_BRANCH_OPERATIONID=branch-operation', 'ZR_BRANCH_DRIVER=' + ('d' * 64),
            'ZR_BRANCH_RUNTIME=runtime-test']
        body()
    })
    branchFixture.binding.setVariable('bat', { Map options ->
        assert branchFixture.nodeDepth == 1 && options.returnStatus == true
        assert options.script.contains('E:/Git/ZirconEngine/tools/jenkins-management/branch_integration.py')
        assert options.script.contains('--output-file')
        assert branchFixture.position < branchFixture.plan.size()
        branchFixture.calls << new JsonSlurper().parseText(branchFixture.writes.last().text)
        branchFixture.current = branchFixture.plan[branchFixture.position++]
        def output = options.script =~ /--output-file "([^"]+)"/
        assert output.find()
        def outputPath = output.group(1)
        assert outputPath == "E:/Git/ZirconEngine/.jenkins/state/controls/branch-branch-operation.branch-build.poll-${branchFixture.position - 1}.response.json"
        if (!branchFixture.current.missingResponse) {
            responses[outputPath] = JsonOutput.toJson(branchFixture.current.response)
        }
        branchFixture.current.exitCode
    })
    branchFixture.binding.setVariable('fileExists', { path ->
        assert branchFixture.nodeDepth == 1 && path.endsWith('.response.json')
        responseLookups << path.toString()
        responses.containsKey(path.toString())
    })
    branchFixture.binding.setVariable('readFile', { Map options ->
        assert branchFixture.nodeDepth == 1 && options.file.endsWith('.response.json')
        assert responses.containsKey(options.file.toString())
        responses[options.file.toString()]
    })
    branchFixture.binding.setVariable('parseJson', { String response ->
        new JsonSlurper().parseText(response)
    })
    branchFixture.binding.setVariable('archiveArtifacts', { Map options ->
        assert branchFixture.nodeDepth == 1 && options.fingerprint == false
        def receiptBase = 'branch-branch-operation.branch-build'
        def pollReceipt = receiptBase + '.poll-' + (branchFixture.position - 1) + '.receipt.json'
        assert options.artifacts.toString() == receiptBase + '.receipt.json,' + pollReceipt
        def canonicalWrite = branchFixture.writes.findAll {
            it.file.toString() == receiptBase + '.receipt.json'
        }.last()
        def pollWrite = branchFixture.writes.findAll { it.file.toString() == pollReceipt }.last()
        assert pollWrite.text == canonicalWrite.text
        archives << new JsonSlurper().parseText(pollWrite.text)
    })
    def failure = null
    try {
        branchFixture.freshScript(new File(root, 'tools/jenkins-management/branch-integration.groovy'),
            'canonical_branch_integration.groovy').run()
    } catch (IllegalStateException error) {
        failure = error
    }
    assert branchCase.failure ? failure?.message?.contains(branchCase.failure) : failure == null
    assert branchFixture.position == branchCase.attempts
    assert branchFixture.labels.size() == branchCase.attempts && branchFixture.nodeDepth == 0
    assert branchFixture.sleeps == branchCase.sleeps
    assert branchFixture.calls.every { it == new JsonSlurper().parseText(branchParameters.INPUT_JSON) }
    assert branchFixture.binding.getVariable('params') == branchParameters
    def archivedAttempts = branchCase.get('archivedAttempts', branchCase.attempts)
    assert archives.size() == archivedAttempts && archives.last().status == branchCase.terminal
    assert responses.size() == archivedAttempts
    assert responseLookups == (0..<branchCase.attempts).collect {
        "E:/Git/ZirconEngine/.jenkins/state/controls/branch-branch-operation.branch-build.poll-${it}.response.json".toString()
    }
    if (branchCase.plan.last().missingResponse) {
        assert responses.containsKey(responseLookups.first())
        assert !responses.containsKey(responseLookups.last())
    }
}
println 'PASS canonical branch Git waiting, original request, executor release and bounded retry'
'''
        import tempfile
        temporary_root = Path(os.environ["TEMP"]).resolve()
        self.assertIn(temporary_root.drive.upper(), ("D:", "E:", "F:"))
        self.assertEqual("cargo-targets", temporary_root.parts[1].casefold())
        with tempfile.TemporaryDirectory(prefix="branch-waiting-groovy-", dir=temporary_root) as temporary:
            harness_path = Path(temporary) / "contention.groovy"
            harness_path.write_bytes((GROOVY_HARNESS + branch_harness).encode("utf-8"))
            completed = subprocess.run(
                [str(java), "-cp", classpath, "groovy.ui.GroovyMain", str(harness_path), str(root)],
                cwd=root, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=180,
            )
        self.assertEqual(0, completed.returncode, completed.stdout + completed.stderr)
        self.assertIn("PASS canonical Pipeline contention", completed.stdout)
        self.assertIn("PASS canonical branch Git waiting", completed.stdout)


    def test_prepared_flow_pipeline_repairs_only_the_proved_dispatch_mismatch(self):
        import hashlib
        root = Path(__file__).resolve().parents[3]
        bridge = root / "tools/dev/resume_jenkins_flow.py"
        self.assertIn(hashlib.sha256(bridge.read_bytes()).hexdigest(),
                      (root / ".jenkins/pipeline/zircon-flow.groovy").read_text(encoding="utf-8"))
        java = Path(os.environ.get("JENKINS_GROOVY_JAVA", str(root / ".jenkins/runtime/jdk/bin/java.exe")))
        jars = sorted((root / ".jenkins/cache/war/WEB-INF/lib").glob("groovy-all-*.jar"))
        classpath = os.environ.get("JENKINS_GROOVY_CLASSPATH") or (str(jars[-1]) if jars else None)
        if not java.is_file() or not classpath:
            self.skipTest("Jenkins Java/Groovy runtime unavailable; Pipeline acceptance remains pending")
        harness = GROOVY_HARNESS.split("def root = new File(args[0])", 1)[0] + r'''
def root = new File(args[0])
def identity = [sessionId:'session', requestId:'request', repositoryId:'repo', attemptId:'attempt',
    generation:7, sourceInputDigest:'a' * 64, coverageDigest:'c' * 64, stageImplementationDigest:'d' * 64]
def parameters = [REPOSITORY_ID:'repo', SESSION_ID:'session', REQUEST_ID:'request',
    ATTEMPT_ID:'attempt', GENERATION:'7', CURRENT_SOURCE_REF:'b' * 64,
    BUILD_ROOT:'E:/cargo-targets', ALLOW_COMMIT:'false', ALLOW_NOTIFY:'false']
def response = { Map value, int exitCode = 0 -> [exitCode:exitCode, response:value] }
def prefix = [response([status:'prepared', identity:identity, sealedInputRef:'a' * 64,
        sourcePreparationRef:'current-source-' + ('b' * 64)]),
    response([status:'pending', executionId:'registration-only']),
    response([status:'ready', template:'editor_product_development', stages:['source','build','acceptance']]),
    response([status:'ready'])]
def actual = 'e' * 32
def recipe = 'f' * 64
def cache = response([status:'observed', result:[cleanupRequired:false, gcAuthorized:false, scratch:[status:'protected']]])
for (mode in ['mismatch', 'normal', 'other-failure', 'unknown']) {
    def responses = new ArrayList(prefix)
    responses << (mode == 'normal' ? response([status:'ready',executionId:actual,recipeRef:recipe]) :
        response([status:'failed', reasonCode:mode == 'other-failure' ? 'recipe_unproven' : 'submission_identity_mismatch'],2))
    if (mode == 'normal') responses << response([status:'submitted',executionId:actual,recipeRef:recipe])
    if (mode in ['mismatch','normal']) responses.addAll([
        response([status:'ready',executionId:actual]), response([status:'accepted']),
        response([status:'accepted',notifyEligible:false])])
    if (mode == 'other-failure') responses << response([status:'cancelled'])
    responses << cache
    def fixture = new PipelineFixture(root, responses)
    fixture.binding.getVariable('params').putAll(parameters)
    fixture.binding.getVariable('env').JENKINS_PYTHON = 'managed-python'
    def bridgeCalls = []
    def originalBat = fixture.binding.getVariable('bat')
    fixture.binding.setVariable('withEnv', { List values, Closure body ->
        values = values*.toString()
        assert fixture.nodeDepth == 1
        assert values.contains('ZIRCON_JENKINS_SESSION_ID=session') && values.contains('INPUT_JSON=')
        assert values.any { it.startsWith('XDG_CONFIG_HOME=') }
        body()
    })
    fixture.binding.setVariable('bat', { Map options ->
        if (!options.script.contains('resume_jenkins_flow.py')) return originalBat(options)
        assert fixture.nodeDepth == 1 && options.returnStatus == true
        assert options.script.contains('--expected-bridge-sha256 ')
        def control = new JsonSlurper().parseText(fixture.writes.last().text)
        assert control == [identity:identity, currentSourceRef:'b' * 64]
        bridgeCalls << [query:options.script.contains(' --query-only'),
                       file:fixture.writes.last().file, payload:control]
        fixture.current = mode == 'unknown' ? response([status:'unknown', reasonCode:'delivery_unknown'],2) :
            bridgeCalls.size() == 1 ? response([status:'pending',executionId:actual,recipeRef:recipe]) :
                response([status:'complete',executionId:actual,recipeRef:recipe])
        return fixture.current.exitCode
    })
    if (mode in ['mismatch','normal']) {
        fixture.jobScript('flow').run()
        assert fixture.stages.contains('Await execution receipts') && fixture.stages.contains('Acceptance')
        assert fixture.calls.findAll { it.label == 'Jenkins control flow/observe-flow' }*.payload*.executionId == [actual]
        assert fixture.calls.findAll { it.label == 'Jenkins control flow/run-stage' }*.payload*.stage == ['source','acceptance']
        assert !fixture.calls.any { it.label == 'Jenkins control flow/reconcile-flow' }
        if (mode == 'mismatch') {
            assert bridgeCalls*.query == [false,true] && bridgeCalls[0].file == bridgeCalls[1].file
            assert fixture.sleeps == [2]
            assert !fixture.calls.any { it.label == 'Jenkins control flow/ensure-execution-job' }
        } else {
            assert bridgeCalls.empty && fixture.sleeps.empty
            assert fixture.calls.count { it.label == 'Jenkins control flow/ensure-execution-job' } == 1
        }
    } else {
        def failure = rejected { fixture.jobScript('flow').run() }
        assert !fixture.stages.contains('Acceptance')
        if (mode == 'unknown') {
            assert failure.message.contains('requires reconciliation')
            assert bridgeCalls*.query == [false] && fixture.sleeps.empty
            assert !fixture.calls.any { it.label == 'Jenkins control flow/reconcile-flow' }
        } else {
            assert failure.message.contains('recipe_unproven') && bridgeCalls.empty
            assert fixture.calls.count { it.label == 'Jenkins control flow/reconcile-flow' } == 1
        }
    }
    assert fixture.position == responses.size() && fixture.nodeDepth == 0
    assert fixture.calls.last().label == 'Jenkins control maintenance/post-task-cache'
}
println 'PASS prepared Flow bridge cases=4'
'''
        completed = subprocess.run([str(java), "-cp", classpath, "groovy.ui.GroovyMain",
            "-e", harness, str(root)], cwd=root, capture_output=True, text=True,
            encoding="utf-8", errors="replace", timeout=120)
        self.assertEqual(0, completed.returncode, completed.stdout + completed.stderr)
        self.assertIn("PASS prepared Flow bridge cases=4", completed.stdout)

    @unittest.skipUnless(os.name == "nt", "Real prepared worktree authority requires Windows")
    def test_public_flow_bridge_uses_real_claim_and_preserves_idempotent_delivery(self):
        import copy
        import importlib.util
        from types import SimpleNamespace
        from tools.jenkins.contracts import JenkinsError
        root = Path(__file__).resolve().parents[3]
        def module(name, path):
            spec = importlib.util.spec_from_file_location(name, root / path)
            value = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(value)
            return value
        fixtures = module("default_flow_prepared_fixture",
                          "tools/jenkins-management/tests/test_prepared_worktree_flow.py")
        bridge = module("default_flow_public_bridge", "tools/dev/resume_jenkins_flow.py")
        fixture = fixtures.PreparedWorktreeFlowTests(methodName="runTest")
        with fixture.claimed_prepared_flow() as value:
            f, actor = value.f, value.actor
            posts, calls = [], []
            def transport(path, **arguments):
                posts.append((path, arguments["data"]))
                return 201, b"", {"Location":"http://127.0.0.1:18080/queue/item/21/"}
            actor.manager = SimpleNamespace(base_url="http://127.0.0.1:18080", _request_with_headers=transport)
            context = {"config":actor.config, "module":SimpleNamespace(source_helper=lambda:fixtures.helper)}
            def dispatch(repo, payload, *, query_only):
                self.assertEqual(f.repo, repo)
                calls.append((copy.deepcopy(payload), query_only))
                if query_only:
                    return {"status":"pending", "reasonCode":"original_operation_missing", "resubmitted":False}
                actor.payload = payload
                return {"phase":"complete", "status":"observed", "result":actor._dispatch()}
            entry = SimpleNamespace(capability=lambda repo:context, dispatch=dispatch)
            supplied = {"identity":copy.deepcopy(value.identity), "currentSourceRef":value.reference}
            original = f.state.get_request(f.repo_id, actor.session, value.identity["requestId"])
            typed = f.state.get_operation(value.result["sourcePreparationRef"])
            execution = f.state.get("execution", value.execution)
            intent = f.state.get("current_source_intent", value.reference)["payload"]
            first = bridge.resume_flow(f.repo, supplied, entry_module=entry)
            self.assertEqual("complete", first["status"])
            self.assertEqual(value.execution, first["executionId"])
            self.assertEqual(value.recipe, first["recipeRef"])
            self.assertFalse(first["formalAcceptance"])
            self.assertEqual(intent["identity"], calls[0][0]["flowPayload"]["identity"])
            self.assertEqual(value.value.owned, calls[0][0]["ownedPaths"])
            self.assertEqual(25, len(calls[0][0]["ownedPaths"]))
            self.assertEqual(1, len(posts))
            management_identity = first["managementIdentity"]
            saved_marker = f.state.get("jenkins_submission", value.marker)
            for status in ("submitted", "accepted", "completed"):
                with self.subTest(delivery=status):
                    f.state.put("jenkins_submission", value.marker,
                                {**saved_marker["payload"], "delivery":status})
                    replay = bridge.resume_flow(f.repo, supplied, entry_module=entry)
                    self.assertEqual("complete", replay["status"])
                    self.assertEqual(status, replay["originalDeliveryState"])
                    self.assertEqual(management_identity, replay["managementIdentity"])
                    self.assertFalse(calls[-1][1])
                    self.assertEqual(1, len(posts))
            for status in ("dispatching", "unknown"):
                with self.subTest(delivery=status):
                    f.state.put("jenkins_submission", value.marker,
                                {**saved_marker["payload"], "delivery":status})
                    before = {domain:f.state.get(domain, key) for domain,key in
                        (("workflow",value.key),("execution",value.execution),("jenkins_submission",value.marker))}
                    observed = bridge.resume_flow(f.repo, supplied, entry_module=entry)
                    self.assertEqual("unknown", observed["status"])
                    self.assertEqual(status, observed["originalDeliveryState"])
                    self.assertTrue(calls[-1][1])
                    self.assertEqual(1, len(posts))
                    self.assertEqual(before, {domain:f.state.get(domain, key) for domain,key in
                        (("workflow",value.key),("execution",value.execution),("jenkins_submission",value.marker))})
            for field in ("attemptId", "sourceInputDigest"):
                with self.subTest(identity=field):
                    wrong = copy.deepcopy(supplied)
                    wrong["identity"][field] = "f" * 64
                    count = len(calls)
                    with self.assertRaises(JenkinsError):
                        bridge.resume_flow(f.repo, wrong, entry_module=entry)
                    self.assertEqual(count, len(calls))
            self.assertEqual(original, f.state.get_request(f.repo_id, actor.session, value.identity["requestId"]))
            self.assertEqual(typed, f.state.get_operation(value.result["sourcePreparationRef"]))
            self.assertEqual(execution, f.state.get("execution", value.execution))
            self.assertEqual(1, len(f.state.list("execution_consumer")))
            self.assertEqual(1, len(posts))


if __name__ == "__main__":
    unittest.main()
