"""Focused validation checks; these do not simulate GPU/Win32 acceptance."""
import copy
import os
import tempfile
import struct
import unittest
import zlib
from zircon_editor_normal_frame_evidence import validate_hold, validate_readback, validate_profile, raw_rgba, digest, decode_own_png, validate_capture_instance, validate_live_request, validate_case_publication, validate_case_contract_documents, validate_capture_method, validate_case_inputs
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
        canonical = root / "dev/penpot/plugins/apps/zircon-zui-plugin"
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
        for key, value in (("canonicalAppRoot", str(root / "tools/penpot/apps/zircon-zui-plugin")),
                           ("repoRoot", "D:/old-source-copy"), ("sourceCopiesCreated", True),
                           ("outputSha256", "c"*64)):
            bad = copy.deepcopy(audit); bad[key] = value
            with self.assertRaises(ValueError):
                validate_case_contract_documents(config, bad, root)
        for index in range(len(audit["inputs"])):
            bad = copy.deepcopy(audit); bad["inputs"].pop(index)
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

if __name__=='__main__':unittest.main()
