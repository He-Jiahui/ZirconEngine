import tempfile, time, unittest
from pathlib import Path
from tools.jenkins.state import State
from tools.jenkins.resources import Capacity, ResourceManager
from tools.jenkins.contracts import JenkinsError

class SchedulingTests(unittest.TestCase):
 def setUp(self):
  self.root=Path(tempfile.mkdtemp()); self.state=State(self.root/'state.db')
  self.state.put('resource_policy','default',{'cpuBudget':2,'memoryBudget':4*1024**3,'diskBudget':8*1024**3,'diskReserveBytes':35*1024**3,'maxHeavyWriters':1,'buildRoot':r'E:\cargo-targets'})
  self.m=ResourceManager(self.state,Capacity(2,4*1024**3,8*1024**3)); self.m.publish_inventory('e1',Capacity(2,4*1024**3,40*1024**3),build_root=r'E:\cargo-targets')
 def test_heavy_cap_and_unknown_inventory(self):
  a=self.m.admit('a',Capacity(1,1,1),inventory_generation='e1',build_root=r'E:\cargo-targets',writer_key='w1',heavy_writer=True,request_id='a')
  self.assertIsNotNone(a)
  self.assertIsNone(self.m.admit('b',Capacity(1,1,1),inventory_generation='e1',build_root=r'E:\cargo-targets',writer_key='w2',heavy_writer=True,request_id='b'))
  with self.assertRaises(JenkinsError): self.m.admit('c',Capacity(1,1,1),inventory_generation='missing',build_root=r'E:\cargo-targets',request_id='c')
 def test_explicit_root_and_payload_collision(self):
  self.m.publish_inventory('f1',Capacity(2,4*1024**3,40*1024**3),build_root=r'F:\cargo-targets')
  a=self.m.admit('a',Capacity(1,1,1),inventory_generation='f1',build_root=r'F:\cargo-targets',request_id='x')
  self.assertIsNotNone(a)
  with self.assertRaises(JenkinsError): self.m.admit('a',Capacity(2,1,1),inventory_generation='f1',build_root=r'F:\cargo-targets',request_id='x')
 def test_queue_age_survives_retry_and_advances(self):
  q=self.m.queue('q',request_id='q1',priority=1,amount=Capacity(1,1,1)); old=q['payload']['queuedAt']
  q2=self.m.queue('q',request_id='q1',priority=1,amount=Capacity(1,1,1)); self.assertEqual(old,q2['payload']['queuedAt'])
  self.assertEqual(self.m.fair_queue()[0]['key'],q['key'])
 def test_positive_heavy_estimates_and_floor(self):
  with self.assertRaises(JenkinsError): self.m.admit('z',Capacity(0,1,1),inventory_generation='e1',build_root=r'E:\cargo-targets',heavy_writer=True,request_id='z')
  self.assertIsNone(self.m.admit('big',Capacity(1,1,6*1024**3),inventory_generation='e1',build_root=r'E:\cargo-targets',request_id='big'))

if __name__=='__main__': unittest.main()
