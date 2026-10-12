import tempfile, time, unittest
from pathlib import Path
from tools.jenkins.state import State
from tools.jenkins.resources import Capacity, ResourceManager
from tools.jenkins.contracts import JenkinsError
from unittest.mock import patch
from tools.jenkins.resources.paths import ApprovedBuildRoot

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

 def multi_root(self):
  gib=1024**3
  self.state.put('resource_policy','default',{'cpuBudget':6,'memoryBudget':12*gib,'diskBudget':40*gib,
      'diskReserveBytes':35*gib,'maxHeavyWriters':4,'maxNewHeavyWriters':2,'agingSeconds':60,
      'allowedBuildRoots':[r'D:\cargo-targets',r'E:\cargo-targets',r'F:\cargo-targets'],
      'quarantineReserve':{'executionIds':['historic-a','historic-b'],'countedInBudget':True}})
  self.m=ResourceManager(self.state,Capacity(6,12*gib,40*gib))
  self.m.publish_inventory('d',Capacity(6,12*gib,40*gib),build_root=r'D:\cargo-targets')
  self.m.publish_inventory('f',Capacity(6,12*gib,40*gib),build_root=r'F:\cargo-targets')
  return gib

 def test_other_root_growth_does_not_charge_selected_root_but_global_budget_stays_shared(self):
  gib=self.multi_root()
  self.state.put('resource_reservation','historic-a:h',{'owner':'historic-a','status':'active','cpu':1,
      'memoryBytes':gib,'diskBytes':10*gib,'buildRoot':r'D:\cargo-targets','heavyWriter':True})
  before=self.state.get('resource_reservation','historic-a:h')
  result=self.m.admit('new-f',Capacity(1,gib,4*gib),inventory_generation='f',request_id='f',
                      build_root=r'F:\cargo-targets',heavy_writer=True,writer_key='pool-f')
  self.assertIsNotNone(result)
  self.assertIsNone(self.m.admit('d-growth',Capacity(1,gib,gib),inventory_generation='d',request_id='d',build_root=r'D:\cargo-targets'))
  self.assertEqual(before,self.state.get('resource_reservation','historic-a:h'))
  self.state.put('resource_reservation','large:e',{'owner':'large','status':'active','cpu':1,
      'memoryBytes':gib,'diskBytes':27*gib,'buildRoot':r'E:\cargo-targets'})
  self.assertIsNone(self.m.admit('global-full',Capacity(1,gib,1),inventory_generation='f',request_id='full',build_root=r'F:\cargo-targets'))

 def test_unknown_historical_root_charges_each_root_conservatively(self):
  gib=self.multi_root()
  self.state.put('resource_reservation','old:unknown',{'owner':'old','status':'active','cpu':1,
      'memoryBytes':gib,'diskBytes':2*gib,'buildRoot':'old-store'})
  self.assertIsNone(self.m.admit('new',Capacity(1,gib,4*gib),inventory_generation='f',request_id='x',build_root=r'F:\cargo-targets'))

 def test_two_new_writers_remain_cap_when_historical_writers_released(self):
  gib=self.multi_root()
  for owner,root,generation in [('a',r'D:\cargo-targets','d'),('b',r'F:\cargo-targets','f')]:
   self.assertIsNotNone(self.m.admit(owner,Capacity(1,gib,gib),inventory_generation=generation,
       request_id=owner,build_root=root,heavy_writer=True,writer_key='pool-'+owner))
  self.assertIsNone(self.m.admit('c',Capacity(1,gib,gib),inventory_generation='f',request_id='c',
      build_root=r'F:\cargo-targets',heavy_writer=True,writer_key='pool-c'))

 def test_new_cap_counts_only_exact_policy_historical_set_and_preserves_budget(self):
  gib=self.multi_root()
  for owner in ('historic-a','historic-b'):
   self.state.put('resource_reservation',owner+':h',{'owner':owner,'status':'active','cpu':1,
       'memoryBytes':gib,'diskBytes':gib,'buildRoot':r'D:\cargo-targets','heavyWriter':True})
  before=self.state.list('resource_reservation')
  for owner in ('a','b'):
   self.assertIsNotNone(self.m.admit(owner,Capacity(1,gib,gib),inventory_generation='f',request_id=owner,
       build_root=r'F:\cargo-targets',heavy_writer=True,writer_key=owner))
  self.assertEqual(before,[self.state.get('resource_reservation',row['key']) for row in before])
  self.assertIsNone(self.m.admit('c',Capacity(1,gib,gib),inventory_generation='f',request_id='c',
      build_root=r'F:\cargo-targets',heavy_writer=True,writer_key='c'))

 def test_queue_pins_root_recipe_writer_and_priority_on_retry(self):
  self.multi_root()
  original=self.m.queue('a',request_id='x',amount=Capacity(1,1,1),build_root=r'D:\cargo-targets',heavy_writer=True,writer_key='pool')
  self.assertEqual(original,self.m.queue('a',request_id='x',amount=Capacity(1,1,1),build_root='D:/cargo-targets',heavy_writer=True,writer_key='pool'))
  for change in ({'build_root':r'F:\cargo-targets'},{'priority':1},{'writer_key':'other'},{'heavy_writer':False}):
   with self.subTest(change=change),self.assertRaises(JenkinsError):
    self.m.queue('a',request_id='x',**{'amount':Capacity(1,1,1),'build_root':r'D:\cargo-targets','heavy_writer':True,'writer_key':'pool',**change})
  self.assertEqual(original,self.state.get('resource_queue',original['key']))

 def test_global_aging_head_remains_a_gate_on_other_drive(self):
  gib=self.multi_root()
  oldest=self.m.queue('old',request_id='old',amount=Capacity(1,gib,6*gib),build_root=r'D:\cargo-targets')
  self.m.queue('fit',request_id='fit',amount=Capacity(1,gib,gib),build_root=r'F:\cargo-targets')
  self.assertIsNone(self.m.admit('fit',Capacity(1,gib,gib),request_id='fit',inventory_generation='f',build_root=r'F:\cargo-targets'))
  self.assertEqual(oldest,self.m.fair_queue()[0])

 def test_priority_and_inventory_values_reject_nan_bool_future_stale_zero(self):
  for value in (True,float('nan'),float('inf'),1001,-1001,'1'):
   with self.subTest(value=value),self.assertRaises(JenkinsError): self.m.queue('x',priority=value)
  for update in ({'observedAt':time.time()+10},{'observedAt':time.time()-31},{'cpu':True},{'diskBytes':0},{'complete':1}):
   old=self.state.get('capacity_inventory','e1')['payload']
   self.state.put('capacity_inventory','bad',{**old,**update})
   with self.subTest(update=update),self.assertRaises(JenkinsError):
    self.m.admit('a',Capacity(1,1,1),inventory_generation='bad',build_root=r'E:\cargo-targets')
  with self.assertRaises(JenkinsError): self.m.admit('a',Capacity(True,1,1),inventory_generation='e1',build_root=r'E:\cargo-targets')

 def test_physical_alias_check_and_maintenance_hold_block_admission(self):
  with patch('tools.jenkins.resources.paths.canonical_build_root',side_effect=JenkinsError('build_root_alias','alias')):
   with self.assertRaises(JenkinsError): self.m.admit('a',Capacity(1,1,1),inventory_generation='e1',build_root=r'E:\cargo-targets')
  self.state.put('cache_maintenance_hold','pool',{'status':'active','operationId':'gc'})
  with self.assertRaises(JenkinsError) as caught:
   self.m.admit('a',Capacity(1,1,1),inventory_generation='e1',build_root=r'E:\cargo-targets',writer_key='pool')
  self.assertEqual('pool_cache_maintenance_active',caught.exception.code)

 def fair_head(self,root=r'E:\cargo-targets',disk=None):
  gib=self.multi_root()
  policy=self.state.get('resource_policy','default')
  self.state.put('resource_policy','default',{**policy['payload'],'skipProvenRootDiskBlockedHeads':True})
  generation='head-root'
  self.m.publish_inventory(generation,Capacity(6,12*gib,36*gib if disk is None else disk),build_root=root)
  head=self.m.queue('head',request_id='head',amount=Capacity(1,gib,2*gib),build_root=root,
      heavy_writer=True,writer_key='pool-head')
  fit=self.m.queue('fit',request_id='fit',amount=Capacity(1,gib,gib),build_root=r'F:\cargo-targets',
      heavy_writer=True,writer_key='pool-fit')
  return gib,generation,head,fit

 def fit_admit(self,gib):
  return self.m.admit('fit',Capacity(1,gib,gib),request_id='fit',inventory_generation='f',
      build_root=r'F:\cargo-targets',heavy_writer=True,writer_key='pool-fit')

 def test_proven_E_and_D_disk_blocked_head_allows_F_and_records_unchanged_age(self):
  for root in (r'E:\cargo-targets',r'D:\cargo-targets'):
   with self.subTest(root=root):
    self.setUp()
    gib,generation,head,fit=self.fair_head(root)
    self.assertIsNotNone(self.fit_admit(gib))
    self.assertEqual(head,self.state.get('resource_queue',head['key']))
    decision=self.state.list('resource_fair_decision')[0]['payload']['skippedHeads'][0]
    self.assertEqual(head['key'],decision['queueKey'])
    self.assertEqual(generation,decision['inventoryGeneration'])
    self.assertEqual('root_disk_capacity',decision['reasonCode'])

 def test_global_budget_pressure_does_not_skip_larger_head(self):
  gib,generation,head,fit=self.fair_head()
  row=self.state.get('resource_queue',head['key'])
  self.state.put('resource_queue',row['key'],{**row['payload'],'amount':Capacity(3,3*gib,2*gib).__dict__})
  self.state.put('resource_reservation','busy:1',{'owner':'busy','status':'active','cpu':4,
      'memoryBytes':gib,'diskBytes':gib,'buildRoot':r'D:\cargo-targets'})
  self.assertIsNone(self.fit_admit(gib))
  self.assertEqual([],self.state.list('resource_fair_decision'))

 def test_opened_head_disk_capacity_restores_aged_head_precedence(self):
  gib,generation,head,fit=self.fair_head()
  self.m.publish_inventory('head-fresh',Capacity(6,12*gib,50*gib),build_root=r'E:\cargo-targets')
  self.assertIsNone(self.fit_admit(gib))
  admitted=self.m.admit('head',Capacity(1,gib,2*gib),request_id='head',inventory_generation='head-fresh',
      build_root=r'E:\cargo-targets',heavy_writer=True,writer_key='pool-head')
  self.assertIsNotNone(admitted)
  self.assertIsNotNone(self.fit_admit(gib))

 def test_unknown_stale_head_or_contended_pool_does_not_skip(self):
  for mutation in ('stale','unknown','writer'):
   with self.subTest(mutation=mutation):
    self.setUp()
    gib,generation,head,fit=self.fair_head()
    if mutation=='stale':
     inv=self.state.get('capacity_inventory',generation)
     self.state.put('capacity_inventory',generation,{**inv['payload'],'observedAt':time.time()-31})
    elif mutation=='unknown':
     inv=self.state.get('capacity_inventory',generation)
     self.state.put('capacity_inventory',generation,{**inv['payload'],'volumeIdentity':''})
    else: self.state.put('pool_writer_hold','pool-head',{'status':'active','owner':'other'})
    self.assertIsNone(self.fit_admit(gib))
    self.assertEqual([],self.state.list('resource_fair_decision'))

if __name__=='__main__': unittest.main()
