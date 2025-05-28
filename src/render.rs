use crate::entity::*;
use crate::graph::*;

////////////////////////////////////////////////////////////////////////////////
// Public API

pub fn render_graph(graph: &Graph) -> String {
    render_root(graph, "graph", "--")
}

pub fn render_digraph(graph: &Graph) -> String {
    render_root(graph, "graph", "->")
}

pub fn render_strict_graph(graph: &Graph) -> String {
    render_root(graph, "graph", "--")
}

pub fn render_strict_digraph(graph: &Graph) -> String {
    render_root(graph, "graph", "->")
}

////////////////////////////////////////////////////////////////////////////////
// Internal

fn indent(mut lines: Vec<String>) -> Vec<String> {
    for line in lines.iter_mut() {
        *line = format!("    {}", *line);
    }
    lines
}

fn render_root(graph: &Graph, kind: &str, arrow: &str) -> String {
    let header = format!("{} {{", kind);
    let subgraph = &graph.subgraphs[&ROOT];
    let attributes = &graph.attributes.get(&ROOT).unwrap();
    render_group(graph, arrow, header, subgraph, attributes).join("\n")
}

fn render_subgraph(graph: &Graph, arrow: &str, entity: Entity) -> Vec<String> {
    let header = format!("subgraph {} {{", render_entity(graph, entity));
    let subgraph = &graph.subgraphs.get(&entity).unwrap();
    let attributes = &graph.attributes.get(&entity).unwrap();
    render_group(graph, arrow, header, subgraph, attributes)
}

fn render_group(
    graph: &Graph,
    arrow: &str,
    header: String,
    subgraph: &SubgraphInfo,
    attributes: &Attributes,
) -> Vec<String> {
    let attributes = render_attributes(attributes);
    let nodes = subgraph
        .nodes
        .iter()
        .map(|entity| render_node(graph, *entity))
        .collect();
    let edges = subgraph
        .edges
        .iter()
        .map(|entity| render_edge(graph, arrow, *entity))
        .collect();
    let subgraphs = subgraph
        .subgraphs
        .iter()
        .flat_map(|entity| render_subgraph(graph, arrow, *entity))
        .collect();
    [
        vec![header],
        indent(attributes),
        indent(nodes),
        indent(edges),
        indent(subgraphs),
        vec!["}".to_string()],
    ]
    .concat()
}

fn render_node(graph: &Graph, entity: Entity) -> String {
    format!(
        "{} [{}]",
        render_entity(graph, entity),
        render_attributes(graph.attributes.get(&entity).unwrap()).join(", ")
    )
}

fn render_edge(graph: &Graph, arrow: &str, entity: Entity) -> String {
    let edge = graph.edges.get(&entity).unwrap();
    let head = render_entity(graph, edge.head_node);
    let tail = render_entity(graph, edge.tail_node);
    let mut attributes = render_attributes(graph.attributes.get(&entity).unwrap());
    let mut attributes = match edge.head_subgraph {
        None => attributes,
        Some(entity) => {
            attributes.push(render_attribute("lhead", &render_entity(graph, entity)));
            attributes
        }
    };
    let attributes = match edge.tail_subgraph {
        None => attributes,
        Some(entity) => {
            attributes.push(render_attribute("ltail", &render_entity(graph, entity)));
            attributes
        }
    };
    format!("{} {} {} [{}]", head, arrow, tail, attributes.join(", "))
}

fn render_entity(graph: &Graph, entity: Entity) -> String {
    let max = ((graph.latest as f32).log10() + 1.0f32) as usize;
    match entity.kind {
        Kind::Node => format!("node_{:0>1$}", entity.id, max),
        Kind::Edge => format!("edge_{:0>1$}", entity.id, max),
        Kind::Cluster => format!("cluster_{:0>1$}", entity.id, max),
        Kind::Subgraph => format!("subgraph_{:0>1$}", entity.id, max),
    }
}

fn render_attributes(attributes: &Attributes) -> Vec<String> {
    attributes
        .iter()
        .map(|(k, v)| render_attribute(k, v))
        .collect()
}

fn render_attribute(key: &str, value: &String) -> String {
    format!("{}=\"{}\"", key, value)
}
