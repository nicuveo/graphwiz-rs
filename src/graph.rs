use std::collections::HashMap;

use crate::builder::RootBuilder;

////////////////////////////////////////////////////////////////////////////////
// Public API

/// Simple enum representing the four kinds of entities.
///
/// Clusters and subgraphs are considered different, allowing the user to
/// specify different default attributes for each (see `Builder::defaults`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Node,
    Edge,
    Cluster,
    Subgraph,
}

/// Unique identifier for a graph entity.
///
/// This opaque and lightweight identifier can be copied freely and doesn't hold
/// a reference to the graph.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Entity {
    pub(crate) kind: Kind,
    pub(crate) id: Id,
}

/// Attributes of an entity.
pub type Attributes = HashMap<&'static str, String>;

/// Default attributes for a given [Kind].
pub type Defaults = HashMap<Kind, Attributes>;

/// Resulting graph.
///
/// A graph is not created directly: [Graph::new_builder] creates a
/// [RootBuilder], and [RootBuilder::build] consumes the builder and returns the
/// graph.
///
/// The graph can be transformed into a DOT representation using any of the
/// rendering functions.
#[derive(Debug)]
pub struct Graph {
    pub(crate) attributes: HashMap<Entity, Attributes>,
    pub(crate) subgraphs: HashMap<Entity, SubgraphInfo>,
    pub(crate) edges: HashMap<Entity, EdgeInfo>,
    pub(crate) latest: Id,
}

impl Graph {
    /// Creates a new [RootBuilder].
    pub fn new_builder() -> RootBuilder {
        RootBuilder::new()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Internal

type Id = u32;

pub(crate) const ROOT: Entity = Entity {
    kind: Kind::Subgraph,
    id: 0,
};

impl Graph {
    pub(crate) fn register(&mut self, kind: Kind, defaults: &Defaults) -> Entity {
        self.latest += 1;
        let entity = Entity {
            kind,
            id: self.latest,
        };
        self.attributes
            .insert(entity, defaults.get(&kind).cloned().unwrap_or_default());
        entity
    }

    fn locate(&self, entity: &Entity) -> Option<Entity> {
        let info = self.subgraphs.get(entity)?;
        info.nodes
            .first()
            .cloned()
            .or_else(|| info.subgraphs.iter().find_map(|e| self.locate(e)))
    }

    fn resolve<F>(&mut self, entity: Entity, func: F) -> (Entity, Option<Entity>)
    where
        F: FnOnce(&EdgeInfo) -> (Entity, Option<Entity>),
    {
        match entity.kind {
            Kind::Node => (entity, None),
            Kind::Edge => func(&self.edges[&entity]),
            Kind::Cluster | Kind::Subgraph => {
                self.attributes
                    .get_mut(&ROOT)
                    .unwrap()
                    .insert("compound", "true".to_string());
                (self.locate(&entity).unwrap_or(entity), Some(entity))
            }
        }
    }

    pub(crate) fn new_node<S: Into<String>>(&mut self, label: S, defaults: &Defaults) -> Entity {
        let entity = self.register(Kind::Node, defaults);
        self.attributes_mut(entity).insert("label", label.into());
        entity
    }

    pub(crate) fn new_edge(&mut self, head: Entity, tail: Entity, defaults: &Defaults) -> Entity {
        let (head_node, head_subgraph) = self.resolve(head, |i| (i.tail_node, i.tail_subgraph));
        let (tail_node, tail_subgraph) = self.resolve(tail, |i| (i.head_node, i.head_subgraph));
        let info = EdgeInfo {
            head_node,
            tail_node,
            head_subgraph,
            tail_subgraph,
        };
        let entity = self.register(Kind::Edge, defaults);
        self.edges.insert(entity, info);
        entity
    }

    pub(crate) fn attributes(&self, entity: Entity) -> &Attributes {
        self.attributes.get(&entity).unwrap()
    }

    pub(crate) fn attributes_mut(&mut self, entity: Entity) -> &mut Attributes {
        self.attributes.get_mut(&entity).unwrap()
    }
}

#[derive(Debug)]
pub(crate) struct EdgeInfo {
    pub(crate) head_node: Entity,
    pub(crate) tail_node: Entity,
    pub(crate) head_subgraph: Option<Entity>,
    pub(crate) tail_subgraph: Option<Entity>,
}

#[derive(Debug, Default)]
pub(crate) struct SubgraphInfo {
    pub(crate) nodes: Vec<Entity>,
    pub(crate) edges: Vec<Entity>,
    pub(crate) subgraphs: Vec<Entity>,
}
