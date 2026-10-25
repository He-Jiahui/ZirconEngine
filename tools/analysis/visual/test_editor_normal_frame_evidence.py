"""Focused validation checks; these do not simulate GPU/Win32 acceptance."""
import copy
import json
import os
import tempfile
import struct
import unittest
import zlib
from zircon_editor_normal_frame_evidence import validate_hold, validate_readback, validate_profile, raw_rgba, digest, decode_own_png, validate_capture_instance, validate_live_request, validate_case_publication, validate_case_contract_documents, validate_capture_method, validate_case_inputs, decode_scene_png, scene_rgba, validate_scene_command, runtime_scene_packet
from pathlib import Path

def fixtures():
    hold={'schema':'dev.zircon.editor.normal-capture-hold','version':1,'authority':'normal-successful-gpu-present','pixelFrameVerified':False,'status':'held','nonce':'test','pid':7,'caseSha256':'a'*64,'physicalSize':[1,1],'presentationCursor':'actual-test-cursor','presentedFrameCount':4,'expiresAtUnixMs':1000,'acquiredAtUnixMs':10}
    ticket={'device_id':1,'generation':2,'queue_class':'Graphics','sequence':3}
    value={'schema':'dev.zircon.editor.normal-frame-readback','version':1,'authority':'normal-actual-gpu-packet-and-readback-receipt','gpuReadbackCompleted':True,'failure':None,'pixelFrameVerified':False,'hwnd':9,'surfaceEpoch':1,'nativePresentIndex':3,'colorEncoding':'srgb','opaquePixels':True,'rawByteLength':4,'submission':ticket,
           'receipt':{'request':8,'kind':'Texture','byte_len':256,'frame_key':{'device_id':1,'generation':2,'submission':copy.deepcopy(ticket)},'terminal':'Succeeded'}}
    value.update({key:hold[key] for key in ('nonce','pid','caseSha256','physicalSize','presentationCursor','presentedFrameCount','expiresAtUnixMs')})
    profile={'presenter_backend':'gpu','winit_scale_factor':1.0,'window_client_size':{'width':1,'height':1},'capture_sequence':5,
             'normal_window_evidence':{'captureHold':copy.deepcopy(hold),'presentedFrameCount':4,'commandStreamFullRebuild':True},
             'submitted_gpu_text':{'presentedFrameCount':4,'commandStreamFullRebuild':True,'coverage':'submitted-command-stream'}}
    return hold,value,profile

class NormalFrameValidation(unittest.TestCase):
    def test_observed_business_is_stable_across_actual_frame_count_change(self):
        _, _, profile = fixtures()
        counters = {key: 1 for key in ('structure','geometry','interaction','viewport','hitTest','theme','diagnostics')}
        dto = {'stateFingerprint': 'a'*64, 'activeLocale': 'en', 'selected': False}
        managed = {'projectRoot': 'D:/actual', 'sources': []}
        publication = {'authority':'normal-retained-workbench-projection', 'observed':True,
                       'presentationCursor':counters, 'workbenchPresentation':dto,
                       'managedSceneFingerprint':managed}
        case = {'data':{'workbenchPresentation':copy.deepcopy(dto), 'managedSceneFingerprint':copy.deepcopy(managed),
                        'normal_window_evidence':{'workbenchStatePublication':copy.deepcopy(publication)}}}
        normal = profile['normal_window_evidence']
        normal.update(presentationCursor=copy.deepcopy(counters),captureSequence=5,
                      workbenchStatePublication={**copy.deepcopy(publication),'presentedFrameCount':4})
        validate_case_publication(profile,case)
        normal['presentedFrameCount']=9;normal['workbenchStatePublication']['presentedFrameCount']=9
        profile['submitted_gpu_text']['presentedFrameCount']=9
        validate_case_publication(profile,case)
        for key in counters:
            changed=copy.deepcopy(profile);changed['normal_window_evidence']['workbenchStatePublication']['presentationCursor'][key]=2
            with self.assertRaises(ValueError):validate_case_publication(changed,case)
        changed=copy.deepcopy(profile);changed['normal_window_evidence']['workbenchStatePublication']['workbenchPresentation']['selected']=0
        with self.assertRaises(ValueError):validate_case_publication(changed,case)
        changed=copy.deepcopy(profile);changed['normal_window_evidence']['workbenchStatePublication']['presentedFrameCount']=4
        with self.assertRaises(ValueError):validate_case_publication(changed,case)
        changed=copy.deepcopy(case);changed['data']['normal_window_evidence']['workbenchStatePublication']['presentedFrameCount']=4
        with self.assertRaises(ValueError):validate_case_publication(profile,changed)
        changed=copy.deepcopy(case);changed['data']['normal_window_evidence']['workbenchStatePublication']['presentationCursor']['theme']=True
        with self.assertRaises(ValueError):validate_case_publication(profile,changed)
        normal['focusPublication']={'authority':'normal-native-pane-interaction','observed':True,'focusedNativeControlId':None}
        with self.assertRaises(ValueError):validate_case_publication(profile,case)
        case['data']['normal_window_evidence']['focusPublication']=copy.deepcopy(normal['focusPublication'])
        validate_case_publication(profile,case)
        changed=copy.deepcopy(case);changed['data']['normal_window_evidence']['captureSequence']=5
        with self.assertRaises(ValueError):validate_case_publication(profile,changed)
        normal.pop('focusPublication')
        with self.assertRaises(ValueError):validate_case_publication(profile,case)


    @unittest.skipUnless(os.name == "nt", "Win32 extended-length paths require Windows")
    def test_actual_managed_project_matches_physical_identity_across_win32_spellings(self):
        approved = Path("D:/cargo-targets/zircon-local").resolve(strict=True)
        scratch = Path(tempfile.mkdtemp(prefix="normal-project-identity-", dir=approved))
        self.assertTrue(scratch.resolve(strict=True).is_relative_to(approved))
        repo = scratch / "unit-repo"
        project = scratch / "actual-project"
        other = scratch / "other-project"
        other.mkdir()
        owner = "zircon_editor/assets/ui/editor/unreal/level_editor_window.zui"
        source = repo / owner
        source.parent.mkdir(parents=True)
        source.write_bytes(b"unit source identity")
        sources = []
        for relative in ("zircon-project.toml", "assets/scenes/main.scene.toml",
                         "assets/scenes/workbench-review-empty.scene.toml"):
            path = project / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            data = ("unit input " + relative).encode("utf-8")
            path.write_bytes(data)
            sources.append({"sourcePath": relative, "sha256": digest(data)})
        publication = {"workbenchPresentation": {"sourceFingerprint":
            {"sourcePath": owner, "sha256": digest(source.read_bytes())}},
            "managedSceneFingerprint": {"projectRoot": str(project.resolve(strict=True)),
                                        "sources": sources}}
        case = {"sourcePath": owner, "data": {"normal_window_evidence":
            {"workbenchStatePublication": publication}}}
        ordinary = project.resolve(strict=True)
        extended = Path("\\\\?\\" + str(ordinary))
        self.assertTrue(extended.samefile(ordinary))
        for serialized, launched in ((ordinary, ordinary), (extended, ordinary),
                                     (ordinary, extended), (extended, extended)):
            publication["managedSceneFingerprint"]["projectRoot"] = str(serialized)
            validate_case_inputs(case, repo, launched)
            with self.assertRaisesRegex(ValueError, "differs from this official Editor"):
                validate_case_inputs(case, repo, other)
        (project / "assets/scenes/main.scene.toml").write_bytes(b"changed after capture")
        with self.assertRaisesRegex(ValueError, "changed after case preparation"):
            validate_case_inputs(case, repo, ordinary)

    def test_actual_request_deadline_is_not_extended_by_hold_acquisition(self):
        request={'requestedAtUnixMs':100,'deadlineMs':50,'release':False,'gpuReadback':True}
        validate_live_request(request,request,150)
        for now in (99,151):
            with self.assertRaises(ValueError):validate_live_request(request,request,now)
        for key,bad in (('deadlineMs',True),('requestedAtUnixMs',True),('deadlineMs',49),('deadlineMs',10001)):
            changed={**request,key:bad}
            with self.assertRaises(ValueError):validate_live_request(changed,changed,100)
        with self.assertRaises(ValueError):validate_live_request({**request,'release':True},request,100)
    def test_native_texture_row_padding_and_accounted_alignment_gap(self):
        hold,value,_=fixtures()
        # Actual native service reserves each texture row at 256 bytes and the
        # tracker accounts the gap from the previous request to its aligned start.
        # Its existing RHI test admits buffer4 then texture1x1: 256+252 = 508.
        for width,height,gap in ((1,1,0),(1,1,252),(1350,2,0),(1350,2,252)):
            hold['physicalSize']=[width,height];value['physicalSize']=[width,height]
            value['rawByteLength']=width*height*4
            padded=((width*4+255)//256)*256*height
            value['receipt']['byte_len']=padded+gap
            validate_readback(value,hold,9,(width,height))
            for invalid in (value['rawByteLength'] if value['rawByteLength']!=padded else padded-1, padded-1, padded+256):
                value['receipt']['byte_len']=invalid
                with self.assertRaises(ValueError):validate_readback(value,hold,9,(width,height))
            value['receipt']['byte_len']=padded+gap
    def test_other_instance_and_missing_instance_are_rejected(self):
        proof={'pid':7,'hwnd':9}
        for capture in ({}, {'process_id':8,'window_handle':9}, {'process_id':7,'window_handle':True}):
            with self.assertRaises(ValueError):validate_capture_instance(proof,capture,{},Path('unused'))
    def test_typed_actual_join(self):
        hold,value,profile=fixtures()
        validate_hold(hold,'test',7,'a'*64,(1,1),20)
        validate_readback(value,hold,9,(1,1));validate_profile(profile,hold,(1,1),1.0)
    def test_missing_and_bool_ticket_rejected(self):
        hold,value,_=fixtures()
        for changed in (None, {'device_id':True,'generation':2,'queue_class':'Graphics','sequence':3}):
            value['submission']=changed;value['receipt']['frame_key']['submission']=changed
            with self.assertRaises(ValueError):validate_readback(value,hold,9,(1,1))
    def test_unknown_queue_and_bool_present_count_rejected(self):
        hold,value,profile=fixtures()
        value['submission']['queue_class']='InventedQueue'
        value['receipt']['frame_key']['submission']['queue_class']='InventedQueue'
        with self.assertRaises(ValueError):validate_readback(value,hold,9,(1,1))
        hold['presentedFrameCount']=1
        profile['normal_window_evidence']['captureHold']=hold
        profile['normal_window_evidence']['presentedFrameCount']=True
        profile['submitted_gpu_text']['presentedFrameCount']=1
        with self.assertRaises(ValueError):validate_profile(profile,hold,(1,1),1.0)
    def test_receipt_device_and_byte_extent_rejected(self):
        hold,value,_=fixtures()
        value['receipt']['frame_key']['generation']=9
        with self.assertRaises(ValueError):validate_readback(value,hold,9,(1,1))
        value['receipt']['frame_key']['generation']=2;value['receipt']['byte_len']=True
        with self.assertRaises(ValueError):validate_readback(value,hold,9,(1,1))
    def test_hold_expiry_cursor_and_retained_stream_rejected(self):
        hold,value,profile=fixtures()
        with self.assertRaises(ValueError):validate_hold(hold,'test',7,'a'*64,(1,1),1000)
        profile['normal_window_evidence']['captureHold']['presentationCursor']='changed'
        with self.assertRaises(ValueError):validate_profile(profile,hold,(1,1),1.0)
        profile['normal_window_evidence']['captureHold']=hold;profile['submitted_gpu_text']['commandStreamFullRebuild']=False
        with self.assertRaises(ValueError):validate_profile(profile,hold,(1,1),1.0)
    def test_focus_publication_is_actual_and_unknown_not_false(self):
        hold,_,profile=fixtures()
        validate_profile(profile,hold,(1,1),1.0,None)
        with self.assertRaises(ValueError):validate_profile(profile,hold,(1,1),1.0,{'observed':True})
    def test_raw_alpha_and_lossless_bgra(self):
        _,value,_=fixtures();raw=bytes((1,2,3,255));value.update(rawSha256=digest(raw),sourceFormat='Bgra8UnormSrgb',sourceByteLayout='bgra8')
        self.assertEqual(raw_rgba(value,raw,(1,1)),bytes((3,2,1,255)))
        altered=bytes((1,2,3,0));value['rawSha256']=digest(altered)
        with self.assertRaises(ValueError):raw_rgba(value,altered,(1,1))
    def test_png_exact_crc_extent_and_pixel_decode(self):
        def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
        pixel=bytes((1,2,3,255));png=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',1,1,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(b'\0'+pixel))+chunk(b'IEND',b'')
        self.assertEqual(decode_own_png(png,(1,1)),pixel)
        with self.assertRaises(ValueError):decode_own_png(png,(2,1))
        changed=bytearray(png);changed[-5]^=1
        with self.assertRaises(ValueError):decode_own_png(bytes(changed),(1,1))

class CaseContractValidation(unittest.TestCase):
    def contract(self):
        root = Path("E:/Git/ZirconEngine")
        canonical = root / "tools/penpot/apps/zircon-zui-plugin"
        config = {"schema":"dev.zircon.editor.normal-frame-case-producer", "version":1}
        for label in ("node", "producer", "contract", "contractInputs"):
            config[label] = {"path":str(Path("D:/cargo-targets/zircon-local") / label),
                             "sha256":"a"*64}
        audit = {"schema":"dev.zircon.editor.normal-workbench-contract-inputs", "version":1,
                 "repoRoot":str(root), "canonicalAppRoot":str(canonical),
                 "sourceCopiesCreated":False,
                 "outputPath":config["contract"]["path"], "outputSha256":"a"*64,
                 "inputs":[{"path":str(canonical / ("tools/" + name + ".ts")), "sha256":"b"*64}
                           for name in ("zui-layout-review-contract", "zui-layout-workbench-presentation",
                                        "zui-layout-workbench-managed-inputs", "zui-layout-component-hosts")]}
        return root, config, audit

    def test_requires_current_canonical_owners_and_no_source_copies(self):
        root, config, audit = self.contract()
        validate_case_contract_documents(config, audit, root)
        for key, value in (("canonicalAppRoot", str(root / "dev/penpot/plugins/apps/zircon-zui-plugin")),
                           ("repoRoot", "D:/old-source-copy"), ("sourceCopiesCreated", True),
                           ("outputSha256", "c"*64)):
            bad = copy.deepcopy(audit); bad[key] = value
            with self.assertRaises(ValueError):
                validate_case_contract_documents(config, bad, root)
        for index in range(len(audit["inputs"])):
            bad = copy.deepcopy(audit); bad["inputs"].pop(index)
            with self.assertRaises(ValueError):
                validate_case_contract_documents(config, bad, root)

    def test_accepts_compiler_helpers_and_pinned_toml_dependencies_but_rejects_reference_modules(self):
        root, config, audit = self.contract()
        support = [root / "tools/analysis/visual" / name for name in
                   ("build-normal-workbench-contracts.mjs", "normal-workbench-case-producer.mjs")]
        toml_root = Path("D:/cargo-targets/zircon-local/penpot-public/zircon-plugin/workspace/tools/penpot/node_modules/.pnpm/smol-toml@1.8.0/node_modules/smol-toml/dist")
        support.extend(toml_root / (name + ".js") for name in
                       ("date", "error", "extract", "index", "parse", "primitive", "stringify", "struct", "util"))
        audit["inputs"].extend({"path": str(path), "sha256": "b" * 64} for path in support)
        validate_case_contract_documents(config, audit, root)
        for foreign in (root / "dev/penpot/plugins/apps/zircon-zui-plugin/tools/zui-layout-review-contract.ts",
                        root / "tools/analysis/visual/unreviewed-helper.mjs",
                        root / "tools/penpot/old.ts", toml_root / "unexpected.js"):
            bad = copy.deepcopy(audit)
            bad["inputs"].append({"path": str(foreign), "sha256": "b" * 64})
            with self.assertRaises(ValueError):
                validate_case_contract_documents(config, bad, root)

    def test_rejects_duplicate_foreign_and_unpinned_inputs(self):
        root, config, audit = self.contract()
        for rows in (audit["inputs"] + [copy.deepcopy(audit["inputs"][0])],
                     [{**audit["inputs"][0], "path":str(root / "tools/penpot/old.ts")},
                      *audit["inputs"][1:]],
                     [{**audit["inputs"][0], "sha256":"invalid"}, *audit["inputs"][1:]]):
            bad = copy.deepcopy(audit); bad["inputs"] = rows
            with self.assertRaises(ValueError):
                validate_case_contract_documents(config, bad, root)
        bad = copy.deepcopy(config); bad["contract"]["sha256"] = "x"*64
        with self.assertRaises(ValueError):
            validate_case_contract_documents(bad, audit, root)

    def test_only_actual_win32_methods_are_eligible_for_pixel_join(self):
        for method in ("win32-desktop-bitblt-gdiflush-own-client",
                       "win32-printwindow-client-renderfullcontent-gdiflush"):
            validate_capture_method(method)
        for method in (None, True, "gpu-readback-only", "synthetic-preview", "other-window"):
            with self.assertRaises(ValueError):
                validate_capture_method(method)


class CanonicalCasePublicationValidation(unittest.TestCase):
    def case_and_held_frame(self):
        # Full toolbar schema/provenance is checked by the canonical producer.
        # These fixtures isolate immutable seed-to-held joins, not GPU acceptance.
        hold, readback, profile = fixtures()
        counters = {key: 1 for key in
                    ('structure', 'geometry', 'interaction', 'viewport', 'hitTest', 'theme', 'diagnostics')}
        dto = {'stateFingerprint': 'a' * 64, 'activeLocale': 'en', 'selected': False}
        managed = {'projectRoot': 'D:/actual', 'sources': []}
        business = {'authority': 'normal-retained-workbench-projection', 'observed': True,
                    'presentationCursor': copy.deepcopy(counters), 'presentedFrameCount': 4,
                    'workbenchPresentation': copy.deepcopy(dto), 'managedSceneFingerprint': copy.deepcopy(managed)}
        node = {'nodeId': 'choice', 'parentNodeId': 'row',
                'sourcePath': 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui',
                'sourceNodeId': 'Perspective', 'instancePath': '[]',
                'parentSourcePath': 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui',
                'parentSourceNodeId': 'ToolbarRow', 'parentInstancePath': '[]',
                'controlId': 'perspective', 'component': 'Dropdown', 'visible': True,
                'bounds': {'x': 0, 'y': 0, 'width': 100, 'height': 28},
                'clip': {'x': 0, 'y': 0, 'width': 100, 'height': 28},
                'state': {'selected': None, 'checked': None, 'popupOpen': False,
                          'value': 'perspective', 'text': None, 'valueText': 'Perspective', 'disabled': False},
                'scroll': None, 'desiredSize': {'width': 100, 'height': 28, 'coordinateSpace': 'logical-pixels'}}
        toolbar = {'schema': 'dev.zircon.editor.viewport-toolbar-publication', 'version': 1,
                   'authority': 'normal-prepared-source-surface', 'observed': True,
                   'coordinateSpace': 'physical-pixels', 'surfaceGeneration': 1, 'surfaceTreeId': 'test-surface',
                   'rootNodeId': 'root', 'rowNodeId': 'row', 'moreNodeId': 'more', 'popupNodeId': 'popup',
                   'groupNodeIds': ['group'], 'nodes': [node],
                   'presentationCursor': copy.deepcopy(counters), 'presentedFrameCount': 4}
        seed = {'workbenchStatePublication': business, 'presentationCursor': copy.deepcopy(counters),
                'presentedFrameCount': 4, 'viewportToolbarPublication': toolbar}
        case = {'data': {'workbenchPresentation': dto, 'managedSceneFingerprint': managed,
                         'normal_window_evidence': copy.deepcopy(seed)}}
        hold['presentedFrameCount'] = 9
        readback['presentedFrameCount'] = 9
        normal = profile['normal_window_evidence']
        normal.update(copy.deepcopy(seed))
        normal['captureSequence'] = 5
        normal['captureHold'] = copy.deepcopy(hold)
        normal['presentedFrameCount'] = 9
        normal['workbenchStatePublication']['presentedFrameCount'] = 9
        normal['viewportToolbarPublication']['presentedFrameCount'] = 9
        profile['submitted_gpu_text']['presentedFrameCount'] = 9
        return hold, readback, profile, case

    def test_full_seed_allows_later_matching_held_frame(self):
        hold, readback, profile, case = self.case_and_held_frame()
        validate_case_publication(profile, case)
        validate_hold(hold, 'test', 7, 'a' * 64, (1, 1), 20)
        validate_readback(readback, hold, 9, (1, 1))
        validate_profile(profile, hold, (1, 1), 1.0)

    def test_optional_actual_commands_are_preserved_and_joined_to_the_held_profile(self):
        _, _, profile, case = self.case_and_held_frame()
        commands = [{'commandIndex': 124, 'source': {
            'sourceRef': {'authority': 'surface-node-owner', 'primitiveIdentity': False,
                'treeId': 'template.v2.res://ui/editor/unreal/level_editor_window.zui',
                'generation': 11, 'nodeId': 30, 'nodeCommandIndex': 0, 'fragmentIndex': 0}},
            'paint': {'kind': 'image', 'generation': 1}}]
        case['data']['normal_window_evidence']['commands'] = copy.deepcopy(commands)
        profile['normal_window_evidence']['commands'] = copy.deepcopy(commands)
        before = copy.deepcopy((profile, case))
        validate_case_publication(profile, case)
        self.assertEqual((profile, case), before)
        for kind in ('missing', 'generation rollback', 'wrong tree', 'wrong command index', 'different JSON type'):
            changed = copy.deepcopy(profile)
            actual = changed['normal_window_evidence']
            if kind == 'missing': actual.pop('commands')
            if kind == 'generation rollback': actual['commands'][0]['source']['sourceRef']['generation'] = 10
            if kind == 'wrong tree': actual['commands'][0]['source']['sourceRef']['treeId'] = 'template.v2.other'
            if kind == 'wrong command index': actual['commands'][0]['commandIndex'] = 125
            if kind == 'different JSON type': actual['commands'][0]['paint']['generation'] = True
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                validate_case_publication(changed, case)
        for invalid in (None, [None], [{'commandIndex': True}], [{'commandIndex': -1}],
                        [{'commandIndex': 2**53}], [commands[0], copy.deepcopy(commands[0])]):
            changed_profile, changed_case = copy.deepcopy(profile), copy.deepcopy(case)
            changed_profile['normal_window_evidence']['commands'] = copy.deepcopy(invalid)
            changed_case['data']['normal_window_evidence']['commands'] = copy.deepcopy(invalid)
            with self.subTest(commands=invalid), self.assertRaises(ValueError):
                validate_case_publication(changed_profile, changed_case)

    def test_legacy_case_keeps_its_exact_keys_even_when_semantics_match(self):
        _, _, profile, case = self.case_and_held_frame()
        seed = case['data']['normal_window_evidence']
        for key in ('presentationCursor', 'presentedFrameCount', 'viewportToolbarPublication'):
            seed.pop(key)
        seed['workbenchStatePublication'].pop('presentedFrameCount')
        validate_case_publication(profile, case)
        semantics = [{'nodeId': 'choice', 'visible': True}]
        seed['semanticNodes'] = copy.deepcopy(semantics)
        profile['normal_window_evidence']['semanticNodes'] = copy.deepcopy(semantics)
        with self.assertRaises(ValueError):
            validate_case_publication(profile, case)
        seed.pop('semanticNodes')
        profile['normal_window_evidence'].pop('semanticNodes')
        seed['commands'] = []
        profile['normal_window_evidence']['commands'] = []
        with self.assertRaises(ValueError):
            validate_case_publication(profile, case)

    def test_partial_and_unknown_canonical_shapes_are_rejected(self):
        _, _, profile, case = self.case_and_held_frame()
        seed = case['data']['normal_window_evidence']
        for key in ('workbenchStatePublication', 'presentationCursor', 'presentedFrameCount', 'viewportToolbarPublication'):
            with self.subTest(missing=key):
                changed = copy.deepcopy(case)
                changed['data']['normal_window_evidence'].pop(key)
                with self.assertRaises(ValueError):
                    validate_case_publication(profile, changed)
        for scope in ('normal', 'business'):
            with self.subTest(extra=scope):
                changed = copy.deepcopy(case)
                target = changed['data']['normal_window_evidence']
                if scope == 'business':
                    target = target['workbenchStatePublication']
                target['captureSequence'] = 5
                with self.assertRaises(ValueError):
                    validate_case_publication(profile, changed)
        self.assertEqual(seed['presentedFrameCount'], 4)

    def test_all_seven_seed_and_held_cursor_joins_are_required(self):
        _, _, profile, case = self.case_and_held_frame()
        for location in ('outer', 'workbenchStatePublication', 'viewportToolbarPublication'):
            for counter in case['data']['normal_window_evidence']['presentationCursor']:
                with self.subTest(location=location, counter=counter):
                    changed = copy.deepcopy(case)
                    target = changed['data']['normal_window_evidence']
                    if location != 'outer':
                        target = target[location]
                    target['presentationCursor'][counter] = 2
                    with self.assertRaises(ValueError):
                        validate_case_publication(profile, changed)
                    changed_profile = copy.deepcopy(profile)
                    actual = changed_profile['normal_window_evidence']
                    if location != 'outer':
                        actual = actual[location]
                    actual['presentationCursor'][counter] = 2
                    with self.assertRaises(ValueError):
                        validate_case_publication(changed_profile, case)

    def test_seed_counts_are_safe_positive_and_held_toolbar_is_current(self):
        _, _, profile, case = self.case_and_held_frame()
        for location in ('outer', 'workbenchStatePublication', 'viewportToolbarPublication'):
            for count in (0, -1, True, 5, 2**53):
                with self.subTest(location=location, count=count):
                    changed = copy.deepcopy(case)
                    target = changed['data']['normal_window_evidence']
                    if location != 'outer':
                        target = target[location]
                    target['presentedFrameCount'] = count
                    with self.assertRaises(ValueError):
                        validate_case_publication(profile, changed)
        for count in (4, 10, True):
            changed = copy.deepcopy(profile)
            changed['normal_window_evidence']['viewportToolbarPublication']['presentedFrameCount'] = count
            with self.assertRaises(ValueError):
                validate_case_publication(changed, case)

    def test_full_toolbar_source_geometry_and_state_remain_immutable(self):
        _, _, profile, case = self.case_and_held_frame()
        for path, value in ((('sourcePath',), 'other.zui'), (('parentSourceNodeId',), 'other-parent'),
                            (('state', 'popupOpen'), True), (('state', 'disabled'), 0),
                            (('bounds', 'x'), 1), (('clip', 'width'), 99),
                            (('desiredSize', 'width'), 99)):
            with self.subTest(path=path):
                changed = copy.deepcopy(profile)
                target = changed['normal_window_evidence']['viewportToolbarPublication']['nodes'][0]
                for key in path[:-1]:
                    target = target[key]
                target[path[-1]] = value
                with self.assertRaises(ValueError):
                    validate_case_publication(changed, case)
        changed = copy.deepcopy(profile)
        changed['normal_window_evidence']['viewportToolbarPublication']['unknown'] = True
        with self.assertRaises(ValueError):
            validate_case_publication(changed, case)

    def test_optional_semantics_and_focus_preserve_presence_and_json_identity(self):
        _, _, profile, case = self.case_and_held_frame()
        for key, value in (('semanticNodes', [{'nodeId': 'choice', 'visible': True}]),
                           ('focusPublication', {'authority': 'normal-native-pane-interaction',
                                                 'observed': True, 'focusedNativeControlId': None})):
            with self.subTest(key=key):
                bound_case = copy.deepcopy(case)
                bound_profile = copy.deepcopy(profile)
                bound_case['data']['normal_window_evidence'][key] = copy.deepcopy(value)
                bound_profile['normal_window_evidence'][key] = copy.deepcopy(value)
                validate_case_publication(bound_profile, bound_case)
                with self.assertRaises(ValueError):
                    validate_case_publication(profile, bound_case)
                with self.assertRaises(ValueError):
                    validate_case_publication(bound_profile, case)
                target = bound_profile['normal_window_evidence'][key]
                if isinstance(target, list):
                    target[0]['visible'] = 1
                else:
                    target['focusedNativeControlId'] = 'choice'
                with self.assertRaises(ValueError):
                    validate_case_publication(bound_profile, bound_case)


    def case_with_current_publications(self):
        hold, readback, profile, case = self.case_and_held_frame()
        seed = case['data']['normal_window_evidence']
        normal = profile['normal_window_evidence']
        token = {'schema': 'dev.zircon.editor.host-paint-token-input', 'version': 1,
                 'authority': 'host-paint-theme-snapshot', 'observed': True,
                 'themeGeneration': 27, 'settingsGeneration': None, 'scaleFactor': 1.25,
                 'tokens': {'surface': [0.1, 0.1, 0.1, 1]}}
        inventory = {'schema': 'dev.zircon.editor.normal-arranged-inventory', 'version': 1,
                     'authority': 'normal-retained-runtime-surfaces', 'observed': True,
                     'inventoryAudit': {'complete': True, 'surfaceCount': 1},
                     'presentationCursor': copy.deepcopy(seed['presentationCursor']),
                     'presentedFrameCount': seed['presentedFrameCount'],
                     'complete': False, 'pending': ['mounted-pane-source-style-overlay-closure'],
                     'surfaces': [{'id': 'main', 'presentedFrameCount': 2, 'observed': True}]}
        seed['designTokenInput'] = copy.deepcopy(token)
        normal['paintEvidence'] = {'designTokenInput': copy.deepcopy(token)}
        seed['arrangedInventoryPublication'] = copy.deepcopy(inventory)
        normal['arrangedInventoryPublication'] = copy.deepcopy(inventory)
        normal['arrangedInventoryPublication']['presentedFrameCount'] = normal['presentedFrameCount']
        return hold, readback, profile, case

    def test_current_publications_allow_later_matching_frame_and_keep_pending(self):
        _, _, profile, case = self.case_with_current_publications()
        original_profile, original_case = copy.deepcopy(profile), copy.deepcopy(case)
        validate_case_publication(profile, case)
        self.assertEqual(profile, original_profile)
        self.assertEqual(case, original_case)
        inventory = case['data']['normal_window_evidence']['arrangedInventoryPublication']
        self.assertIs(inventory['complete'], False)
        self.assertEqual(inventory['pending'], ['mounted-pane-source-style-overlay-closure'])

    def test_current_publications_preserve_presence_and_paired_null(self):
        _, _, profile, case = self.case_with_current_publications()
        for key in ('designTokenInput', 'arrangedInventoryPublication'):
            for missing in ('seed', 'actual'):
                changed_profile, changed_case = copy.deepcopy(profile), copy.deepcopy(case)
                seed = changed_case['data']['normal_window_evidence']
                actual = changed_profile['normal_window_evidence']
                owner = actual['paintEvidence'] if key == 'designTokenInput' else actual
                (seed if missing == 'seed' else owner).pop(key)
                with self.subTest(key=key, missing=missing), self.assertRaisesRegex(ValueError, 'presence differs'):
                    validate_case_publication(changed_profile, changed_case)
            paired_profile, paired_case = copy.deepcopy(profile), copy.deepcopy(case)
            paired_case['data']['normal_window_evidence'][key] = None
            actual = paired_profile['normal_window_evidence']
            owner = actual['paintEvidence'] if key == 'designTokenInput' else actual
            owner[key] = None
            validate_case_publication(paired_profile, paired_case)
            with self.assertRaises(ValueError):
                validate_case_publication(profile, paired_case)

    def test_token_input_matches_full_typed_payload_from_paint_owner(self):
        _, _, profile, case = self.case_with_current_publications()
        for key, value in (('settingsGeneration', 0), ('themeGeneration', True),
                           ('observed', 1), ('tokens', {'surface': [0.1, 0.1, 0.1, True]})):
            changed = copy.deepcopy(profile)
            changed['normal_window_evidence']['paintEvidence']['designTokenInput'][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_case_publication(changed, case)
        changed = copy.deepcopy(profile)
        changed['normal_window_evidence']['designTokenInput'] = changed['normal_window_evidence']['paintEvidence'].pop('designTokenInput')
        with self.assertRaisesRegex(ValueError, 'presence differs'):
            validate_case_publication(changed, case)

    def test_inventory_requires_each_own_safe_frame_count_and_cursor(self):
        _, _, profile, case = self.case_with_current_publications()
        for side in ('seed', 'actual'):
            for count in (0, -1, True, 4.0, 2**53, 8):
                changed_profile, changed_case = copy.deepcopy(profile), copy.deepcopy(case)
                owner = (changed_case['data'] if side == 'seed' else changed_profile)['normal_window_evidence']
                owner['arrangedInventoryPublication']['presentedFrameCount'] = count
                with self.subTest(side=side, count=count), self.assertRaises(ValueError):
                    validate_case_publication(changed_profile, changed_case)
            for key in case['data']['normal_window_evidence']['presentationCursor']:
                changed_profile, changed_case = copy.deepcopy(profile), copy.deepcopy(case)
                owner = (changed_case['data'] if side == 'seed' else changed_profile)['normal_window_evidence']
                owner['arrangedInventoryPublication']['presentationCursor'][key] = True
                with self.subTest(side=side, counter=key), self.assertRaises(ValueError):
                    validate_case_publication(changed_profile, changed_case)

    def test_inventory_exempts_only_top_frame_count_and_preserves_unknown_rows(self):
        _, _, profile, case = self.case_with_current_publications()
        for key, value in (('complete', 0), ('pending', []), ('surfaces', []), ('unknown', True)):
            changed = copy.deepcopy(profile)
            changed['normal_window_evidence']['arrangedInventoryPublication'][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_case_publication(changed, case)
        changed = copy.deepcopy(profile)
        changed['normal_window_evidence']['arrangedInventoryPublication']['surfaces'][0]['presentedFrameCount'] = 3
        with self.assertRaises(ValueError):
            validate_case_publication(changed, case)

    def test_current_fields_cannot_bypass_canonical_seed_shape(self):
        for key in ('designTokenInput', 'arrangedInventoryPublication'):
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_case_publication({}, {'data': {'normal_window_evidence': {key: None}}})
        _, _, profile, case = self.case_with_current_publications()
        seed = case['data']['normal_window_evidence']
        for key in ('presentationCursor', 'presentedFrameCount', 'viewportToolbarPublication',
                    'designTokenInput', 'arrangedInventoryPublication'):
            seed.pop(key)
        seed['workbenchStatePublication'].pop('presentedFrameCount')
        validate_case_publication(profile, case)

class RuntimeSceneMediaValidation(unittest.TestCase):
    """Reduced unit packets exercise joins; none is an actual GPU receipt."""
    @staticmethod
    def png(pixels, width, height, filters, split=True):
        def chunk(kind, body):
            return struct.pack('>I', len(body)) + kind + body + struct.pack('>I', zlib.crc32(kind + body) & 0xffffffff)
        stride, rows = width * 4, bytearray()
        for row in range(height):
            method = filters[row]; rows.append(method)
            for x in range(stride):
                left = pixels[row * stride + x - 4] if x >= 4 else 0
                up = pixels[(row - 1) * stride + x] if row else 0
                diagonal = pixels[(row - 1) * stride + x - 4] if row and x >= 4 else 0
                predictors = [0, left, up, (left + up) // 2]
                estimate = left + up - diagonal
                distances = [abs(estimate - left), abs(estimate - up), abs(estimate - diagonal)]
                predictors.append([left, up, diagonal][distances.index(min(distances))])
                rows.append((pixels[row * stride + x] - predictors[method]) & 255)
        compressed = zlib.compress(rows)
        ids = [compressed[:len(compressed)//2], compressed[len(compressed)//2:]] if split else [compressed]
        return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)) +
                chunk(b'tEXt', b'Unit\0official PNG filter coverage') + b''.join(chunk(b'IDAT', body) for body in ids) + chunk(b'IEND', b''))

    @staticmethod
    def command_fixture():
        invocation = '[{"sourcePath":"res://ui/editor/unreal/level_editor_window.zui","sourceNodeId":"viewport"}]'
        owner = {'sourcePath':'res://ui/editor/unreal/components/viewport.zui','sourceNodeId':'scene_media','instancePath':invocation}
        viewport = {**owner,'sourceNodeId':'viewport'}
        viewport_root = {**owner,'sourceNodeId':'root'}
        reference = {'authority':'surface-node-owner','primitiveIdentity':False,
                     'treeId':'template.v2.res://ui/editor/unreal/level_editor_window.zui','generation':11,
                     'nodeId':3,'nodeCommandIndex':0,'fragmentIndex':0}
        bounds = {'x':0,'y':50,'width':2,'height':1}
        source = {**owner,'sourceRef':reference,'parentNodeId':'viewport',
                  'controlId':'WorkbenchViewportSurface','parentSourcePath':viewport['sourcePath'],
                  'parentSourceNodeId':viewport['sourceNodeId'],'parentInstancePath':invocation}
        command = {'commandIndex':4,'bounds':bounds,'clip':bounds,'source':source,
                   'paint':{'kind':'image','resourceKey':'viewport:unit','generation':1,'width':2,'height':1,
                            'residentAfterSuccessfulPresent':True,'hasUploadBytes':False}}
        # Observed R57 brush identities/indices in reduced unit packets, not GPU proof.
        dock_images = [{**command, 'commandIndex': index, 'source': {
            'sourcePath': None, 'sourceNodeId': None, 'instancePath': None, 'controlId': 'DockTab' + str(order % 3),
            'sourceRef': {'authority': 'surface-node-owner', 'primitiveIdentity': False, 'treeId': 'host.bottom.dock.header',
                          'generation': 3, 'nodeId': node_id, 'nodeCommandIndex': node_command, 'fragmentIndex': fragment}}}
            for order, (index, node_id, node_command, fragment) in enumerate([
                (78,6,0,0),(81,9,0,0),(84,12,0,0),(94,6,1,1),(98,9,1,1),(102,12,1,1),(104,6,2,0),(106,9,2,0),(108,12,2,0)])]
        source_null_images = [{**command, 'commandIndex': index, 'source': None} for index in (284,286,288)]
        scene = {'schema':'dev.zircon.editor.normal-scene-image-readback','version':1,
                 'authority':'normal-actual-submitted-runtime-image-readback','owner':owner,'viewportRoot':viewport_root,
                 'paneKind':'Scene','surfaceKey':'actual-unit-surface','atlasUv':None,'overlayPresent':True,
                 'commandIndex':4,'sourceRef':reference,'commandBounds':bounds,'commandClip':bounds,
                 'resourceKey':'viewport:unit','resourceGeneration':1,'physicalSize':[2,1]}
        root = {'sourcePath':'res://ui/editor/unreal/level_editor_window.zui','sourceNodeId':'root','instancePath':'[]',
                'nodeId':'root','surfaceNodeId':1,'component':'Overlay','controlId':'WorkbenchWindowRoot','parent':None}
        authored_root = {**viewport_root,'nodeId':'viewport-root','surfaceNodeId':4,'component':'VerticalGroup',
                         'controlId':'WorkbenchMainBandViewportPanel','parent':root}
        pane = {**viewport,'nodeId':'viewport','surfaceNodeId':2,'component':'Overlay',
                'controlId':'UnrealSceneViewViewport','parent':authored_root}
        image = {**owner,'nodeId':'scene','surfaceNodeId':3,'component':'SceneViewViewport',
                 'controlId':'WorkbenchViewportSurface','parent':pane}
        rows = [root,authored_root,pane,image]
        for row in rows:
            row['projected'] = {key:value for key,value in row.items() if key != 'parent'}
            row['projected']['parentId'] = row['parent']['nodeId'] if row['parent'] else None
        counters = {key:1 for key in ('structure','geometry','hitTest','interaction','viewport','theme','diagnostics')}
        profile = {'normal_window_evidence':{'commands':[command, *dock_images, *source_null_images], 'presentationCursor':counters,'presentedFrameCount':4,
                   'arrangedInventoryPublication':{'schema':'dev.zircon.editor.normal-arranged-inventory','version':1,
                   'authority':'normal-retained-runtime-surfaces','observed':True,'complete':False,
                   'presentationCursor':counters,'presentedFrameCount':4,'surfaces':[{'schema':'dev.zircon.editor.normal-surface-arranged-inventory',
                   'version':1,'coordinateSpace':'surface-logical','frameAuthority':'retained-host-projection','scope':'workbench-window',
                   'documentId':'res://ui/editor/unreal/level_editor_window.zui','sourceFrameGeneration':13,'rootIds':[1],'nodes':rows}]}}}
        return scene, profile

    def test_actual_scene_command_selection_preserves_unrelated_r57_image_sources(self):
        scene, profile = self.command_fixture()
        commands = profile['normal_window_evidence']['commands']
        before = copy.deepcopy(commands)
        self.assertEqual([row['commandIndex'] for row in commands if (row.get('source') or {}).get('sourceRef', {}).get('treeId') == 'host.bottom.dock.header'],
                         [78,81,84,94,98,102,104,106,108])
        self.assertEqual([row['commandIndex'] for row in commands if row.get('source') is None], [284,286,288])
        for frame_local in (False, True):
            self.assertIs(validate_scene_command(scene, profile, frame_local=frame_local), commands[0])
        self.assertEqual(commands, before)

    def test_actual_scene_command_selection_rejects_relevant_identity_gaps_and_duplicates(self):
        for kind in ('missing invocation', 'null invocation', 'malformed invocation', 'partial invocation step', 'wrong alias',
                     'wrong node', 'wrong invocation', 'partial candidate beside target', 'duplicate owner'):
            scene, profile = self.command_fixture()
            commands = profile['normal_window_evidence']['commands']; source = commands[0]['source']
            if kind == 'missing invocation': source.pop('instancePath')
            if kind == 'null invocation': source['instancePath'] = None
            if kind == 'malformed invocation': source['instancePath'] = '['
            if kind == 'partial invocation step': source['instancePath'] = json.dumps([{'sourcePath': 'res://ui/editor/unreal/level_editor_window.zui'}])
            if kind == 'wrong alias': source['sourcePath'] = 'res://other/editor/unreal/components/viewport.zui'
            if kind == 'wrong node': source['sourceNodeId'] = 'toolbar'
            if kind == 'wrong invocation': source['instancePath'] = '[]'
            if kind in ('partial candidate beside target', 'duplicate owner'):
                extra = copy.deepcopy(commands[0]); extra['commandIndex'] = 5
                extra['source']['sourcePath'] = 'zircon_editor/assets/ui/editor/unreal/components/viewport.zui'
                if kind == 'partial candidate beside target': extra['source'].pop('instancePath')
                commands.append(extra)
            for frame_local in (False, True):
                with self.subTest(kind=kind, frame_local=frame_local), self.assertRaises(ValueError):
                    validate_scene_command(scene, profile, frame_local=frame_local)

    def test_actual_scene_root_identity_accepts_official_aliases_without_rewriting_packet(self):
        scene, profile = self.command_fixture()
        before = copy.deepcopy(scene)
        def canonical(path):
            return 'zircon_editor/assets/' + path[6:] if path.startswith('res://ui/editor/') else path
        rows = profile['normal_window_evidence']['arrangedInventoryPublication']['surfaces'][0]['nodes']
        for row in rows:
            for value in (row, row['projected']):
                value['sourcePath'] = canonical(value['sourcePath'])
                steps = json.loads(value['instancePath']) if isinstance(value['instancePath'], str) else value['instancePath']
                value['instancePath'] = json.dumps([{**step, 'sourcePath': canonical(step['sourcePath'])} for step in steps])
        self.assertEqual(validate_scene_command(scene, profile, frame_local=True)['commandIndex'], 4)
        self.assertEqual(scene, before)
        self.assertEqual(scene['viewportRoot']['sourceNodeId'], 'root')

    def test_actual_scene_root_identity_rejects_wrong_layer_control_kind_and_parent(self):
        for kind in ('root is inner viewport', 'root invocation', 'root control', 'root kind', 'viewport control', 'viewport kind',
                     'media control', 'media kind', 'command control', 'viewport skips root', 'media skips viewport',
                     'duplicate root', 'projected control', 'source suffix', 'invocation alias'):
            scene, profile = self.command_fixture()
            rows = profile['normal_window_evidence']['arrangedInventoryPublication']['surfaces'][0]['nodes']
            root, viewport, media = (next(row for row in rows if row['nodeId'] == key) for key in ('viewport-root', 'viewport', 'scene'))
            if kind == 'root is inner viewport': scene['viewportRoot']['sourceNodeId'] = 'viewport'
            if kind == 'root invocation': scene['viewportRoot']['instancePath'] = '[]'
            if kind == 'root control': root['controlId'] = 'UnrealSceneViewViewport'
            if kind == 'root kind': root['component'] = 'Overlay'
            if kind == 'viewport control': viewport['controlId'] = 'WorkbenchMainBandViewportPanel'
            if kind == 'viewport kind': viewport['component'] = 'VerticalGroup'
            if kind == 'media control': media['controlId'] = 'another'
            if kind == 'media kind': media['component'] = 'Overlay'
            if kind == 'command control': profile['normal_window_evidence']['commands'][0]['source']['controlId'] = 'another'
            if kind == 'viewport skips root': viewport['parent'] = rows[0]; viewport['projected']['parentId'] = rows[0]['nodeId']
            if kind == 'media skips viewport': media['parent'] = root; media['projected']['parentId'] = root['nodeId']
            if kind == 'duplicate root': rows.append({**copy.deepcopy(root), 'nodeId': 'another-root', 'surfaceNodeId': 5})
            if kind == 'projected control': viewport['projected']['controlId'] = 'another'
            if kind == 'source suffix': scene['viewportRoot']['sourcePath'] = 'foreign/ui/editor/unreal/components/viewport.zui'
            if kind == 'invocation alias': scene['viewportRoot']['instancePath'] = scene['viewportRoot']['instancePath'].replace('res://ui/editor/', 'res://other/editor/')
            for frame_local in (False, True):
                with self.subTest(kind=kind, frame_local=frame_local), self.assertRaises(ValueError):
                    validate_scene_command(scene, profile, frame_local=frame_local)

    def test_official_png_filters_contiguous_idat_and_ancillary_chunks(self):
        rgba = bytes([value for row in range(5) for value in (row*11,30,128,255,5,row*13,200,255)])
        png = self.png(rgba,2,5,[0,1,2,3,4])
        self.assertEqual(decode_scene_png(png,(2,5)),rgba)
        with self.assertRaises(ValueError): decode_own_png(png,(2,5))
        for invalid in (png+b'x',png[:-1],png[:40]+bytes([png[40]^1])+png[41:]):
            with self.assertRaises(ValueError): decode_scene_png(invalid,(2,5))

    def test_gpu_view_interpretation_and_explicit_linear_conversion(self):
        import math
        raw = bytes((0,128,255,255))
        for source_format,layout,encoding in (('Rgba8UnormSrgb','rgba8','srgb'),('Bgra8UnormSrgb','bgra8','srgb'),
                                               ('Rgba8Unorm','rgba8','linear'),('Bgra8Unorm','bgra8','linear')):
            rgba = raw if layout=='rgba8' else bytes((255,128,0,255))
            algorithm = 'identity-rgba8-srgb' if layout=='rgba8' else 'bgra8-srgb-to-rgba8-srgb'
            if encoding=='linear':
                rgba = bytes(math.floor(255*(12.92*(x/255) if x/255<=.0031308 else 1.055*(x/255)**(1/2.4)-.055)+.5)
                             for x in rgba[:3])+b'\xff'
                algorithm = 'linear8-to-rgba8-srgb-iec61966-2-1'
            scene = {'physicalSize':[1,1],'rawRowBytes':4,'rawByteLength':4,'rawSha256':digest(raw),'rgba8Sha256':digest(rgba),
                     'sourceFormat':source_format,'sourceByteLayout':layout,'colorEncoding':encoding,'alphaMode':'opaque',
                     'opaquePixels':True,'pngColorEncoding':'srgb','readbackCompleted':True,'failure':None,
                     'conversion':{'algorithm':algorithm,'inputSha256':digest(raw),'outputSha256':digest(rgba)}}
            self.assertEqual(scene_rgba(scene,raw),rgba)
            for key,bad in (('alphaMode','premultiplied'),('colorEncoding','invented'),('rawRowBytes',256),('rawSha256','a'*64),('conversion',{})):
                with self.assertRaises(ValueError): scene_rgba({**scene,key:bad},raw)

    def test_actual_command_owner_parent_and_generation_are_required(self):
        scene,profile = self.command_fixture()
        self.assertEqual(validate_scene_command(scene,profile,frame_local=True)['commandIndex'],4)
        self.assertEqual(scene['sourceRef']['generation'],11)
        self.assertEqual(profile['normal_window_evidence']['arrangedInventoryPublication']['surfaces'][0]['sourceFrameGeneration'],13)
        for key,value in (('resourceGeneration',2),('commandIndex',9),('physicalSize',[1,1]),('atlasUv',[0,0,1,1])):
            with self.assertRaises(ValueError): validate_scene_command({**scene,key:value},profile,frame_local=True)
        for mutate in ('duplicate command','wrong owner','outside root','stale source frame','missing runtime id','duplicate actual id','partial source reference','wrong root invocation',
                       'generation rollback','wrong command tree','wrong physical owner','wrong physical parent','missing commands'):
            changed = copy.deepcopy(profile); normal=changed['normal_window_evidence']
            rows=normal['arrangedInventoryPublication']['surfaces'][0]['nodes']
            if mutate=='duplicate command': normal['commands'].append(copy.deepcopy(normal['commands'][0]))
            if mutate=='wrong owner': normal['commands'][0]['source']['instancePath']='[]'
            if mutate=='outside root': rows[1]['parent']=None;rows[1]['projected']['parentId']=None
            if mutate=='stale source frame': normal['arrangedInventoryPublication']['surfaces'][0]['sourceFrameGeneration']=10
            if mutate=='missing runtime id': rows[2]['surfaceNodeId']=None
            if mutate=='duplicate actual id': rows[1]['surfaceNodeId']=3
            if mutate=='partial source reference': del normal['commands'][0]['source']['sourceRef']['treeId']
            if mutate=='wrong root invocation': rows[0]['instancePath']='[{"sourcePath":"res://ui/editor/unreal/level_editor_window.zui","sourceNodeId":"another"}]'
            if mutate=='generation rollback': normal['commands'][0]['source']['sourceRef']['generation']=10
            if mutate=='wrong command tree': normal['commands'][0]['source']['sourceRef']['treeId']='template.v2.other'
            if mutate=='wrong physical owner': normal['commands'][0]['source']['sourceRef']['nodeId']=999
            if mutate=='wrong physical parent': normal['commands'][0]['source']['parentNodeId']='other'
            if mutate=='missing commands': normal.pop('commands')
            with self.assertRaises(ValueError): validate_scene_command(scene,changed,frame_local=True)
        for kind in ('foreign matching tree','future matching source frame','partial current source reference'):
            changed_scene,changed = copy.deepcopy((scene,profile))
            reference=changed['normal_window_evidence']['commands'][0]['source']['sourceRef']
            if kind=='foreign matching tree': reference['treeId']='template.v2.other'
            if kind=='future matching source frame': reference['generation']=14
            if kind=='partial current source reference': reference.pop('fragmentIndex')
            changed_scene['sourceRef']=copy.deepcopy(reference)
            for frame_local in (False,True):
                with self.subTest(kind=kind,frame_local=frame_local), self.assertRaises(ValueError):
                    validate_scene_command(changed_scene,changed,frame_local=frame_local)

    def test_scene_packet_does_not_accept_a_disk_path_or_window_only_receipt(self):
        for packet in (None,{'image':{'path':'scene.png'}},{'schema':'dev.zircon.editor.runtime-scene-media','version':1}):
            with self.assertRaises(ValueError): runtime_scene_packet({'data':{'normal_window_evidence':{'runtimeSceneMedia':packet}}})

    def test_two_nonce_lifecycle_preserves_bootstrap_and_rejects_changed_content(self):
        import base64
        import json
        from unittest.mock import patch
        import zircon_editor_normal_frame_evidence as evidence
        # Physical receipt/command/PNG validation is covered above. This boundary
        # checks how their verified results join the two immutable cases.
        scene,profile = self.command_fixture()
        png = self.png(bytes((0,128,255,255,20,30,40,255)),2,1,[0])
        scene.update(rawSha256='1'*64,rgba8Sha256='2'*64,pngSha256=digest(png),pngByteLength=len(png))
        first={'sceneVerified':True,'sceneImage':scene,'png':png,'command':profile['normal_window_evidence']['commands'][0]}
        current=copy.deepcopy(first);current['command']['commandIndex']=5;current['sceneImage']['commandIndex']=5
        current['sceneImage']['sourceRef']={**scene['sourceRef'],'fragmentIndex':1}
        bootstrap={'host':'editor','dpi':1,'viewport':{'width':2,'height':1},'data':{'normal_window_evidence':{}}}
        bootstrap_bytes=json.dumps(bootstrap).encode();bootstrap_sha=evidence.canonical_case_sha(bootstrap)
        contract={'configurationPath':'E:/unit/contract.json','configurationSha256':'3'*64,
                  'moduleSha256':'4'*64,'inputAuditSha256':'5'*64}
        observed={'nonce':'nonce1','caseJsonSha256':digest(bootstrap_bytes),'caseSha256':bootstrap_sha,
                  'officialCaseHashModuleSha256':'4'*64,'officialCaseHashInputAuditSha256':'5'*64,'caseContract':contract,
                  'processImage':{'path':'E:/unit/editor.exe','sha256':'6'*64},'physicalSize':[2,1],'actualWinitScaleFactor':1,
                  'normalReadbackReceipt':{'sceneImage':scene},'normalReadbackReceiptPath':'E:/unit/normal-gpu-nonce1.json'}
        observed_bytes=json.dumps(observed).encode()
        packet={'schema':'dev.zircon.editor.runtime-scene-media','version':1,
                'bootstrapCase':{'path':'E:/unit/bootstrap.json','caseJsonSha256':digest(bootstrap_bytes),'caseSha256':bootstrap_sha},
                'bootstrapPixelAssociation':{'path':'E:/unit/proof.json','sha256':digest(observed_bytes),'observed':observed},
                'image':{'mime':'image/png','byteLength':len(png),'pngSha256':digest(png),'base64':base64.b64encode(png).decode()}}
        final_case={**bootstrap,'data':{'normal_window_evidence':{'runtimeSceneMedia':packet}}}
        final={**observed,'nonce':'nonce2','caseSha256':'7'*64,'normalReadbackReceiptPath':'E:/unit/normal-gpu-nonce2.json',
               'normalReadbackReceipt':{'sceneImage':current['sceneImage']}}
        inputs={str(Path('E:/unit/bootstrap.json')):bootstrap_bytes,str(Path('E:/unit/proof.json')):observed_bytes}
        def joined(final_proof, current_image):
            with patch.object(evidence,'runtime_media_file',side_effect=lambda value:value), \
                 patch.object(Path,'read_bytes',autospec=True,side_effect=lambda value:inputs[str(value)]), \
                 patch.object(evidence,'load_case_contract_configuration',return_value=contract), \
                 patch.object(evidence,'validate_association',return_value=({},Path('.'),Path('.'),profile)), \
                 patch.object(evidence,'validate_scene_image',side_effect=[first,current_image]):
                return evidence.validate_runtime_scene_media(final_case,final_proof,profile)
        result=joined(final,current)
        self.assertTrue(result['sceneVerified']);self.assertEqual(result['bootstrapNonce'],'nonce1');self.assertEqual(result['nonce'],'nonce2')
        self.assertEqual(result['commandIndex'],5)
        self.assertNotIn('wholeUiAccepted',result)
        for kind in ('same nonce','different contract','different binary','different resource','different raw','different PNG'):
            proof,image=copy.deepcopy(final),copy.deepcopy(current)
            if kind=='same nonce': proof['nonce']='nonce1'
            if kind=='different contract': proof['officialCaseHashModuleSha256']='8'*64
            if kind=='different binary': proof['processImage']['sha256']='8'*64
            if kind=='different resource': image['sceneImage']['resourceGeneration']=2
            if kind=='different raw': image['sceneImage']['rawSha256']='8'*64
            if kind=='different PNG': image['png']=png+b'x'
            with self.assertRaises(ValueError,msg=kind): joined(proof,image)

    def test_recorder_keeps_runtime_readback_separate_from_disk_resolver_readiness(self):
        import importlib.util
        spec = importlib.util.spec_from_file_location('normal_window_scene_recorder_test',Path(__file__).with_name('record-editor-normal-window-evidence.py'))
        recorder = importlib.util.module_from_spec(spec); spec.loader.exec_module(recorder)
        scene,profile = self.command_fixture()
        scene.update(rawSha256='1'*64,pngSha256='2'*64,rgba8Sha256='3'*64,conversion={'algorithm':'unit'})
        normal=profile['normal_window_evidence'];normal['commandStreamFullRebuild']=True
        normal['paintEvidence']={'styles':[],'issues':[],'assetAudit':{'complete':False,'resources':[]}}
        profile['submitted_gpu_text']={'commandStreamFullRebuild':True,'presentedFrameCount':4,'runs':[]}
        unverified=recorder.present_observations(profile,Path('.'),1)
        self.assertFalse(unverified['mediaReadiness']['sceneVerified'])
        proof={'sceneVerified':True,'authority':'two-actual-normal-submission-scene-readbacks','sceneImage':scene,
               'commandIndex':4,'nonce':'nonce2','bootstrapNonce':'nonce1'}
        result=recorder.present_observations(profile,Path('.'),1,proof)
        self.assertTrue(result['mediaReadiness']['sceneVerified'])
        self.assertFalse(result['mediaReadiness']['complete'])
        self.assertEqual(result['mediaReadiness']['resources'][0]['kind'],'runtime-product-scene')
        self.assertNotIn('loaded',result['mediaReadiness']['resources'][0])
        self.assertEqual(result['runtimeAssetFingerprints'],[])
        altered=copy.deepcopy(proof);altered['sceneImage']['resourceGeneration']=9
        with self.assertRaises(ValueError): recorder.present_observations(profile,Path('.'),1,altered)


class InspectorRepeatEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        import importlib.util
        path = Path(__file__).with_name('record-editor-normal-window-evidence.py')
        spec = importlib.util.spec_from_file_location('normal_window_record_repeat_test', path)
        cls.recorder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.recorder)

    @staticmethod
    def item(key='11', owner='properties', field='camera.near'):
        return {'sourcePath': 'res://ui/editor/workbench/data/component_property_row.zui',
                'sourceNodeId': 'root', 'instancePath': '[]',
                'parentSourcePath': 'res://ui/editor/unreal/components/details.zui',
                'parentSourceNodeId': owner, 'parentInstancePath': '[]',
                'itemKey': key, 'propertyFieldId': field}

    def profile(self, nodes, styles):
        return {'normal_window_evidence': {'commandStreamFullRebuild': True,
                    'presentedFrameCount': 1, 'commands': [], 'semanticNodes': nodes,
                    'paintEvidence': {'styles': styles, 'issues': [],
                                      'assetAudit': {'complete': True, 'resources': []}}},
                'submitted_gpu_text': {'commandStreamFullRebuild': True,
                                      'presentedFrameCount': 1, 'runs': []}}

    def test_each_item_and_owner_keeps_its_own_painted_style(self):
        owners = [self.item(), self.item('12', field='camera.far'), self.item(owner='other-properties')]
        self.assertEqual(len({self.recorder.source_identity(item) for item in owners}), 1)
        self.assertEqual(len({self.recorder.paint_identity(item) for item in owners}), 3)
        rect = {'x': 0, 'y': 0, 'width': 100, 'height': 20}
        nodes = [{**item, 'nodeId': f'physical-{index}', 'bounds': rect}
                 for index, item in enumerate(owners)]
        styles = [{**item, 'effectiveStyle': {'complete': True, 'properties': {'sample': index}}}
                  for index, item in enumerate(owners)]
        evidence = self.recorder.present_observations(self.profile(nodes, styles), Path('.'), 1.0)
        self.assertTrue(evidence['semanticAudit']['complete'])
        self.assertEqual([node['effectiveStyle']['properties']['sample']
                          for node in evidence['semanticNodes']], [0, 1, 2])

    def test_old_slot_or_owner_receipt_does_not_certify_a_new_item(self):
        original = self.item()
        for changed in (self.item('12'), self.item(owner='other-properties'),
                        self.item(field='camera.changed')):
            self.assertFalse(self.recorder.same_paint_identity(original, changed))
            evidence = self.recorder.present_observations(
                self.profile([{**changed, 'bounds': None}], [original]), Path('.'), 1.0)
            self.assertFalse(evidence['semanticAudit']['complete'])
            self.assertIsNone(evidence['semanticNodes'][0]['effectiveStyle'])
        duplicate = self.recorder.present_observations(
            self.profile([{**original, 'bounds': None}], [original, original]), Path('.'), 1.0)
        self.assertFalse(duplicate['semanticAudit']['complete'])

    def test_partial_and_noncanonical_repeat_identity_stays_unknown(self):
        original = self.item()
        for key in ('', '011', '-11', ' 11', '١١', str(1 << 128), 11, True):
            self.assertIsNone(self.recorder.paint_identity({**original, 'itemKey': key}))
        for field in ('parentSourcePath', 'parentSourceNodeId', 'parentInstancePath', 'propertyFieldId'):
            for missing in (None, ''):
                self.assertIsNone(self.recorder.paint_identity({**original, field: missing}))
        authored = {key: original[key] for key in ('sourcePath', 'sourceNodeId', 'instancePath')}
        self.assertEqual(self.recorder.paint_identity(authored), self.recorder.source_identity(authored))


if __name__=='__main__':unittest.main()
