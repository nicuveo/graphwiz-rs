use std::collections::HashMap;

use crate::entity::*;
use crate::graph::*;

/// Builder for the root graph
#[derive(Debug)]
pub struct RootBuilder {
    pub(crate) graph: Graph,
    pub(crate) current: SubgraphInfo,
    pub(crate) defaults: HashMap<Kind, Attributes>,
}

impl RootBuilder {
    fn new_builder(&mut self, entity: Entity) -> SubgraphBuilder {
        SubgraphBuilder {
            graph: &mut self.graph,
            entity: entity,
            current: SubgraphInfo::new(),
            defaults: self.defaults.clone(),
        }
    }

    /// Finalizes the builder and returns the final graph
    pub fn build(mut self) -> Graph {
        self.graph.subgraphs.insert(ROOT, self.current);
        self.graph
    }
}

/// Builer for all subgraphs
#[derive(Debug)]
pub struct SubgraphBuilder<'a> {
    graph: &'a mut Graph,
    entity: Entity,
    current: SubgraphInfo,
    defaults: HashMap<Kind, Attributes>,
}

impl SubgraphBuilder<'_> {
    fn new_builder(&mut self, entity: Entity) -> SubgraphBuilder {
        SubgraphBuilder {
            graph: self.graph,
            entity: entity,
            current: SubgraphInfo::new(),
            defaults: self.defaults.clone(),
        }
    }

    /// Finalizes the builder and returns the corresponding entity
    pub fn build(self) -> Entity {
        self.graph.subgraphs.insert(self.entity, self.current);
        self.entity
    }
}

/// All required functions to build new graph eleements
pub trait Builder {
    /// Creates a new node within the current scope, with the given label.
    /// Returns the new node's entity, which can be used to alter this node's
    /// attributes.
    fn new_node<S: Into<String>>(&mut self, label: S) -> Entity;

    /// Creates a new edge between the two provided entities.
    ///
    /// If any of the entities is a subgraph or a cluster, then this sets the
    /// "compound" property of the graph to true, and it finds a random node
    /// within the subgraph or cluster to be able to generate the proper edge.
    ///
    /// If any of the entities is an edge, this function "chains" them:
    ///
    ///     use graphwiz::{Graph, Builder};
    ///     let mut builder = Graph::new();
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
    /// comsumed with 'build'.
    fn new_subgraph(&mut self) -> SubgraphBuilder;

    /// Creates a new cluster within the current scope with the given label.
    ///
    /// This function borrows the underlying shared state, meaning that this
    /// builder can no longer be used until the new subgraph builder has been
    /// comsumed with 'build'.
    fn new_cluster<S: Into<String>>(&mut self, label: S) -> SubgraphBuilder;

    /// Like 'new_node' but takes attributes to add to the default as an argument.
    fn new_node_with<S: Into<String>>(&mut self, label: S, attribs: Attributes) -> Entity {
        let entity = self.new_node(label);
        self.attributes_mut(entity).extend(attribs.into_iter());
        entity
    }

    /// Like 'new_edge' but takes attributes to add to the default as an argument.
    fn new_edge_with(&mut self, head: Entity, tail: Entity, attribs: Attributes) -> Entity {
        let entity = self.new_edge(head, tail);
        self.attributes_mut(entity).extend(attribs.into_iter());
        entity
    }

    /// Like 'new_subgraph' but takes attributes to add to the default as an argument.
    fn new_subgraph_with(&mut self, attribs: Attributes) -> SubgraphBuilder {
        let mut result = self.new_subgraph();
        result
            .attributes_mut(result.entity)
            .extend(attribs.into_iter());
        result
    }

    /// Like 'new_cluster' but takes attributes to add to the default as an argument.
    fn new_cluster_with<S: Into<String>>(
        &mut self,
        label: S,
        attribs: Attributes,
    ) -> SubgraphBuilder {
        let mut result = self.new_cluster(label);
        result
            .attributes_mut(result.entity)
            .extend(attribs.into_iter());
        result
    }

    fn defaults(&self, kind: Kind) -> &Attributes;

    fn defaults_mut(&mut self, kind: Kind) -> &mut Attributes;

    fn attributes(&self, entity: Entity) -> &Attributes;

    fn attributes_mut(&mut self, entity: Entity) -> &mut Attributes;
}

impl Builder for RootBuilder {
    fn new_node<S: Into<String>>(&mut self, label: S) -> Entity {
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

    fn new_cluster<S: Into<String>>(&mut self, label: S) -> SubgraphBuilder {
        let entity = self.graph.register(Kind::Cluster, &self.defaults);
        self.current.subgraphs.push(entity);
        self.attributes_mut(entity).insert("label", label.into());
        self.new_builder(entity)
    }

    fn defaults(&self, kind: Kind) -> &Attributes {
        self.defaults.get(&kind).unwrap()
    }

    fn defaults_mut(&mut self, kind: Kind) -> &mut Attributes {
        self.defaults.get_mut(&kind).unwrap()
    }

    fn attributes(&self, entity: Entity) -> &Attributes {
        self.graph.attributes(entity)
    }

    fn attributes_mut(&mut self, entity: Entity) -> &mut Attributes {
        self.graph.attributes_mut(entity)
    }
}

impl Builder for SubgraphBuilder<'_> {
    fn new_node<S: Into<String>>(&mut self, label: S) -> Entity {
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

    fn new_cluster<S: Into<String>>(&mut self, label: S) -> SubgraphBuilder {
        let entity = self.graph.register(Kind::Cluster, &self.defaults);
        self.current.subgraphs.push(entity);
        self.attributes_mut(entity).insert("label", label.into());
        self.new_builder(entity)
    }

    fn defaults(&self, kind: Kind) -> &Attributes {
        self.defaults.get(&kind).unwrap()
    }

    fn defaults_mut(&mut self, kind: Kind) -> &mut Attributes {
        self.defaults.get_mut(&kind).unwrap()
    }

    fn attributes(&self, entity: Entity) -> &Attributes {
        self.graph.attributes(entity)
    }

    fn attributes_mut(&mut self, entity: Entity) -> &mut Attributes {
        self.graph.attributes_mut(entity)
    }
}
