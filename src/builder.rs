use std::collections::HashMap;
use std::mem;

use crate::graph::*;

////////////////////////////////////////////////////////////////////////////////
// Public API

/// Builder for the root graph.
///
/// This can only be constructed by `Graph::new_builder`, and implements the
/// `Builder` trait.
#[derive(Debug)]
pub struct RootBuilder {
    graph: Graph,
    current: SubgraphInfo,
    defaults: Defaults,
}

impl RootBuilder {
    /// Finalizes the builder and returns the final graph.
    pub fn build(mut self) -> Graph {
        self.graph.subgraphs.insert(ROOT, self.current);
        self.graph
    }
}

/// Builder for all subgraphs.
///
/// This can be constructed by calling the `new_subgraph` or `new_cluster`
/// functions on a `Builder`. The newly created builder becomes the "active"
/// one, as it will hold the reference to the whole graph; the previous builder
/// becomes active again when its child builder has been dropped and the mutable
/// reference is gone.
#[derive(Debug)]
pub struct SubgraphBuilder<'a> {
    graph: &'a mut Graph,
    entity: Entity,
    current: SubgraphInfo,
    defaults: Defaults,
}

impl SubgraphBuilder<'_> {
    /// Finalizes the builder and returns the corresponding entity.
    ///
    /// This releases the hold that the builder has on the reference to the
    /// graph, allowing its parent to be used again. This relies on the `Drop`
    /// trait.
    pub fn build(self) -> Entity {
        self.entity
    }
}

impl Drop for SubgraphBuilder<'_> {
    /// If a subgraph builder gets dropped without being explicitly finalized
    /// with `build`, we want to ensure that it is properly finalized.
    fn drop(&mut self) {
        self.graph.subgraphs.insert(
            self.entity,
            mem::replace(&mut self.current, SubgraphInfo::new()),
        );
    }
}

/// All required functions to build new graph elements.
pub trait Builder {
    /// Creates a new node within the current scope, with the given label.
    /// Returns the new node's entity, which can be used to alter this node's
    /// attributes.
    fn new_node(&mut self, label: impl Into<String>) -> Entity;

    /// Creates a new edge between the two provided entities.
    ///
    /// If any of the entities is a subgraph or a cluster, then this sets the
    /// "compound" property of the graph to true, and it finds a random node
    /// within the subgraph or cluster to be able to generate the proper edge.
    ///
    /// If any of the entities is an edge, this function "chains" them:
    ///
    ///     use graphwiz::{Graph, Builder};
    ///     let mut builder = Graph::new_builder();
    ///     let a  = builder.new_node("a");
    ///     let b  = builder.new_node("b");
    ///     let c  = builder.new_node("c");
    ///     let d  = builder.new_node("d");
    ///     let ab = builder.new_edge(a, b);   // creates a --> b
    ///     let cd = builder.new_edge(c, d);   // creates c --> d
    ///     let bc = builder.new_edge(ab, cd); // creates b --> c
    ///
    /// Returns the entity of the newly created edge.
    fn new_edge(&mut self, head: Entity, tail: Entity) -> Entity;

    /// Creates a new subgraph within the current scope.
    ///
    /// This function borrows the underlying shared state, meaning that this
    /// builder can no longer be used until the new subgraph builder has been
    /// comsumed with 'build' or has been dropped.
    fn new_subgraph(&mut self) -> SubgraphBuilder;

    /// Creates a new cluster within the current scope with the given label.
    ///
    /// This function borrows the underlying shared state, meaning that this
    /// builder can no longer be used until the new subgraph builder has been
    /// comsumed with 'build' or has been dropped.
    fn new_cluster(&mut self, label: impl Into<String>) -> SubgraphBuilder;

    /// Like 'new_node' but takes attributes to add to the default as an argument.
    fn new_node_with(&mut self, label: impl Into<String>, attribs: Attributes) -> Entity {
        let entity = self.new_node(label);
        self.attributes_mut(entity).extend(attribs);
        entity
    }

    /// Like 'new_edge' but takes attributes to add to the default as an argument.
    fn new_edge_with(&mut self, head: Entity, tail: Entity, attribs: Attributes) -> Entity {
        let entity = self.new_edge(head, tail);
        self.attributes_mut(entity).extend(attribs);
        entity
    }

    /// Like 'new_subgraph' but takes attributes to add to the default as an argument.
    fn new_subgraph_with(&mut self, attribs: Attributes) -> SubgraphBuilder {
        let mut result = self.new_subgraph();
        result.attributes_mut(result.entity).extend(attribs);
        result
    }

    /// Like 'new_cluster' but takes attributes to add to the default as an argument.
    fn new_cluster_with(
        &mut self,
        label: impl Into<String>,
        attribs: Attributes,
    ) -> SubgraphBuilder {
        let mut result = self.new_cluster(label);
        result.attributes_mut(result.entity).extend(attribs);
        result
    }

    /// Retrieve the defaults for the given kind of nodes, if set.
    ///
    /// When an entity is created, it is initialized with the defaults of its kind.
    ///
    /// Defaults are scoped: changes made to the defaults in a builder are not
    /// forwarded back to its parent, but builders for subgraphs get initialized
    /// with a copy of their parent's defaults.
    ///
    ///     use graphwiz::{Builder, Graph, Kind};
    ///
    ///     let mut root = Graph::new_builder();
    ///     root.defaults_mut(Kind::Node).insert("fillcolor", "green".to_string());
    ///     let a = root.new_node("a");
    ///     assert_eq!(root.attributes(a)["fillcolor"], "green".to_string());
    ///
    ///     let mut sub1 = root.new_cluster("c1");
    ///     sub1.defaults_mut(Kind::Node).insert("fillcolor", "blue".to_string());
    ///     let b = sub1.new_node("b");
    ///     assert_eq!(sub1.attributes(b)["fillcolor"], "blue".to_string());
    ///
    ///     let mut sub2 = sub1.new_cluster("c2");
    ///     let c = sub2.new_node("c");
    ///     assert_eq!(sub2.attributes(c)["fillcolor"], "blue".to_string());
    ///
    ///     sub2.build();
    ///     sub1.build();
    ///     let d = root.new_node("d");
    ///     assert_eq!(root.attributes(d)["fillcolor"], "green".to_string());
    fn defaults(&self, kind: Kind) -> Option<&Attributes>;

    /// Retrieve mutable defaults for the given kind of nodes.
    ///
    /// If defaults don't exist for the given kind, it is created, and a mutable
    /// reference to the newly created hashmap is returned.
    fn defaults_mut(&mut self, kind: Kind) -> &mut Attributes;

    /// Retrieve the attributes for the given entity.
    ///
    /// Attributes that are associated with a given node do not depend on the
    /// current scope, meaning that any builder can access the attributes of any
    /// entity.
    ///
    ///     use graphwiz::{Builder, Graph};
    ///     use std::collections::HashMap;
    ///
    ///     let mut root = Graph::new_builder();
    ///     let a = root.new_node_with("a", HashMap::from([
    ///         ("fillcolor", "blue".to_string()),
    ///     ]));
    ///
    ///     let mut subgraph = root.new_cluster("c");
    ///     assert_eq!(subgraph.attributes(a)["fillcolor"], "blue".to_string());
    fn attributes(&self, entity: Entity) -> &Attributes;

    /// Retrieve mutable attributes for the given kind of nodes.
    fn attributes_mut(&mut self, entity: Entity) -> &mut Attributes;
}

////////////////////////////////////////////////////////////////////////////////
// Internal

impl RootBuilder {
    pub(crate) fn new() -> RootBuilder {
        RootBuilder {
            graph: Graph {
                attributes: HashMap::from([(ROOT, HashMap::new())]),
                subgraphs: HashMap::new(),
                edges: HashMap::new(),
                latest: 0,
            },
            current: SubgraphInfo::new(),
            defaults: HashMap::new(),
        }
    }

    fn new_builder(&mut self, entity: Entity) -> SubgraphBuilder {
        SubgraphBuilder {
            graph: &mut self.graph,
            entity,
            current: SubgraphInfo::new(),
            defaults: self.defaults.clone(),
        }
    }
}

impl SubgraphBuilder<'_> {
    fn new_builder(&mut self, entity: Entity) -> SubgraphBuilder {
        SubgraphBuilder {
            graph: self.graph,
            entity,
            current: SubgraphInfo::new(),
            defaults: self.defaults.clone(),
        }
    }
}

impl Builder for RootBuilder {
    fn new_node(&mut self, label: impl Into<String>) -> Entity {
        let entity = self.graph.new_node(label, &self.defaults);
        self.current.nodes.push(entity);
        entity
    }

    fn new_edge(&mut self, head: Entity, tail: Entity) -> Entity {
        let entity = self.graph.new_edge(head, tail, &self.defaults);
        self.current.edges.push(entity);
        entity
    }

    fn new_subgraph(&mut self) -> SubgraphBuilder {
        let entity = self.graph.register(Kind::Subgraph, &self.defaults);
        self.current.subgraphs.push(entity);
        self.new_builder(entity)
    }

    fn new_cluster(&mut self, label: impl Into<String>) -> SubgraphBuilder {
        let entity = self.graph.register(Kind::Cluster, &self.defaults);
        self.current.subgraphs.push(entity);
        self.attributes_mut(entity).insert("label", label.into());
        self.new_builder(entity)
    }

    fn defaults(&self, kind: Kind) -> Option<&Attributes> {
        self.defaults.get(&kind)
    }

    fn defaults_mut(&mut self, kind: Kind) -> &mut Attributes {
        self.defaults.entry(kind).or_default()
    }

    fn attributes(&self, entity: Entity) -> &Attributes {
        self.graph.attributes(entity)
    }

    fn attributes_mut(&mut self, entity: Entity) -> &mut Attributes {
        self.graph.attributes_mut(entity)
    }
}

impl Builder for SubgraphBuilder<'_> {
    fn new_node(&mut self, label: impl Into<String>) -> Entity {
        let entity = self.graph.new_node(label, &self.defaults);
        self.current.nodes.push(entity);
        entity
    }

    fn new_edge(&mut self, head: Entity, tail: Entity) -> Entity {
        let entity = self.graph.new_edge(head, tail, &self.defaults);
        self.current.edges.push(entity);
        entity
    }

    fn new_subgraph(&mut self) -> SubgraphBuilder {
        let entity = self.graph.register(Kind::Subgraph, &self.defaults);
        self.current.subgraphs.push(entity);
        self.new_builder(entity)
    }

    fn new_cluster(&mut self, label: impl Into<String>) -> SubgraphBuilder {
        let entity = self.graph.register(Kind::Cluster, &self.defaults);
        self.current.subgraphs.push(entity);
        self.attributes_mut(entity).insert("label", label.into());
        self.new_builder(entity)
    }

    fn defaults(&self, kind: Kind) -> Option<&Attributes> {
        self.defaults.get(&kind)
    }

    fn defaults_mut(&mut self, kind: Kind) -> &mut Attributes {
        self.defaults.entry(kind).or_default()
    }

    fn attributes(&self, entity: Entity) -> &Attributes {
        self.graph.attributes(entity)
    }

    fn attributes_mut(&mut self, entity: Entity) -> &mut Attributes {
        self.graph.attributes_mut(entity)
    }
}
