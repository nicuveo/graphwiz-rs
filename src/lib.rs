//! # GraphWiz.
//!
//! Builders to generate and render graphs in the [GraphViz DOT
//! format](https://graphviz.org/).
//!
//! ## Example
//!
//! The test file `example.rs` contains the following:
//!
//! ```rust
#![doc = include_str!("../tests/example.rs")]
//! ```
//!
//! This generates the following DOT graph.
//!
//! ```DOT
#![doc = include_str!("../tests/testdata/example.golden")]
//! ```
//!
//! Which DOT renders like this:
//!
#![doc = include_str!("../example.svg")]
//!
//! ## Usage
//!
//! The only way to create a graph is to obtain a [RootBuilder]. You can do so
//! with [Graph::new_builder]. This type implements the [Builder] trait, which
//! lets you create one of the four [kinds][Kind] of graph entities: nodes,
//! edges, subgraphs, or clusters[^1]. Once you are done, a call to
//! [RootBuilder::build] will finalize the build and give you a [Graph], which
//! you can then render using one of the rendering functions such as
//! [render_digraph].
//!
//! ### Entities
//!
//! When created, any element gives you an [Entity], which is a lightweight and
//! opaque identifier. It uniquely identifies said entity within the graph,
//! regardless of its label, and is therefore used to build edges.
//!
//! Nodes and edges return the corresponding [Entity] immediately upon creation,
//! but subgraphs and clusters only do so when their builder is finalized.
//!
//! ### Edges
//!
//! [new_edge][Builder::new_edge] is smart, and will let you link two entities
//! regardless of their [Kind]. If one end of an edge is another edge, the newly
//! created edge will be a "continuation" of the previous one:
//!
//! ```rust
//! use graphwiz::{Graph, Builder};
//!
//! let mut root = Graph::new_builder();
//! let a = root.new_node("a");
//! let b = root.new_node("b");
//! let c = root.new_node("c");
//! let d = root.new_node("d");
//! let ab = root.new_edge(a, b);
//! let cd = root.new_edge(c, d);
//! let bc = root.new_edge(ab, cd);
//! ```
//!
//! If one of the two ends of an edge is a subgraph or a cluster, then the
//! graph's [compound][attributes::COMPOUND] attribute will atumatically be set
//! to "true", and the edge will be altered to match the required DOT syntax:
//! the edge will be between nodes within the given subgraphs / clusters, but
//! the [lhead][attributes::LHEAD] or [ltail][attributes::LTAIL] attributes will
//! be set properly.
//!
//! ### Subgraphs
//!
//! When you call [new_subgraph][Builder::new_subgraph] or
//! [new_cluster][Builder::new_cluster], you obtain a new builder, specifically
//! a [SubgraphBuilder]. It also implements the [Builder] trait, meaning you can
//! use it to build new nodes, edges, subgraphs, or clusters. Each [Builder]
//! creates entities within its own scope.
//!
//! Crucially, every builder holds a mutable reference to the underlying
//! [Graph], meaning *only one builder can be used at a time*. A child builder
//! must be finalized to release its hold on the reference, allowing the parent
//! to be used again. [SubgraphBuilder] implements the [Drop] trait to finalize
//! the build properly, meaning calling [build][SubgraphBuilder::build] isn't
//! required.
//!
//! ### Attributes
//!
//! Each entity has attributes associated to it, which is a simple mapping from
//! [&str] to [String]. The [attributes] module provides constants for [every
//! known attribute](https://graphviz.org/doc/info/attrs.html), but having the
//! key be an arbitrary [&str] makes this more flexible. Some attributes are
//! automatically set, such as a node or a cluster's label.
//!
//! You can provide default attribute values for a given [Kind] of entity using
//! a builder's [defaults][Builder::defaults] functions. Defaults are scoped,
//! meaning that changes made to the defaults in a builder are not forwarded
//! back to its parent, but builders for subgraphs get initialized with a copy
//! of their parent's defaults, which allows you to "scope" them.
//!
//! Any builder has access to the full graph, meaning that you can always use
//! the current builder's [attributes][Builder::attributes] function to access
//! or modify any entity's attributes, even if it was created by a different
//! builder.
//!
//! ### Rendering
//!
//! Each render function takes a [Graph] and creates a [String] that represents
//! said graph. You can choose between rendering the graph as directed or
//! undirected, and optionally as strict. See [GraphViz's
//! documentation](https://graphviz.org/doc/info/lang.html#lexical-and-semantic-notes)
//! for more information about the distinction.
//!
//! [^1]: Clusters are a peculiarity of DOT: they are subgraphs whose name
//! happen to start with "cluster". They are rendered, while other subgraphs are
//! not. This library treats them as different for the purpose of allowing
//! different default attributes for each.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod attributes;
mod builder;
mod graph;
mod render;

pub use builder::*;
pub use graph::*;
pub use render::*;
