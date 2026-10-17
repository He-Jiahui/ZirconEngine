"""Fixed-ABI process inventory and nonterminal retained-object gates."""
import ctypes
import struct
import unittest
from unittest import mock
from types import SimpleNamespace
from pathlib import Path
from tools.jenkins.deployment import startup_observations as inventory
from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.startup_observations import _classify_nonexecuting_object, _parse_process_snapshot


class RuntimeOwnerInventoryTests(unittest.TestCase):
    def snapshot(self, *, parent=29312, threads=0, handles=0, peak=0, user=0, kernel=0):
        row={"pid":41432,"parentPid":parent,"creationTime":134365342481017826,
             "imageName":"python.exe","numberOfThreads":threads,"handleCount":handles,
             "peakThreads":peak,"userTime":user,"kernelTime":kernel}
        return {41432:row}

    def test_zero_history_requires_orphan_in_both_complete_snapshots_and_is_nonterminal(self):
        first,second=self.snapshot(),self.snapshot()
        observation=_classify_nonexecuting_object(first,second,41432,r"\Device\HarddiskVolume7\runtime\python.exe")
        self.assertEqual("non-executing-retained-process-object",observation["status"])
        for name in ("terminalProof","nativeProofIssued","stdoutEof","stderrEof","executionHoldsReleased"):
            self.assertFalse(observation[name])

    def test_zero_cpu_peak_only_and_live_creator_are_not_history_witnesses(self):
        first,second=self.snapshot(peak=4),self.snapshot(peak=4)
        first[29312]=second[29312]={"pid":29312}
        with self.assertRaises(JenkinsError):
            _classify_nonexecuting_object(first,second,41432,r"\Device\HarddiskVolume7\runtime\python.exe")

    def test_positive_cpu_allows_live_creator_but_not_live_activity_or_handles(self):
        first,second=self.snapshot(user=1),self.snapshot(user=2)
        first[29312]=second[29312]={"pid":29312}
        accepted=_classify_nonexecuting_object(first,second,41432,r"\Device\HarddiskVolume7\runtime\python.exe")
        self.assertEqual(2,accepted["userTime"])
        for bad in (self.snapshot(threads=1,peak=1,user=1),self.snapshot(handles=1,user=1)):
            with self.assertRaises(JenkinsError):
                _classify_nonexecuting_object(first,{41432:bad[41432]},41432,r"\Device\HarddiskVolume7\runtime\python.exe")

    def test_birth_parent_image_and_monotonic_counters_must_match(self):
        baseline=self.snapshot(user=2)
        for change in ({"creationTime":9},{"parentPid":7},{"imageName":"java.exe"},{"userTime":1},{"peakThreads":-1}):
            changed=self.snapshot(user=2);changed[41432].update(change)
            with self.subTest(change=change),self.assertRaises(JenkinsError):
                _classify_nonexecuting_object(baseline,changed,41432,r"\Device\HarddiskVolume7\runtime\python.exe")

    def test_fixed_parser_rejects_truncated_and_misaligned_records(self):
        with self.assertRaises(JenkinsError):_parse_process_snapshot(bytes(255),0x100000)
        raw=bytearray(264);struct.pack_into("<I",raw,0,4)
        with self.assertRaises(JenkinsError):_parse_process_snapshot(bytes(raw),0x100000)

    def test_fixed_parser_reads_x64_identity_and_checks_image_bounds(self):
        encoded="python.exe".encode("utf-16-le"); raw=ctypes.create_string_buffer(256+len(encoded)+4)
        base=ctypes.addressof(raw)
        struct.pack_into("<II",raw,0,0,0);struct.pack_into("<I",raw,20,0)
        struct.pack_into("<qqq",raw,32,134365342481017826,0,0)
        struct.pack_into("<HH",raw,56,len(encoded),len(encoded));struct.pack_into("<Q",raw,64,base+256)
        struct.pack_into("<QQII",raw,80,41432,29312,0,1);ctypes.memmove(base+256,encoded,len(encoded))
        result=_parse_process_snapshot(ctypes.string_at(raw,ctypes.sizeof(raw)),base)
        self.assertEqual(29312,result[41432]["parentPid"])
        self.assertEqual("python.exe",result[41432]["imageName"])
        self.assertEqual(134365342481017826,result[41432]["creationTime"])


    def test_observer_brackets_both_full_images_with_two_complete_snapshots(self):
        events=[]
        path=r"\Device\HarddiskVolume5\Git\ZirconEngine\.jenkins\runtime\python\python.exe"
        def snapshot():
            events.append("snapshot")
            return self.snapshot()
        def image(pid):
            self.assertEqual(41432,pid);events.append("image");return path
        with mock.patch.object(inventory,"_native_process_snapshot",side_effect=snapshot), \
                mock.patch.object(inventory,"_system_process_image",side_effect=image), \
                mock.patch.object(inventory,"_under_runtime",return_value=True), \
                mock.patch.object(inventory,"_physical_dos_path",side_effect=lambda x:x):
            observed=inventory._observe_nonexecuting_object(41432,Path(r"E:\Git\ZirconEngine"))
        self.assertEqual(["snapshot","image","snapshot","image"],events)
        self.assertFalse(observed["terminalProof"])

    def test_observer_rejects_same_basename_changed_volume_birth_query_or_creator(self):
        path=r"\Device\HarddiskVolume5\Git\ZirconEngine\.jenkins\runtime\python\python.exe"
        changed=self.snapshot();changed[41432]["creationTime"]+=1
        live_creator=self.snapshot();live_creator[29312]={"pid":29312}
        cases=(
            ([self.snapshot(),self.snapshot()],[path,path.replace("Volume5","Volume6")]),
            ([self.snapshot(),changed],[path,path]),
            ([self.snapshot(),self.snapshot()],[path,JenkinsError("deployment_inventory_inconclusive","query failed")]),
            ([live_creator,self.snapshot()],[path,path]),
            ([self.snapshot(),live_creator],[path,path]),
        )
        for snapshots,images in cases:
            with self.subTest(snapshots=snapshots,images=images), \
                    mock.patch.object(inventory,"_native_process_snapshot",side_effect=snapshots), \
                    mock.patch.object(inventory,"_system_process_image",side_effect=images), \
                    mock.patch.object(inventory,"_under_runtime",return_value=True), \
                    mock.patch.object(inventory,"_physical_dos_path",side_effect=lambda x:x), \
                    self.assertRaises(JenkinsError):
                inventory._observe_nonexecuting_object(41432,Path(r"E:\Git\ZirconEngine"))

    def query_image(self,path):
        def query(kind,pointer,size,returned):
            self.assertEqual(88,kind)
            info=pointer._obj
            self.assertEqual(41432,info.ProcessId)
            encoded=path.encode("utf-16-le")
            self.assertLess(len(encoded)+2,info.ImageName.MaximumLength)
            ctypes.memmove(info.ImageName.Buffer,encoded+b"\0\0",len(encoded)+2)
            info.ImageName.Length=len(encoded)
            info.ImageName.MaximumLength=len(encoded)+2
            return 0
        function=mock.Mock(side_effect=query)
        with mock.patch.object(inventory.ctypes,"WinDLL",
                return_value=SimpleNamespace(NtQuerySystemInformation=function)):
            return inventory._system_process_image(41432)

    def test_native_image_query_accepts_canonical_nt_path_on_python_314(self):
        path=r"\Device\HarddiskVolume5\Git\ZirconEngine\.jenkins\runtime\python\python.exe"
        self.assertEqual(path,self.query_image(path))

    def test_native_image_query_rejects_aliases_rooted_dos_and_embedded_nul(self):
        for path in (r"\foo\python.exe",r"\Device\..\python.exe",
                r"\Device\Volume5\foo\..\python.exe",r"\Device\\python.exe",
                "\\Device\\Volume5\\foo\0python.exe"):
            with self.subTest(path=path),self.assertRaises(JenkinsError):
                self.query_image(path)

    def record(self,*,pid=41432,threads=0,peak=0,name="java.exe",maximum_extra=2):
        encoded=name.encode("utf-16-le")
        minimum=256+80*threads
        maximum=len(encoded)+maximum_extra if encoded else 0
        length=(minimum+maximum+7)//8*8
        raw=bytearray(length)
        struct.pack_into("<II",raw,0,0,threads)
        struct.pack_into("<I",raw,20,peak)
        struct.pack_into("<qqq",raw,32,134365342481017826,0,0)
        struct.pack_into("<HH",raw,56,len(encoded),maximum)
        struct.pack_into("<Q",raw,64,0x100000+minimum if encoded else 0)
        struct.pack_into("<QQII",raw,80,pid,29312,0,1)
        raw[minimum:minimum+len(encoded)]=encoded
        return raw

    def test_parser_accepts_idle_zero_peak_and_maximum_length_padding(self):
        idle=self.record(pid=0,threads=48,peak=0,name="")
        self.assertEqual(48,_parse_process_snapshot(bytes(idle),0x100000)[0]["numberOfThreads"])
        raw=self.record()
        self.assertEqual("java.exe",_parse_process_snapshot(bytes(raw),0x100000)[41432]["imageName"])

    def test_parser_rejects_maximum_length_escape_and_truncated_thread_extent(self):
        bad_max=self.record();struct.pack_into("<H",bad_max,58,100)
        bad_threads=self.record(pid=0,threads=48,name="")[:-8]
        bad_peak=self.record(pid=4,threads=1,peak=0,name="")
        for raw in (bad_max,bad_threads,bad_peak):
            with self.subTest(length=len(raw)),self.assertRaises(JenkinsError):
                _parse_process_snapshot(bytes(raw),0x100000)


    def test_public_owner_observation_retains_raw_clients_and_nonterminal_objects(self):
        from tools.jenkins.tests.test_runtime_roles import RoleFixture
        from tools.jenkins.deployment import runtime_roles as roles
        fixture = RoleFixture()
        fixture.arguments[:] = [fixture.identity["executable"], "-B", "-", "ordinary-input"]
        fixture.identity["commandDigest"] = roles._digest_args(tuple(fixture.arguments))
        raw = inventory._RuntimeOwnerInventory([dict(fixture.identity)], [{"pid": 41432, "terminalProof": False}])
        with fixture.observations(), mock.patch.object(inventory, "runtime_owner_inventory", return_value=raw):
            view = inventory.observe_runtime_owner_inventory(fixture.manager.paths.repo)
        self.assertEqual(list(raw), view["runtimeInventory"])
        self.assertEqual(list(raw), view["activeOwners"])
        self.assertEqual([], view["blockingOwners"])
        self.assertEqual(1, len(view["nonServiceProcesses"]))
        self.assertFalse(view["nonExecutingObjects"][0]["terminalProof"])
        self.assertFalse(view["terminalProof"])


if __name__=="__main__":unittest.main()
