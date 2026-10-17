use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use super::super::error::{CoreError, CoreResult};
use super::{ModuleDescriptor, RegistryName};
use crate::core::{ServiceKind, StartupMode};

/// Immutable declaration snapshot used by every module lifecycle entry point.
///
/// Runtime entries keep their mutable lifecycle state in the registries. This
/// graph intentionally retains only validated topology and provenance, so the
/// declaration order cannot drift after lifecycle work has started.
///
/// 此快照不保存工厂或运行状态；失败时不产出半个图，成功缓存由 CoreHandle 的调用入口负责。
#[derive(Clone, Debug)]
pub(crate) struct FrozenModuleGraph {
    module_activation_order: Arc<[String]>,
    module_dependencies: HashMap<String, Arc<[String]>>,
    module_dependents: HashMap<String, Arc<[String]>>,
    module_services: HashMap<String, FrozenModuleServices>,
}

/// 冻结模块的服务计划：启动仅选 Immediate 项，关闭则反转完整依赖顺序。
#[derive(Clone, Debug)]
pub(crate) struct FrozenModuleServices {
    /// 本模块的全部服务，保留全局服务拓扑序在该模块上的投影。
    service_names: Arc<[RegistryName]>,
    /// 上述投影中标为 Immediate 的服务；解析其依赖仍可能创建 Lazy 服务。
    startup_service_names: Arc<[RegistryName]>,
    /// 全部服务投影的逆序，用于关闭、排空与卸载，不限于启动时实际创建的项。
    shutdown_service_names: Arc<[RegistryName]>,
}

impl FrozenModuleGraph {
    /// 从借用的描述符构建完整模块/服务拓扑，不执行工厂或生命周期回调。
    /// 模块边、服务边或环校验失败返回 CoreError；本函数不修改注册表或发布缓存。
    /// 名称归属与注册种类的准入由注册入口负责，图校验不能代替该入口。
    pub(crate) fn freeze(descriptors: &[ModuleDescriptor]) -> CoreResult<Self> {
        let module_activation_order = sort_module_activation_order(descriptors)?;
        let module_dependencies = module_dependency_map(descriptors);
        let module_dependents =
            module_dependent_map(&module_dependencies, module_activation_order.as_slice());
        let service_nodes = collect_service_nodes(descriptors)?;

        validate_service_dependencies(&service_nodes, &module_dependencies)?;
        let service_activation_order = sort_service_activation_order(&service_nodes)?;
        let module_services = module_service_plans(
            module_activation_order.as_slice(),
            service_activation_order.as_slice(),
            &service_nodes,
        );

        Ok(Self {
            module_activation_order: module_activation_order.into(),
            module_dependencies,
            module_dependents,
            module_services,
        })
    }

    /// 借用全局模块启动顺序；运行状态变化不会重排此拓扑。
    pub(crate) fn module_activation_order(&self) -> &[String] {
        &self.module_activation_order
    }

    /// 返回目标模块及其传递提供方，按全局启动序排列的独立名称列表。
    /// 目标未登记时返回缺失模块错误；这里只遍历声明，不检查模块当前是否运行。
    pub(crate) fn module_activation_closure(&self, module_name: &str) -> CoreResult<Vec<String>> {
        let Some((registered_name, _)) = self.module_dependencies.get_key_value(module_name) else {
            return Err(CoreError::MissingModule(module_name.to_owned()));
        };
        let mut closure: HashSet<&str> = HashSet::with_capacity(self.module_dependencies.len());
        let mut pending = vec![registered_name.as_str()];
        while let Some(current) = pending.pop() {
            if !closure.insert(current) {
                continue;
            }
            let dependencies = self
                .module_dependencies
                .get(current)
                .expect("validated module graph must contain every dependency node");
            pending.extend(dependencies.iter().map(String::as_str));
        }
        Ok(self
            .module_activation_order
            .iter()
            .filter(|name| closure.contains(name.as_str()))
            .cloned()
            .collect())
    }

    /// Lists all transitively dependent modules in stable activation order.
    /// 不包含目标模块自身；停用入口还需另查这些依赖方的当前生命周期。
    pub(crate) fn module_dependent_closure(&self, module_name: &str) -> CoreResult<Vec<String>> {
        let Some((_, direct_dependents)) = self.module_dependents.get_key_value(module_name) else {
            return Err(CoreError::MissingModule(module_name.to_owned()));
        };
        let mut closure: HashSet<&str> = HashSet::with_capacity(self.module_dependents.len());
        let mut pending = direct_dependents
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        while let Some(current) = pending.pop() {
            if !closure.insert(current) {
                continue;
            }
            let dependents = self
                .module_dependents
                .get(current)
                .expect("validated module graph must contain every dependent node");
            pending.extend(dependents.iter().map(String::as_str));
        }
        Ok(self
            .module_activation_order
            .iter()
            .filter(|name| closure.contains(name.as_str()))
            .cloned()
            .collect())
    }

    /// 借用指定模块的冻结服务计划；目标不存在时返回缺失模块错误。
    pub(crate) fn module_services(&self, module_name: &str) -> CoreResult<&FrozenModuleServices> {
        self.module_services
            .get(module_name)
            .ok_or_else(|| CoreError::MissingModule(module_name.to_owned()))
    }
}

impl FrozenModuleServices {
    pub(crate) fn service_names(&self) -> &Arc<[RegistryName]> {
        &self.service_names
    }

    pub(crate) fn startup_service_names(&self) -> &Arc<[RegistryName]> {
        &self.startup_service_names
    }

    pub(crate) fn shutdown_service_names(&self) -> &Arc<[RegistryName]> {
        &self.shutdown_service_names
    }
}

#[derive(Clone, Debug)]
struct ServiceGraphNode {
    name: RegistryName,
    owner_module: String,
    kind: ServiceKind,
    startup_mode: StartupMode,
    dependencies: Vec<RegistryName>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Visited,
}

#[derive(Clone, Copy, Debug)]
struct TraversalFrame {
    // 规范名称仍由图节点持有；显式帧只保存节点索引和下一条待访问的边。
    node_index: usize,
    next_dependency_index: usize,
}

/// Sorted service dependency indices used only while computing lifecycle order.
///
/// Service declarations retain their authored dependency order for validation so
/// diagnostics report the first invalid declaration. The traversal needs its
/// own deterministic lexical view after that validation has succeeded.
struct ServiceTraversalEdges {
    // 每个节点的边区间由相邻起点界定，末尾额外起点等于目标数组长度。
    starts: Vec<usize>,
    targets: Vec<usize>,
}

/// 校验模块依赖并返回提供方先于使用方的启动名称列表。
///
/// # 参数与返回
///
/// `descriptors` 仅被借用；返回的名称由结果独立持有。重复模块/依赖、缺失目标、
/// 依赖更晚初始化层级或存在环时返回相应 CoreError，不执行生命周期回调。
///
/// # 顺序与边界
///
/// 先按初始化层级和输入位置选遍历根，再沿各模块原有依赖顺序进行显式栈遍历。
/// 本入口只校验模块图；服务依赖、注册名归属与服务种类还须经过注册/完整冻结流程。
pub fn sort_module_activation_order(descriptors: &[ModuleDescriptor]) -> CoreResult<Vec<String>> {
    let mut by_name: HashMap<&str, usize> = HashMap::with_capacity(descriptors.len());
    for (index, descriptor) in descriptors.iter().enumerate() {
        if by_name.insert(descriptor.name.as_str(), index).is_some() {
            return Err(CoreError::DuplicateModule(descriptor.name.clone()));
        }
    }

    for descriptor in descriptors {
        let mut declared_dependencies =
            HashSet::with_capacity(descriptor.module_dependencies.len());
        for dependency in &descriptor.module_dependencies {
            if !declared_dependencies.insert(dependency.module_name.as_str()) {
                return Err(CoreError::DuplicateModuleDependency {
                    module: descriptor.name.clone(),
                    dependency: dependency.module_name.clone(),
                });
            }
            let Some(&dependency_index) = by_name.get(dependency.module_name.as_str()) else {
                return Err(CoreError::MissingModuleDependency {
                    module: descriptor.name.clone(),
                    dependency: dependency.module_name.clone(),
                });
            };
            let dependency_descriptor = &descriptors[dependency_index];
            if dependency_descriptor.init_level > descriptor.init_level {
                return Err(CoreError::ModuleInitLevelViolation {
                    module: descriptor.name.clone(),
                    module_level: descriptor.init_level.as_str().to_owned(),
                    dependency: dependency_descriptor.name.clone(),
                    dependency_level: dependency_descriptor.init_level.as_str().to_owned(),
                });
            }
        }
    }

    let mut traversal: Vec<usize> = (0..descriptors.len()).collect();
    traversal.sort_by_key(|&index| (descriptors[index].init_level, index));

    let mut states = vec![None; descriptors.len()];
    let mut frames = Vec::new();
    let mut order = Vec::with_capacity(descriptors.len());
    for index in traversal {
        if states[index].is_some() {
            continue;
        }
        visit_module_iterative(
            index,
            descriptors,
            &by_name,
            &mut states,
            &mut frames,
            &mut order,
        )?;
    }
    Ok(order)
}

fn module_dependency_map(descriptors: &[ModuleDescriptor]) -> HashMap<String, Arc<[String]>> {
    descriptors
        .iter()
        .map(|descriptor| {
            (
                descriptor.name.clone(),
                descriptor
                    .module_dependencies
                    .iter()
                    .map(|dependency| dependency.module_name.clone())
                    .collect::<Vec<_>>()
                    .into(),
            )
        })
        .collect()
}

fn module_dependent_map(
    module_dependencies: &HashMap<String, Arc<[String]>>,
    module_activation_order: &[String],
) -> HashMap<String, Arc<[String]>> {
    let mut dependents = module_activation_order
        .iter()
        .map(|module_name| (module_name.clone(), Vec::new()))
        .collect::<HashMap<_, _>>();
    for module_name in module_activation_order {
        let dependencies = module_dependencies
            .get(module_name)
            .expect("validated module graph must contain every module");
        for dependency in dependencies.iter() {
            dependents
                .get_mut(dependency)
                .expect("validated dependency must be a graph module")
                .push(module_name.clone());
        }
    }
    dependents
        .into_iter()
        .map(|(module_name, entries)| (module_name, entries.into()))
        .collect()
}

fn collect_service_nodes(
    descriptors: &[ModuleDescriptor],
) -> CoreResult<BTreeMap<String, ServiceGraphNode>> {
    let mut nodes = BTreeMap::new();
    for descriptor in descriptors {
        for driver in &descriptor.drivers {
            insert_service_node(
                &mut nodes,
                &descriptor.name,
                ServiceKind::Driver,
                driver.name.clone(),
                driver.startup_mode,
                &driver.dependencies,
            )?;
        }
        for manager in &descriptor.managers {
            insert_service_node(
                &mut nodes,
                &descriptor.name,
                ServiceKind::Manager,
                manager.name.clone(),
                manager.startup_mode,
                &manager.dependencies,
            )?;
        }
        for plugin in &descriptor.plugins {
            insert_service_node(
                &mut nodes,
                &descriptor.name,
                ServiceKind::Plugin,
                plugin.name.clone(),
                plugin.startup_mode,
                &plugin.dependencies,
            )?;
        }
    }
    Ok(nodes)
}

fn insert_service_node(
    nodes: &mut BTreeMap<String, ServiceGraphNode>,
    owner_module: &str,
    kind: ServiceKind,
    name: RegistryName,
    startup_mode: StartupMode,
    dependencies: &[super::DependencySpec],
) -> CoreResult<()> {
    let key = name.to_string();
    if nodes.contains_key(&key) {
        return Err(CoreError::DuplicateService(key));
    }
    nodes.insert(
        key,
        ServiceGraphNode {
            name,
            owner_module: owner_module.to_owned(),
            kind,
            startup_mode,
            dependencies: dependencies
                .iter()
                .map(|dependency| dependency.name.clone())
                .collect(),
        },
    );
    Ok(())
}

fn validate_service_dependencies(
    nodes: &BTreeMap<String, ServiceGraphNode>,
    module_dependencies: &HashMap<String, Arc<[String]>>,
) -> CoreResult<()> {
    for node in nodes.values() {
        let mut declared_dependencies = HashSet::with_capacity(node.dependencies.len());
        for dependency in &node.dependencies {
            let dependency_name = dependency.to_string();
            if !declared_dependencies.insert(dependency_name.clone()) {
                return Err(CoreError::DuplicateServiceDependency {
                    service: node.name.to_string(),
                    dependency: dependency_name,
                });
            }
            let Some(dependency_node) = nodes.get(dependency.as_str()) else {
                return Err(CoreError::MissingServiceDependency {
                    service: node.name.to_string(),
                    dependency: dependency_name,
                });
            };
            if !service_kind_can_depend_on(node.kind, dependency_node.kind) {
                return Err(CoreError::InvalidServiceDependencyKind {
                    service: node.name.to_string(),
                    service_kind: node.kind,
                    dependency: dependency_node.name.to_string(),
                    dependency_kind: dependency_node.kind,
                });
            }
            // 跨模块服务边只由所属模块的直接依赖声明授权；传递可达不替代显式边。
            if node.owner_module != dependency_node.owner_module {
                let declared_modules = module_dependencies
                    .get(node.owner_module.as_str())
                    .expect("validated module graph must contain every service owner");
                if !declared_modules
                    .iter()
                    .any(|module_name| module_name == &dependency_node.owner_module)
                {
                    return Err(CoreError::UndeclaredCrossModuleServiceDependency {
                        service: node.name.to_string(),
                        service_module: node.owner_module.clone(),
                        dependency: dependency_node.name.to_string(),
                        dependency_module: dependency_node.owner_module.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn service_kind_can_depend_on(service: ServiceKind, dependency: ServiceKind) -> bool {
    service_kind_rank(dependency) <= service_kind_rank(service)
}

fn service_kind_rank(kind: ServiceKind) -> u8 {
    match kind {
        ServiceKind::Driver => 0,
        ServiceKind::Manager => 1,
        ServiceKind::Plugin => 2,
    }
}

fn sort_service_activation_order(
    nodes: &BTreeMap<String, ServiceGraphNode>,
) -> CoreResult<Vec<RegistryName>> {
    let names = nodes.keys().map(String::as_str).collect::<Vec<_>>();
    let by_name = names
        .iter()
        .enumerate()
        .map(|(index, name)| (*name, index))
        .collect::<HashMap<_, _>>();
    let traversal_edges = sorted_service_traversal_edges(&names, nodes, &by_name);
    let mut states = vec![None; names.len()];
    let mut frames = Vec::new();
    let mut order = Vec::with_capacity(nodes.len());
    for index in 0..names.len() {
        if states[index].is_some() {
            continue;
        }
        visit_service_iterative(
            index,
            &names,
            &traversal_edges,
            nodes,
            &mut states,
            &mut frames,
            &mut order,
        )?;
    }
    Ok(order)
}

fn sorted_service_traversal_edges(
    names: &[&str],
    nodes: &BTreeMap<String, ServiceGraphNode>,
    by_name: &HashMap<&str, usize>,
) -> ServiceTraversalEdges {
    let dependency_count = nodes
        .values()
        .map(|node| node.dependencies.len())
        .sum::<usize>();
    let mut starts = Vec::with_capacity(names.len() + 1);
    let mut targets = Vec::with_capacity(dependency_count);

    for name in names {
        let start = targets.len();
        starts.push(start);
        let node = nodes
            .get(*name)
            .expect("validated service graph must contain every traversal node");
        targets.extend(node.dependencies.iter().map(|dependency| {
            *by_name
                .get(dependency.as_str())
                .expect("validated service graph must contain every dependency")
        }));
        targets[start..].sort_unstable_by(|left, right| names[*left].cmp(names[*right]));
    }
    starts.push(targets.len());

    ServiceTraversalEdges { starts, targets }
}

// 按已排序的服务边使用显式帧进行深度优先遍历；遇到当前栈内节点时返回首尾重复的环路径。
fn visit_service_iterative(
    root_index: usize,
    names: &[&str],
    traversal_edges: &ServiceTraversalEdges,
    nodes: &BTreeMap<String, ServiceGraphNode>,
    states: &mut [Option<VisitState>],
    frames: &mut Vec<TraversalFrame>,
    order: &mut Vec<RegistryName>,
) -> CoreResult<()> {
    states[root_index] = Some(VisitState::Visiting);
    frames.push(TraversalFrame {
        node_index: root_index,
        next_dependency_index: 0,
    });

    while !frames.is_empty() {
        let dependency_index = {
            let frame = frames
                .last_mut()
                .expect("a non-empty traversal frame stack must have a service frame");
            let index = frame.node_index;
            let start = traversal_edges.starts[index];
            let end = traversal_edges.starts[index + 1];
            let dependency_index = (start + frame.next_dependency_index < end).then(|| {
                let dependency_index = traversal_edges.targets[start + frame.next_dependency_index];
                frame.next_dependency_index += 1;
                dependency_index
            });
            dependency_index
        };
        if let Some(dependency_index) = dependency_index {
            match states[dependency_index] {
                Some(VisitState::Visited) => continue,
                Some(VisitState::Visiting) => {
                    let cycle_start = frames
                        .iter()
                        .position(|entry| entry.node_index == dependency_index)
                        .expect("visiting service must be present in the traversal frames");
                    let mut path = frames[cycle_start..]
                        .iter()
                        .map(|entry| names[entry.node_index].to_owned())
                        .collect::<Vec<_>>();
                    path.push(names[dependency_index].to_owned());
                    return Err(CoreError::ServiceDependencyCycle { path });
                }
                None => {
                    states[dependency_index] = Some(VisitState::Visiting);
                    frames.push(TraversalFrame {
                        node_index: dependency_index,
                        next_dependency_index: 0,
                    });
                }
            }
            continue;
        }

        let completed_index = frames
            .pop()
            .expect("a non-empty traversal frame stack must pop a service frame")
            .node_index;
        states[completed_index] = Some(VisitState::Visited);
        order.push(
            nodes
                .get(names[completed_index])
                .expect("validated service graph must contain every traversal node")
                .name
                .clone(),
        );
    }

    Ok(())
}

fn module_service_plans(
    module_activation_order: &[String],
    service_activation_order: &[RegistryName],
    nodes: &BTreeMap<String, ServiceGraphNode>,
) -> HashMap<String, FrozenModuleServices> {
    module_activation_order
        .iter()
        .map(|module_name| {
            let service_names = service_activation_order
                .iter()
                .filter(|service_name| {
                    nodes
                        .get(service_name.as_str())
                        .expect("topologically ordered service must retain graph metadata")
                        .owner_module
                        == *module_name
                })
                .cloned()
                .collect::<Vec<_>>();
            let startup_service_names = service_names
                .iter()
                .filter(|service_name| {
                    nodes
                        .get(service_name.as_str())
                        .expect("topologically ordered service must retain graph metadata")
                        .startup_mode
                        == StartupMode::Immediate
                })
                .cloned()
                .collect::<Vec<_>>();
            let shutdown_service_names = service_names.iter().rev().cloned().collect::<Vec<_>>();
            (
                module_name.clone(),
                FrozenModuleServices {
                    service_names: service_names.into(),
                    startup_service_names: startup_service_names.into(),
                    shutdown_service_names: shutdown_service_names.into(),
                },
            )
        })
        .collect()
}

// 模块边保留声明顺序，显式帧与状态数组区分当前路径和已完成节点，避免递归依赖本机栈。
fn visit_module_iterative(
    root_index: usize,
    descriptors: &[ModuleDescriptor],
    by_name: &HashMap<&str, usize>,
    states: &mut [Option<VisitState>],
    frames: &mut Vec<TraversalFrame>,
    order: &mut Vec<String>,
) -> CoreResult<()> {
    states[root_index] = Some(VisitState::Visiting);
    frames.push(TraversalFrame {
        node_index: root_index,
        next_dependency_index: 0,
    });

    while !frames.is_empty() {
        let (index, next_dependency_index) = {
            let frame = frames
                .last_mut()
                .expect("a non-empty traversal frame stack must have a module frame");
            let index = frame.node_index;
            let next_dependency_index = descriptors[index]
                .module_dependencies
                .get(frame.next_dependency_index)
                .map(|_| frame.next_dependency_index);
            if next_dependency_index.is_some() {
                frame.next_dependency_index += 1;
            }
            (index, next_dependency_index)
        };
        if let Some(next_dependency_index) = next_dependency_index {
            let dependency = &descriptors[index].module_dependencies[next_dependency_index];
            let Some(&dependency_index) = by_name.get(dependency.module_name.as_str()) else {
                return Err(CoreError::MissingModuleDependency {
                    module: descriptors[index].name.clone(),
                    dependency: dependency.module_name.clone(),
                });
            };
            match states[dependency_index] {
                Some(VisitState::Visited) => continue,
                Some(VisitState::Visiting) => {
                    let cycle_start = frames
                        .iter()
                        .position(|entry| entry.node_index == dependency_index)
                        .expect("visiting module must be present in the traversal frames");
                    let mut path = frames[cycle_start..]
                        .iter()
                        .map(|entry| descriptors[entry.node_index].name.clone())
                        .collect::<Vec<_>>();
                    path.push(descriptors[dependency_index].name.clone());
                    return Err(CoreError::ModuleDependencyCycle { path });
                }
                None => {
                    states[dependency_index] = Some(VisitState::Visiting);
                    frames.push(TraversalFrame {
                        node_index: dependency_index,
                        next_dependency_index: 0,
                    });
                }
            }
            continue;
        }

        let completed_index = frames
            .pop()
            .expect("a non-empty traversal frame stack must pop a module frame")
            .node_index;
        states[completed_index] = Some(VisitState::Visited);
        order.push(descriptors[completed_index].name.clone());
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/module_order_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "module_order/tests/perf.rs"]
mod optimization_batch_im_runtime623_tests;
