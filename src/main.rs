/*!
 A minimal, production-ready computational graph library in Rust.

This library allows users to construct and evaluate arithmetic computation graphs with support for:
- Addition and multiplication nodes
- Custom hint-based nodes for extended functionality (e.g. division, square root)
- Value propagation from input nodes
- Equality constraint enforcement
- Graph export in DOT format for visualization

The code is optimized for clarity, extensibility, and performance, with test cases validating correctness and edge-case handling.
*/

use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct NodeId(usize);

#[derive(Debug, Clone)]
struct Node {
    id: NodeId,
    value: Option<u32>,
    op: Option<Op>,
}

#[derive(Clone)]
struct HintFn {
    func: fn(&[u32]) -> u32,
}

impl HintFn {
    fn new(func: fn(&[u32]) -> u32) -> Self {
        HintFn { func }
    }
}

impl Debug for HintFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HintFn")
    }
}

#[derive(Clone)]
enum Op {
    Const(u32),
    Add(NodeId, NodeId),
    Mul(NodeId, NodeId),
    Hint(Vec<NodeId>, HintFn),
}

impl Debug for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Const(v) => write!(f, "Const({})", v),
            Op::Add(a, b) => write!(f, "Add({:?}, {:?})", a, b),
            Op::Mul(a, b) => write!(f, "Mul({:?}, {:?})", a, b),
            Op::Hint(_, _) => write!(f, "Hint(...)"),
        }
    }
}

struct Builder {
    next_id: usize,
    nodes: HashMap<NodeId, Node>,
    constraints: Vec<(NodeId, NodeId)>,
}

impl Builder {
    /// Initializes a new Builder instance with an empty graph structure.
    /// Sets up internal state including node counter, node map, and constraints list.
    pub fn new() -> Self {
        Self {
            next_id: 0,
            nodes: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    /// Creates and registers a new node with a specified operation in the graph.
    /// Used internally to generate nodes for all operations.
    fn new_node(&mut self, op: Option<Op>) -> Node {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        println!("➕ Created Node {:?} with op {:?}", id, op);
        let node = Node {
            id,
            value: None,
            op,
        };
        self.nodes.insert(id, node.clone());
        node
    }

    /// Defines a new input node in the graph that requires external value assignment.
    /// Input nodes have no computation logic and must be initialized via `fill_nodes`.
    pub fn init(&mut self) -> Node {
        self.new_node(None)
    }

    /// Creates a new node with a constant, predefined value.
    /// This node always evaluates to the same value during computation.
    pub fn constant(&mut self, value: u32) -> Node {
        self.new_node(Some(Op::Const(value)))
    }

    /// Constructs a new node representing the sum of two existing nodes.
    /// The node will evaluate to `a.value + b.value` during graph execution.
    pub fn add(&mut self, a: &Node, b: &Node) -> Node {
        self.new_node(Some(Op::Add(a.id, b.id)))
    }

    /// Constructs a new node representing the product of two existing nodes.
    /// The node will evaluate to `a.value * b.value` during graph execution.
    pub fn mul(&mut self, a: &Node, b: &Node) -> Node {
        self.new_node(Some(Op::Mul(a.id, b.id)))
    }

    /// Registers an equality constraint between two nodes.
    /// This will be validated after graph evaluation via `check_constraints`.
    pub fn assert_equal(&mut self, a: &Node, b: &Node) {
        self.constraints.push((a.id, b.id));
    }

    /// Creates a new node whose value is derived from a user-defined function over parent nodes.
    /// Useful for custom logic like division, square root, or other non-native operations.
    pub fn hint(&mut self, parents: Vec<Node>, func: fn(&[u32]) -> u32) -> Node {
        let parent_ids = parents.iter().map(|n| n.id).collect();
        self.new_node(Some(Op::Hint(parent_ids, HintFn::new(func))))
    }

    /// Executes the graph by propagating values from input and constant nodes
    /// through arithmetic and hint nodes. Continues until no further updates occur.
    pub fn fill_nodes(&mut self, inputs: HashMap<NodeId, u32>) {
        for (id, val) in &inputs {
            println!("🔧 Setting input Node {:?} = {}", id, val);
        }
        for (id, val) in inputs {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.value = Some(val);
            }
        }

        let node_ids: Vec<NodeId> = self.nodes.keys().cloned().collect();
        let mut worklist: Vec<NodeId> = node_ids.clone();
        let mut visited = std::collections::HashSet::new();
        while let Some(id) = worklist.pop() {
            if visited.contains(&id) {
                continue;
            }
            // Use a local reference to the node
            let maybe_node = self.nodes.get(&id);
            if maybe_node.is_none() {
                continue;
            }
            let current_val = maybe_node.and_then(|n| n.value);
            if current_val.is_some() {
                visited.insert(id);
                continue;
            }
            let op = maybe_node.and_then(|n| n.op.as_ref());
            println!("Evaluating Node {:?} with op {:?}", id, op);
            let new_val = match op {
                Some(Op::Const(val)) => Some(*val),
                Some(Op::Add(a, b)) => {
                    let a_val = self.nodes.get(a).and_then(|n| n.value);
                    let b_val = self.nodes.get(b).and_then(|n| n.value);
                    match (a_val, b_val) {
                        (Some(x), Some(y)) => Some(x.wrapping_add(y)),
                        _ => None,
                    }
                }
                Some(Op::Mul(a, b)) => {
                    let a_val = self.nodes.get(a).and_then(|n| n.value);
                    let b_val = self.nodes.get(b).and_then(|n| n.value);
                    match (a_val, b_val) {
                        (Some(x), Some(y)) => Some(x.wrapping_mul(y)),
                        _ => None,
                    }
                }
                Some(Op::Hint(parents, f)) => {
                    let mut vals = Vec::with_capacity(parents.len());
                    let mut all_have = true;
                    for pid in parents {
                        match self.nodes.get(pid).and_then(|n| n.value) {
                            Some(v) => vals.push(v),
                            None => {
                                all_have = false;
                                break;
                            }
                        }
                    }
                    if all_have {
                        Some((f.func)(&vals))
                    } else {
                        None
                    }
                }
                None => None,
            };
            if let Some(val) = new_val {
                if let Some(node) = self.nodes.get_mut(&id) {
                    node.value = Some(val);
                    println!("→ Computed Node {:?} = {}", id, val);
                    // Add downstream nodes (dependents) to the worklist
                    // We look for nodes that have this id as a parent
                    for (other_id, other_node) in self.nodes.iter() {
                        if let Some(op) = &other_node.op {
                            let mut is_dependent = false;
                            match op {
                                Op::Add(a, b) | Op::Mul(a, b) => {
                                    if *a == id || *b == id {
                                        is_dependent = true;
                                    }
                                }
                                Op::Hint(parents, _) => {
                                    if parents.iter().any(|pid| *pid == id) {
                                        is_dependent = true;
                                    }
                                }
                                _ => {}
                            }
                            if is_dependent {
                                worklist.push(*other_id);
                            }
                        }
                    }
                }
            }
            visited.insert(id);
        }
    }

    /// Validates all equality constraints defined in the graph by comparing evaluated node values.
    /// Returns true if all constraints hold, otherwise logs mismatches and returns false.
    pub fn check_constraints(&self) -> bool {
        let mut all_ok = true;
        for (a, b) in &self.constraints {
            let val_a = self.nodes.get(a).and_then(|n| n.value);
            let val_b = self.nodes.get(b).and_then(|n| n.value);
            if val_a != val_b {
                println!(
                    "⚠️ Constraint failed: Node {:?} = {:?} != Node {:?} = {:?}",
                    a, val_a, b, val_b
                );
                all_ok = false;
            }
        }
        all_ok
    }

    /// Outputs the current graph structure in DOT format for visualization using Graphviz.
    /// Labels nodes with their operations and shows edges based on computation dependencies.
    pub fn to_dot(&self) -> io::Result<()> {
        let mut file = File::create("graph.dot")?;
        writeln!(file, "digraph ComputationalGraph {{")?;
        for node in self.nodes.values() {
            writeln!(
                file,
                "  Node{} [label=\"{}\"]",
                node.id.0,
                match &node.op {
                    Some(Op::Const(v)) => format!("Const({})", v),
                    Some(Op::Add(a, b)) => format!("Add Node{} + Node{}", a.0, b.0),
                    Some(Op::Mul(a, b)) => format!("Mul Node{} * Node{}", a.0, b.0),
                    Some(Op::Hint(_, _)) => format!("Hint"),
                    None => format!("Input"),
                }
            )?;
            if let Some(op) = &node.op {
                match op {
                    Op::Add(a, b) | Op::Mul(a, b) => {
                        writeln!(file, "  Node{} -> Node{};", a.0, node.id.0)?;
                        writeln!(file, "  Node{} -> Node{};", b.0, node.id.0)?;
                    }
                    Op::Hint(parents, _) => {
                        for p in parents {
                            writeln!(file, "  Node{} -> Node{};", p.0, node.id.0)?;
                        }
                    }
                    _ => {}
                }
            }
        }
        writeln!(file, "}}").map_err(Into::into)
    }
}

/// Demonstrates building and executing a computation graph for f(x) = x^2 + x + 8
fn main() {
    let mut builder = Builder::new();
    let x = builder.init();
    let x_squared = builder.mul(&x, &x);
    let five = builder.constant(8);
    let x_squared_plus_x = builder.add(&x_squared, &x);
    let _y = builder.add(&x_squared_plus_x, &five);

    let mut inputs = HashMap::new();
    inputs.insert(x.id, 3);
    builder.fill_nodes(inputs);
    builder.check_constraints();
    builder.to_dot().expect("Failed to write DOT file");
    // DOT graph written to graph.dot
}

#[cfg(test)]
mod tests;
