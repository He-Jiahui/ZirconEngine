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
    GroovyShell shell
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
        shell = new GroovyShell(getClass().classLoader, binding)
        driver = shell.parse(new File(root, '.jenkins/pipeline/steps.groovy'))
    }

    def jobScript(String job) {
        def source = new File(root, ".jenkins/pipeline/zircon-${job}.groovy").getText('UTF-8')
        // Jenkins assigns a script class name when loading Pipeline code. The
        // plain runner needs a legal logical name for these hyphenated files.
        return shell.parse(source, "zircon_${job}.groovy")
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
    [exitCode:0,response:[status:'ready',template:'comments_only']],
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
        completed = subprocess.run(
            [str(java), "-cp", classpath, "groovy.ui.GroovyMain", "-e", GROOVY_HARNESS, str(root)],
            cwd=root, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=60,
        )
        self.assertEqual(0, completed.returncode, completed.stdout + completed.stderr)
        self.assertIn("PASS canonical Pipeline contention", completed.stdout)


if __name__ == "__main__":
    unittest.main()
