use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;

use nohash_hasher::BuildNoHashHasher;
use pyo3::create_exception;
use pyo3::exceptions;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::{PyAny, Python};

mod hashedany;
use crate::hashedany::HashedAny;

create_exception!(graphlib2, CycleError, exceptions::PyValueError);

#[derive(Clone, Copy)]
enum NodeState {
    Active,
    Ready,
    Done,
}

#[derive(Clone)]
struct NodeInfo {
    state: NodeState,
    npredecessors: usize,
}

#[derive(Clone)]
struct UnpreparedState {
    id2nodeinfo: Vec<NodeInfo>,
    id2node: Vec<HashedAny>,
    node2id: HashMap<HashedAny, usize, BuildNoHashHasher<isize>>,
    parents: Vec<Vec<usize>>,
}

impl UnpreparedState {
    fn new_node(&mut self, node: &HashedAny) -> usize {
        let node_id = self.node2id.len();
        let nodeinfo = NodeInfo {
            state: NodeState::Active,
            npredecessors: 0,
        };
        self.id2node.insert(node_id, node.clone());
        self.node2id.insert(node.clone(), node_id);
        self.id2nodeinfo.insert(node_id, nodeinfo);
        self.parents.insert(node_id, Vec::new());
        node_id
    }
    fn get_or_insert_node_id(&mut self, node: &HashedAny) -> usize {
        if let Some(&v) = self.node2id.get(node) {
            return v;
        }
        self.new_node(node)
    }
    fn add_node(&mut self, node: HashedAny, children: Vec<HashedAny>) -> PyResult<()> {
        // Insert if it doesn't exist
        let node_id = self.get_or_insert_node_id(&node);
        let mut child_id: usize;
        self.id2nodeinfo.get_mut(node_id).unwrap().npredecessors += children.len();
        for child in children.into_iter() {
            child_id = self.get_or_insert_node_id(&child);
            self.parents.get_mut(child_id).unwrap().push(node_id);
        }
        Ok(())
    }
    fn find_cycle(&self) -> Option<Vec<usize>> {
        // Do a DFS with backtracking to find any cycles
        let mut seen: HashSet<usize> = HashSet::new();
        let mut stack = Vec::new();
        let mut itstack = Vec::new();
        let mut node2stackid = HashMap::new();
        let mut node: usize;

        for &n in self.node2id.values() {
            node = n;
            if seen.contains(&node) {
                continue;
            }
            'outer: loop {
                if seen.contains(&node) {
                    // If this node is in the current stack, we have a cycle
                    if node2stackid.contains_key(&node) {
                        let start_id = node2stackid.get(&node).unwrap();
                        let mut res = stack[*start_id..].to_vec();
                        res.push(node);
                        return Some(res);
                    }
                } else {
                    seen.insert(node);
                    itstack.push(self.parents.get(node).unwrap().iter());
                    node2stackid.insert(node, stack.len());
                    stack.push(node);
                }
                // Backtrack to the topmost stack entry with at least 1 parent
                loop {
                    if stack.is_empty() {
                        break 'outer;
                    }
                    match itstack.last_mut().unwrap().next() {
                        Some(parent) => {
                            node = *parent;
                            break;
                        }
                        None => {
                            node2stackid.remove(&stack.pop().unwrap());
                            itstack.pop();
                            continue;
                        }
                    }
                }
            }
        }
        None
    }
}

#[derive(Clone)]
struct SolvedDAG {
    // "Immutable" fields that can be shared
    id2node: Vec<HashedAny>,
    node2id: HashMap<HashedAny, usize, BuildNoHashHasher<isize>>,
    parents: Vec<Vec<usize>>,
}

#[derive(Clone)]
struct PreparedState {
    dag: SolvedDAG,
    // "Mutable" fields that need to be copied
    ready_nodes: VecDeque<usize>,
    id2nodeinfo: Vec<NodeInfo>,
    n_passed_out: usize,
    n_finished: usize,
}

impl PreparedState {
    fn get_ready<'py>(&mut self, py: Python<'py>) -> &'py PyTuple {
        let id2node = &self.dag.id2node;
        self.n_passed_out += self.ready_nodes.len();
        PyTuple::new(
            py,
            self.ready_nodes
                .drain(..)
                .map(|n| id2node.get(n).unwrap().0.as_ref(py)),
        )
    }
    fn is_active(&self) -> bool {
        self.n_finished < self.n_passed_out || !self.ready_nodes.is_empty()
    }
    fn mark_nodes_as_done(&mut self, nodes: Vec<usize>) -> PyResult<()> {
        let mut nodeinfo;
        let mut parent_info;
        let parents = &self.dag.parents;
        let id2nodeinfo = &mut self.id2nodeinfo;
        for node in nodes.into_iter() {
            nodeinfo = id2nodeinfo.get_mut(node).unwrap();
            match nodeinfo.state {
                NodeState::Active => {
                    let pynode = self.dag.id2node.get(node).unwrap();
                    return Err(exceptions::PyValueError::new_err(format!(
                        "node {} was not passed out (still not ready)",
                        hashed_node_to_str(pynode)?
                    )));
                }
                NodeState::Done => {
                    let pynode = self.dag.id2node.get(node).unwrap();
                    return Err(exceptions::PyValueError::new_err(format!(
                        "node {} was already marked as done",
                        hashed_node_to_str(pynode)?
                    )));
                }
                NodeState::Ready => nodeinfo.state = NodeState::Done,
            };
            self.n_finished += 1;
            // Find all parents and reduce their dependency count by one,
            // returning all parents w/o any further dependencies
            for &parent in parents.get(node).unwrap() {
                parent_info = id2nodeinfo.get_mut(parent).unwrap();
                parent_info.npredecessors -= 1;
                if parent_info.npredecessors == 0 {
                    parent_info.state = NodeState::Ready;
                    self.ready_nodes.push_back(parent);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
enum State {
    Unprepared(UnpreparedState),
    Prepared(PreparedState),
}

#[pyclass(module = "graphlib2")]
#[derive(Clone)]
struct TopologicalSorter {
    state: State,
}

#[pymethods]
impl TopologicalSorter {
    // Add a new node to the graph
    fn add(&mut self, node: HashedAny, predecessors: Vec<HashedAny>) -> PyResult<()> {
        match &mut self.state {
            State::Unprepared(state) => state.add_node(node, predecessors),
            State::Prepared(_) => Err(exceptions::PyValueError::new_err(
                "Nodes cannot be added after a call to prepare()",
            )),
        }
    }
    // Check for cycles and gather leafs
    fn prepare(&mut self) -> PyResult<()> {
        let state = match &mut self.state {
            State::Prepared(_) => {
                return Err(exceptions::PyValueError::new_err(
                    "cannot prepare() more than once",
                ))
            }
            State::Unprepared(state) => state,
        };
        let mut ready_nodes = VecDeque::with_capacity(state.node2id.len());
        if let Some(cycle) = state.find_cycle() {
            let nodes_in_cyle: Vec<HashedAny> = cycle
                .into_iter()
                .map(|n| state.id2node.get(n).unwrap().clone())
                .collect();
            let items_str: PyResult<Vec<String>> = nodes_in_cyle
                .iter()
                .map(|n| hashed_node_to_str(n))
                .collect();
            let py_items: Vec<Py<PyAny>> = nodes_in_cyle.iter().map(|n| n.0.clone()).collect();
            return Err(CycleError::new_err((
                format!("Nodes are in a cycle [{}]", items_str?.join(" -> ")),
                py_items,
            )));
        }
        for (node, nodeinfo) in state.id2nodeinfo.iter_mut().enumerate() {
            if nodeinfo.npredecessors == 0 {
                ready_nodes.push_back(node);
                nodeinfo.state = NodeState::Ready;
            }
        }
        self.state = State::Prepared(PreparedState {
            dag: SolvedDAG {
                id2node: state.id2node.clone(),
                node2id: state.node2id.clone(),
                parents: state.parents.clone(),
            },
            ready_nodes,
            id2nodeinfo: state.id2nodeinfo.clone(),
            n_passed_out: 0,
            n_finished: 0,
        });
        Ok(())
    }
    #[new]
    fn new() -> Self {
        TopologicalSorter {
            state: State::Unprepared(UnpreparedState {
                id2nodeinfo: Vec::new(),
                id2node: Vec::new(),
                node2id: HashMap::default(),
                parents: Vec::new(),
            }),
        }
    }
    /// Returns string representation of the graph
    fn __str__(&self) -> PyResult<String> {
        Ok("TopologicalSorter()".to_string())
    }
    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
    /// Returns a deep copy of this graph
    fn copy(&self) -> TopologicalSorter {
        self.clone()
    }
    /// Mark nodes as done and possibly free up their dependents
    /// # Arguments
    ///
    /// * `nodes` - Python objects representing nodes in the graph
    fn done(&mut self, nodes: &PyTuple) -> PyResult<()> {
        let state = match &mut self.state {
            State::Prepared(state) => state,
            State::Unprepared(_) => {
                return Err(exceptions::PyValueError::new_err(
                    "prepare() must be called first",
                ))
            }
        };
        let mut node_ids = Vec::with_capacity(nodes.len());
        let mut hashed_node;
        for node in nodes {
            hashed_node = HashedAny::extract(node)?;
            match state.dag.node2id.get(&hashed_node) {
                Some(&v) => node_ids.push(v),
                None => {
                    return Err(PyValueError::new_err(format!(
                        "node {} was not added using add()",
                        hashed_node_to_str(&hashed_node)?
                    )))
                }
            };
        }
        state.mark_nodes_as_done(node_ids)
    }
    fn is_active(&self) -> PyResult<bool> {
        match &self.state {
            State::Prepared(state) => Ok(state.is_active()),
            State::Unprepared(_) => Err(exceptions::PyValueError::new_err(
                "prepare() must be called first",
            )),
        }
    }
    /// Returns all nodes with no dependencies
    fn get_ready<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyTuple> {
        let state = match &mut self.state {
            State::Prepared(state) => state,
            State::Unprepared(_) => {
                return Err(exceptions::PyValueError::new_err(
                    "prepare() must be called first",
                ))
            }
        };
        Ok(state.get_ready(py))
    }
}

#[pymodule]
fn _graphlib2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TopologicalSorter>()?;
    m.add("CycleError", _py.get_type::<CycleError>())?;
    Ok(())
}

// Misc helper methods
fn hashed_node_to_str(node: &HashedAny) -> PyResult<String> {
    Python::with_gil(|py| -> PyResult<String> {
        Ok(node.0.as_ref(py).repr()?.to_str()?.to_string())
    })
}

// Use the result of calling repr() on the Python object as the debug string value
impl fmt::Debug for HashedAny {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hashed_node_to_str(self).unwrap())
    }
}
