use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;

use pyo3::create_exception;
use pyo3::exceptions;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyTuple;
use pyo3::{Py, PyAny, Python};
use nohash_hasher::BuildNoHashHasher;


mod hashedany;
use crate::hashedany::HashedAny;

create_exception!(graphlib2, CycleError, exceptions::PyValueError);


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



#[derive(Debug, Clone, Copy)]
enum NodeState {
    Active,
    Ready,
    Done,
}


#[derive(Clone,Debug)]
struct NodeInfo {
    node: HashedAny,
    state: NodeState,
    npredecessors: usize,
}


#[pyclass(module = "graphlib2")]
#[derive(Clone)]
struct TopologicalSorter {
    idx2nodeinfo: HashMap<usize, NodeInfo, BuildNoHashHasher<usize>>,
    node2idx: HashMap<HashedAny, usize>,
    parents: HashMap<usize, Vec<usize>, BuildNoHashHasher<usize>>,
    children: HashMap<usize, Vec<usize>, BuildNoHashHasher<usize>>,
    ready_nodes: VecDeque<usize>,
    n_passed_out: usize,
    n_finished: usize,
    prepared: bool,
    node_idx_counter: usize,
}

impl TopologicalSorter {
    fn mark_node_as_done(
        &mut self,
        node: usize,
        done_queue: Option<&mut VecDeque<usize>>,
    ) -> PyResult<()> {
        // Check that this node is ready to be marked as done and mark it
        // There is currently a remove and an insert here just to take ownership of the value
        // so that we can reference it while modifying other values
        // Maybe there's a better way?
        let nodeinfo = self.idx2nodeinfo.get_mut(&node).unwrap();
        match nodeinfo.state {
            NodeState::Active => {
                return Err(exceptions::PyValueError::new_err(format!(
                    "node {} was not passed out (still not ready)",
                    hashed_node_to_str(&nodeinfo.node)?
                )))
            }
            NodeState::Done => {
                return Err(exceptions::PyValueError::new_err(format!(
                    "node {} was already marked as done",
                    hashed_node_to_str(&nodeinfo.node)?
                )))
            }
            NodeState::Ready => nodeinfo.state = NodeState::Done,
        };
        self.n_finished += 1;
        // Find all parents and reduce their dependency count by one,
        // returning all parents w/o any further dependencies
        let q = match done_queue {
            Some(v) => v,
            None => &mut self.ready_nodes,
        };
        let mut parent_info: &mut NodeInfo;
        for parent in self.parents.get(&node).unwrap() {
            parent_info = self.idx2nodeinfo.get_mut(&parent).unwrap();
            parent_info.npredecessors -= 1;
            if parent_info.npredecessors == 0 {
                parent_info.state = NodeState::Ready;
                q.push_back(*parent);
            }
        }
        Ok(())
    }
    fn new_node(&mut self, node: &HashedAny) -> usize {
        self.node_idx_counter += 1;
        let nodeinfo = NodeInfo {
            node: node.clone(),
            state: NodeState::Active,
            npredecessors: 0,
        };
        self.node2idx.insert(node.clone(), self.node_idx_counter);
        self.idx2nodeinfo.insert(self.node_idx_counter, nodeinfo);
        self.parents.insert(self.node_idx_counter, Vec::new());
        self.children.insert(self.node_idx_counter, Vec::new());
        self.node_idx_counter
    }
    fn get_or_insert_node_idx(&mut self, node: &HashedAny) -> usize {
        match self.node2idx.get(node) {
            Some(&v) => return v,
            None => (),
        }
        self.new_node(node)
    }

    fn add_node(&mut self, node: HashedAny, children: Vec<HashedAny>) -> PyResult<()> {
        // Insert if it doesn't exist
        let nodeidx = self.get_or_insert_node_idx(&node);
        let nodeinfo = self.idx2nodeinfo.get_mut(&nodeidx).unwrap();
        nodeinfo.npredecessors += children.len();
        let mut child_idx: usize;
        for child in children {
            child_idx = self.get_or_insert_node_idx(&child);
            self.parents
                .entry(child_idx)
                .or_insert_with(Vec::new)
                .push(nodeidx);
        }
        Ok(())
    }
    fn find_cycle(&self) -> Option<Vec<usize>> {
        let mut seen: HashSet<usize> = HashSet::new();
        let mut stack = Vec::new();
        let mut itstack = Vec::new();
        let mut node2stackidx: HashMap<usize, usize, BuildNoHashHasher<usize>> = HashMap::with_hasher(BuildNoHashHasher::default());
        let mut node: usize;

        for n in self.idx2nodeinfo.keys() {
            node = *n;
            // // Only begin exploring from root nodes
            // if nodeinfo.parents.len() != 0 {
            //     continue;
            // }
            if seen.contains(&node) {
                continue;
            }
            'outer: loop {
                if seen.contains(&node) {
                    // If this node is in the current stack, we have a cycle
                    if node2stackidx.contains_key(&node) {
                        let start_idx = node2stackidx.get(&node).unwrap();
                        let mut res = stack[*start_idx..].to_vec();
                        res.push(node);
                        return Some(res);
                    }
                } else {
                    seen.insert(node);
                    itstack.push(self.parents.get(&node).unwrap().iter());
                    node2stackidx.insert(node, stack.len());
                    stack.push(node);
                }
                // Backtrack to the topmost stack entry with at least 1 parent
                let mut broke = false;
                while !stack.is_empty() {
                    match itstack.last_mut().unwrap().next() {
                        Some(parent) => {
                            node = *parent;
                            broke = true;
                            break;
                        }
                        None => {
                            node2stackidx.remove(&stack.pop().unwrap());
                            itstack.pop();
                            continue;
                        }
                    }
                }
                if !broke {
                    break 'outer;
                }
            }
        }
        None
    }
}

#[pymethods]
impl TopologicalSorter {
    fn add(&mut self, node: HashedAny, predecessors: Vec<HashedAny>) -> PyResult<()> {
        self.add_node(node, predecessors)?;
        Ok(())
    }
    fn prepare(&mut self) -> PyResult<()> {
        if self.prepared {
            return Err(exceptions::PyValueError::new_err(
                "cannot prepare() more than once",
            ));
        }
        match self.find_cycle() {
            Some(cycle) => {
                let items: PyResult<Vec<String>> = cycle
                    .iter()
                    .map(|n| hashed_node_to_str(&self.idx2nodeinfo.get(&n).unwrap().node))
                    .collect();
                return Err(CycleError::new_err((
                    format!("nodes are in a cycle [{}]", items?.join(", ")),
                    cycle,
                )));
            }
            None => (),
        }
        self.prepared = true;
        for (&node, nodeinfo) in self.idx2nodeinfo.iter_mut() {
            if nodeinfo.npredecessors == 0 {
                self.ready_nodes.push_back(node);
                nodeinfo.state = NodeState::Ready;
            }
        }
        Ok(())
    }
    #[new]
    fn new(graph: Option<&PyDict>) -> PyResult<Self> {
        let mut this = TopologicalSorter {
            idx2nodeinfo: HashMap::with_hasher(BuildNoHashHasher::default()),
            node2idx: HashMap::new(),
            parents: HashMap::with_hasher(BuildNoHashHasher::default()),
            children: HashMap::with_hasher(BuildNoHashHasher::default()),
            ready_nodes: VecDeque::new(),
            n_passed_out: 0,
            n_finished: 0,
            prepared: false,
            node_idx_counter: 0,
        };
        if !graph.is_none() {
            for (node, v) in graph.unwrap().iter() {
                let i = v.iter()?;
                let mut children: Vec<HashedAny> = Vec::new();
                for el in i {
                    children.push(HashedAny::extract(el?)?);
                }
                this.add_node(node.extract()?, children)?;
            }
        }
        Ok(this)
    }
    /// Returns string representation of the graph
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("TopologicalSorter()"))
    }
    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
    /// Returns a deep copy of this graph
    fn copy(&self) -> TopologicalSorter {
        self.clone()
    }
    /// Returns any nodes with no dependencies after marking `node` as done
    /// # Arguments
    ///
    /// * `node` - A node in the graph
    fn done(&mut self, nodes: Vec<HashedAny>) -> PyResult<()> {
        if !self.prepared {
            return Err(exceptions::PyValueError::new_err(
                "prepare() must be called first",
            ));
        }
        let mut nodeidx: usize;
        for node in nodes {
            nodeidx = match self.node2idx.get(&node) {
                Some(&v) => v,
                None => {
                    return Err(PyValueError::new_err(format!(
                        "node {} was not added using add()",
                        hashed_node_to_str(&node)?
                    )))
                }
            };
            self.mark_node_as_done(nodeidx, None)?;
        }
        Ok(())
    }
    fn is_active(&self) -> PyResult<bool> {
        if !self.prepared {
            return Err(exceptions::PyValueError::new_err(
                "prepare() must be called first",
            ));
        }
        Ok(self.n_finished < self.n_passed_out || !self.ready_nodes.is_empty())
    }
    /// Returns all nodes with no dependencies
    fn get_ready<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyTuple> {
        if !self.prepared {
            return Err(exceptions::PyValueError::new_err(
                "prepare() must be called first",
            ));
        }
        let ret = PyTuple::new(
            py,
            self.ready_nodes
                .iter()
                .map(|&node| self.idx2nodeinfo.get(&node).unwrap().node.0.clone()),
        );
        self.n_passed_out += self.ready_nodes.len();
        self.ready_nodes.clear();
        Ok(ret)
    }
    fn static_order<'py>(&mut self) -> PyResult<Vec<Py<PyAny>>> {
        self.prepare()?;
        let mut out = Vec::new();
        let mut queue: VecDeque<usize> = VecDeque::from(self.ready_nodes.clone());
        let mut node: usize;
        loop {
            if queue.is_empty() {
                break;
            }
            node = queue.pop_front().unwrap();
            self.mark_node_as_done(node, Some(&mut queue))?;
            out.push(self.idx2nodeinfo.get(&node).unwrap().node.0.clone());
        }
        self.n_passed_out += out.len();
        Ok(out)
    }
}

#[pymodule]
fn graphlib2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TopologicalSorter>()?;
    m.add("CycleError", _py.get_type::<CycleError>())?;
    Ok(())
}
