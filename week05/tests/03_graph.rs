//! Run this file with `cargo test --test 03_graph`.

//! implement a directed acyclic graph with dependency tracking
//!
//! Implement a graph represented as a set of nodes that can depend on one another.
//! Each node has both links (pointers) to its dependencies, but also to its dependents (the nodes
//! that depend on it), so that it can access them quickly.
//!
//! It is not possible to represent something like this using references alone.
//! Therefore, this is an exercise for working with `Rc` and `RefCell`.
//!
//! When borrowing the individual nodes, make sure to never borrow the same node mutably more than
//! once, otherwise the code will panic (due to "alias XOR mutate" runtime check in `RefCell`).
//!
//! Question: is it possible to create cycles (except for self-loops) in the graph using the
//! API described below?
//! Answer: No it is no possible due to ready/finished check in finish method.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

/// This is just a type alias, not a newtype.
/// It can be useful to start with it if you want to give a new name
/// to an existing type, but don't want to deal with newtype wrapping yet.
type NodeId = u64;

#[derive(Default)]
struct Graph<T> {
    nodes: HashMap<NodeId, Rc<RefCell<Node<T>>>>,
}

/// Single node of the graph
/// It depends on N other nodes, and M other nodes depend on it.
/// These dependency links are represented directly as pointers, to enable low-latency access.
///
/// When created, a node does not contain any value, thus it is **not finished**.
/// It can become finished by receiving a value.
/// That can only happen if the node is **ready**.
/// A node becomes **ready** once all its `dependencies` become **finished**.
struct Node<T> {
    id: NodeId,
    /// Some -> finished
    /// None -> not finished
    value: Option<T>,
    /// This node depends on the following nodes
    dependencies: Vec<Rc<RefCell<Self>>>,
    /// The following nodes depend on this node
    dependents: Vec<Rc<RefCell<Self>>>,
}

impl<T> Graph<T> {
    /// Add a new node to the graph.
    /// The `dependents` links of all the passed `dependencies` should be updated.
    ///
    /// If there is already a node with the given node ID, the function should panic.
    /// If `dependencies` contains an unknown node ID, the function should panic.
    /// If `dependencies` contain `id`, the function should panic.
    fn add(&mut self, id: NodeId, dependencies: Vec<NodeId>) {
        if self.nodes.contains_key(&id) {
            panic!("Node ID {} already exists", id);
        }

        let mut node_dependencies = Vec::with_capacity(dependencies.len());

        for dependency in &dependencies {
            if *dependency == id {
                panic!(
                    "Dependency Node ID {} is the same as the current Node ID",
                    dependency
                );
            }

            if let Some(dependency) = self.nodes.get(dependency) {
                node_dependencies.push(dependency.clone());
            } else {
                panic!("Unknown node ID {}", dependency);
            }
        }

        let node = Rc::new(RefCell::new(Node::<T> {
            id,
            value: None,
            dependencies: node_dependencies,
            dependents: vec![],
        }));

        for dependency in &node.borrow().dependencies {
            dependency.borrow_mut().dependents.push(node.clone());
        }

        self.nodes.insert(id, node);
    }

    /// Remove a node from the graph.
    /// The `dependencies` and `dependents` links of affected nodes should be updated.
    ///
    /// If the id does not exist, the function should panic.
    fn remove(&mut self, id: NodeId) {
        match self.nodes.remove(&id) {
            Some(node) => {
                for dependency in &node.borrow().dependencies {
                    let mut dependency = dependency.borrow_mut();
                    dependency
                        .dependents
                        .retain(|dependent| dependent.borrow().id != id);
                }

                for dependent in &node.borrow().dependents {
                    let mut dependent = dependent.borrow_mut();
                    dependent
                        .dependencies
                        .retain(|dependency| dependency.borrow().id != id);
                }
            }
            None => panic!("Node ID {} does not exist", id),
        }
    }

    /// Finish the node with the given `id` with the provided `value`.
    /// If the given node is not **ready** (or does not exist), the function should panic.
    ///
    /// Returns node IDs of (directly) dependent tasks that are ready after this operation.
    fn finish(&self, id: NodeId, value: T) -> Vec<NodeId> {
        let Some(node) = self.nodes.get(&id) else {
            panic!("Node ID {} does not exist", id);
        };

        if !self.is_ready(id) {
            panic!("Node ID {} is not ready", id);
        }

        if node.borrow().value.is_some() {
            panic!("Node ID {} is already finished", id);
        }

        node.borrow_mut().value = Some(value);

        let mut ready_dependents = vec![];
        for dependent in &node.borrow().dependents {
            if self.is_ready(dependent.borrow().id) {
                ready_dependents.push(dependent.borrow().id);
            }
        }

        ready_dependents
    }

    /// Returns true if the node with the given `id` is **ready**.
    fn is_ready(&self, id: NodeId) -> bool {
        self.nodes
            .get(&id)
            .map(|node| {
                node.borrow()
                    .dependencies
                    .iter()
                    .all(|dependency| dependency.borrow().value.is_some())
            })
            .unwrap_or(false)
    }

    /// Returns the value within a node with the given `id`.
    fn get_value(&self, id: NodeId) -> Option<T>
    where
        T: Clone,
    {
        self.nodes
            .get(&id)
            .map(|node| node.borrow().value.clone())?
    }

    /// Returns IDs of the direct dependencies of the node with the given `id`.
    fn get_dependencies(&self, id: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(&id)
            .map(|node| {
                node.borrow()
                    .dependencies
                    .iter()
                    .map(|dependency| dependency.borrow().id)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns IDs of nodes that directly depend on the node with the given `id`.
    fn get_dependents(&self, id: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(&id)
            .map(|node| {
                node.borrow()
                    .dependents
                    .iter()
                    .map(|dependent| dependent.borrow().id)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns an iterator over **all** transitive dependencies of the node with the given `id`.
    /// The dependencies should be iterated in breadth-first order (iterate the direct dependencies,
    /// then the direct dependencies of the direct dependencies, etc.).
    /// Each dependency should be returned only once from the iterator, so make sure to filter
    /// duplicates.
    ///
    /// Note that this should be implemented with a separate struct that implements the `Iterator`
    /// trait. Once generators are stabilized, it would also be possible to be implemented directly
    /// within this function :)
    fn dependencies_iter(&self, id: NodeId) -> impl Iterator<Item=NodeId> + '_ {
        struct DependenciesIterator<T> {
            visited: HashSet<NodeId>,
            queue: VecDeque<Rc<RefCell<Node<T>>>>,
        }

        impl<T> Iterator for DependenciesIterator<T> {
            type Item = NodeId;

            fn next(&mut self) -> Option<Self::Item> {
                while let Some(node) = self.queue.pop_front() {
                    if self.visited.insert(node.borrow().id) {
                        self.queue
                            .extend(node.borrow().dependencies.iter().cloned());
                        return Some(node.borrow().id);
                    }
                }

                None
            }
        }

        let visited = HashSet::new();
        let queue = VecDeque::from(
            self.nodes
                .get(&id)
                .map(|node| node.borrow().dependencies.to_vec())
                .unwrap_or_default(),
        );

        DependenciesIterator { visited, queue }
    }

    /// Return the number of nodes in the graph.
    fn len(&self) -> usize {
        self.nodes.len()
    }
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::{Graph, NodeId};
    use std::fmt::Debug;

    #[test]
    fn length() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        graph.add(1, vec![]);
        assert_eq!(graph.len(), 2);
    }

    // Shortened macro name, because `insta::assert_compact_debug_snapshot` is quite long.
    macro_rules! check {
        ($($arg:tt)*) => {
            insta::assert_compact_debug_snapshot!($($arg)*);
        };
    }

    #[test]
    fn add_dependencies() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        graph.add(1, vec![0]);
        check!(node(&graph, 0), @"NodeStats { dependencies: [], dependents: [1], value: None, ready: true }");
        check!(node(&graph, 1), @"NodeStats { dependencies: [0], dependents: [], value: None, ready: false }");
    }

    #[test]
    fn add_dependencies_complex() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        graph.add(1, vec![0]);
        graph.add(2, vec![0]);
        graph.add(3, vec![1, 2]);
        graph.add(4, vec![3, 1, 0]);

        check!(node(&graph, 0), @"NodeStats { dependencies: [], dependents: [1, 2, 4], value: None, ready: true }");
        check!(node(&graph, 1), @"NodeStats { dependencies: [0], dependents: [3, 4], value: None, ready: false }");
        check!(node(&graph, 2), @"NodeStats { dependencies: [0], dependents: [3], value: None, ready: false }");
        check!(node(&graph, 3), @"NodeStats { dependencies: [1, 2], dependents: [4], value: None, ready: false }");
        check!(node(&graph, 4), @"NodeStats { dependencies: [3, 1, 0], dependents: [], value: None, ready: false }");
    }

    #[test]
    fn remove_task() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        graph.add(1, vec![0]);
        graph.add(2, vec![0, 1]);
        graph.add(3, vec![0, 1, 2]);
        graph.remove(1);

        assert_eq!(graph.len(), 3);
        check!(node(&graph, 0), @"NodeStats { dependencies: [], dependents: [2, 3], value: None, ready: true }");
        check!(node(&graph, 2), @"NodeStats { dependencies: [0], dependents: [3], value: None, ready: false }");
        check!(node(&graph, 3), @"NodeStats { dependencies: [0, 2], dependents: [], value: None, ready: false }");
    }

    #[test]
    fn remove_then_add() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        graph.finish(0, 42);
        graph.add(1, vec![0]);
        graph.remove(0);
        graph.add(0, vec![]);

        check!(node(&graph, 0), @"NodeStats { dependencies: [], dependents: [], value: None, ready: true }");
        check!(node(&graph, 1), @"NodeStats { dependencies: [], dependents: [], value: None, ready: true }");
    }

    #[test]
    fn finish_task() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        graph.finish(0, 42);
        assert_eq!(graph.get_value(0), Some(42));
    }

    #[test]
    fn finish_task_string() {
        let mut graph = Graph::<String>::default();
        graph.add(0, vec![]);
        graph.finish(0, String::from("foo"));
        assert_eq!(graph.get_value(0), Some(String::from("foo")));
    }

    #[test]
    #[should_panic]
    fn finish_task_that_is_not_ready() {
        let mut graph = Graph::default();
        graph.add(0, vec![]);
        graph.add(1, vec![0]);

        // This task is not ready, finishing it should thus panic
        graph.finish(1, 1);
    }

    #[test]
    #[should_panic]
    fn add_duplicate_id() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        // Adding the same ID is not allowed
        graph.add(0, vec![]);
    }

    #[test]
    #[should_panic]
    fn remove_non_existent() {
        let mut graph = Graph::<u32>::default();
        // Removing a non-existent node should panic
        graph.remove(0);
    }

    #[test]
    #[should_panic]
    fn finish_twice() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        graph.finish(0, 42);
        // Finishing a task twice should panic
        graph.finish(0, 42);
    }

    #[test]
    #[should_panic]
    fn unknown_dependency() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![1]);
    }

    #[test]
    #[should_panic]
    fn self_link() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![0]);
    }

    #[test]
    fn remove_become_ready() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        graph.add(1, vec![0]);
        graph.add(2, vec![1]);
        graph.remove(1);

        check!(node(&graph, 2), @"NodeStats { dependencies: [], dependents: [], value: None, ready: true }");
    }

    #[test]
    fn mark_readiness() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        graph.add(1, vec![0]);
        graph.add(2, vec![0]);
        graph.add(3, vec![1, 2]);
        graph.add(4, vec![3, 1, 0]);

        let new_ready = graph.finish(0, 42);
        assert_eq!(new_ready, vec![1, 2]);

        let new_ready = graph.finish(2, 50);
        assert_eq!(new_ready, vec![]);

        let new_ready = graph.finish(1, 102);
        assert_eq!(new_ready, vec![3]);

        let new_ready = graph.finish(3, 86);
        assert_eq!(new_ready, vec![4]);

        let new_ready = graph.finish(4, 2);
        assert_eq!(new_ready, vec![]);

        check!(node(&graph, 0), @"NodeStats { dependencies: [], dependents: [1, 2, 4], value: Some(42), ready: true }");
        check!(node(&graph, 1), @"NodeStats { dependencies: [0], dependents: [3, 4], value: Some(102), ready: true }");
        check!(node(&graph, 2), @"NodeStats { dependencies: [0], dependents: [3], value: Some(50), ready: true }");
        check!(node(&graph, 3), @"NodeStats { dependencies: [1, 2], dependents: [4], value: Some(86), ready: true }");
        check!(node(&graph, 4), @"NodeStats { dependencies: [3, 1, 0], dependents: [], value: Some(2), ready: true }");
    }

    #[test]
    fn dependencies_iterator() {
        let mut graph = Graph::<u32>::default();
        graph.add(0, vec![]);
        graph.add(1, vec![0]);
        graph.add(2, vec![0]);
        graph.add(3, vec![1, 2]);
        graph.add(4, vec![3, 1, 0]);
        graph.add(5, vec![3, 4]);
        graph.add(6, vec![1, 5]);

        let deps = graph.dependencies_iter(6);
        assert_eq!(deps.collect::<Vec<_>>(), vec![1, 5, 0, 3, 4, 2]);
    }

    #[derive(Debug)]
    #[allow(unused)]
    struct NodeStats<T> {
        dependencies: Vec<NodeId>,
        dependents: Vec<NodeId>,
        value: Option<T>,
        ready: bool,
    }

    fn node<T>(graph: &Graph<T>, id: NodeId) -> NodeStats<T>
    where
        T: Clone,
    {
        let dependencies = graph.get_dependencies(id);
        let dependents = graph.get_dependents(id);
        let value = graph.get_value(id);
        let ready = graph.is_ready(id);
        NodeStats {
            dependencies,
            dependents,
            value,
            ready,
        }
    }
}
