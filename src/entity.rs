use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Public API

/// Simple Enum representing the four kinds of entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Node,
    Edge,
    Cluster,
    Subgraph,
}

/// Unique identifier for a graph entity
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Entity {
    pub(crate) kind: Kind,
    pub(crate) id: Id,
}

/// Attributes of an entity
pub type Attributes = HashMap<&'static str, String>;

pub type Defaults = HashMap<Kind, Attributes>;

////////////////////////////////////////////////////////////////////////////////
// Internal

pub(crate) type Id = u32;

pub(crate) const ROOT: Entity = Entity {
    kind: Kind::Subgraph,
    id: 0,
};
