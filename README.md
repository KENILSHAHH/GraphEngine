

# 🧠 succinctGraph - A Minimal Computational Graph Engine in Rust

This project implements a minimal computational graph system for arithmetic expressions and value propagation. It's built in Rust and is designed to support basic arithmetic, constraint validation, hinting for complex operations, and graph visualization via Graphviz.

---

## 📦 Features

- 🔢 Node-based arithmetic graph construction
- ➕ Addition and multiplication operations
- 💡 Custom user-defined computations via hints
- ✅ Equality constraint checking
- 🧪 Built-in unit tests for correctness
- 📈 Graph export in DOT format (for Graphviz)

---

## Run the Project 
```bash
cargo run
```

## 🧰 Data Structures

### `NodeId(usize)`
- Unique identifier for each node in the graph.

### `Node`
Represents an individual node in the computation graph.

| Field    | Type         | Description                               |
|----------|--------------|-------------------------------------------|
| `id`     | `NodeId`     | Unique identifier                         |
| `value`  | `Option<u32>`| Holds the computed value if available     |
| `op`     | `Option<Op>` | Describes how this node is computed       |

### `Op`
Enum defining the operation for a node:

- `Const(u32)`: A constant value
- `Add(NodeId, NodeId)`: Sum of two nodes
- `Mul(NodeId, NodeId)`: Product of two nodes
- `Hint(Vec<NodeId>, HintFn)`: Custom computation over inputs

### `HintFn`
Wraps a user-supplied function: `fn(&[u32]) -> u32`

---

## 🧠 Builder API

### `Builder::new()`
Creates a new computational graph builder.

### `init() -> Node`
Initializes a node with no operation (to be set as input).

### `constant(value: u32) -> Node`
Creates a constant-value node.

### `add(&Node, &Node) -> Node`
Creates a node that computes the sum of two nodes.

### `mul(&Node, &Node) -> Node`
Creates a node that computes the product of two nodes.

### `hint(Vec<Node>, fn(&[u32]) -> u32) -> Node`
Defines a custom computation node using external logic.

### `assert_equal(&Node, &Node)`
Adds a constraint that the two nodes must have equal values.

### `fill_nodes(inputs: HashMap<NodeId, u32>)`
Propagates values throughout the graph, computing derived values from inputs.

### `check_constraints() -> bool`
Validates that all equality constraints hold.

### `to_dot() -> Result<()>`
Exports the current graph structure in DOT format as `graph.dot` (for Graphviz).

---

## 📊 Visualization

Run the program and generate a graph representation:

```bash
cargo run
dot -Tpng graph.dot -o graph.png
open graph.png  # macOS
```

> Requires Graphviz installed (`brew install graphviz` on macOS)

---

## 🧪 Example: f(x) = x^2 + x + 5

```rust
let x = builder.init();
let x_squared = builder.mul(&x, &x);
let five = builder.constant(5);
let sum = builder.add(&x_squared, &x);
let output = builder.add(&sum, &five);

let mut inputs = HashMap::new();
inputs.insert(x.id, 3);

builder.fill_nodes(inputs);
assert!(builder.check_constraints());
```



## 📁 Project Structure

```
succintGraph/
├── src/
│   └── main.rs
├── graph.dot          # Auto-generated visual output
├── README.md
├── Cargo.toml
```

---

## 🧪 Running Tests

```bash
cargo test
```

Includes tests for:
- Polynomial expressions
- Division using hints
- Square root via hints and constraint validation

---
