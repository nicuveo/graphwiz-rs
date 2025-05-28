use goldie;
use graphwiz::*;
use std::collections::HashMap;
use trees::{Node, tr};

#[test]
fn forest_simple() {
    let forest = vec![
        tr(0) / (-(tr(1) / (-tr(2) - tr(3))) - (tr(4) / (-tr(5) - tr(6)))),
        tr(10) / tr(20) / tr(30),
        tr(10) / (tr(20) / tr(30)),
    ];

    let mut builder = Graph::new_builder();
    for node in forest.iter() {
        visit_node(&mut builder.new_subgraph(), node);
    }
    let text = render_digraph(&builder.build());
    goldie::assert!(text)
}

fn visit_node<B: Builder>(builder: &mut B, tree_node: &Node<u32>) -> Entity {
    let entity = builder.new_node(tree_node.data().to_string());
    for child in tree_node.iter() {
        let child_node = visit_node(builder, child);
        builder.new_edge(entity, child_node);
    }
    entity
}

#[test]
fn original_hs() {
    let mut builder = Graph::new_builder();
    builder
        .defaults_mut(Kind::Node)
        .insert("style", "filled".to_string());

    let mut front = builder.new_cluster("front end");
    let code = front.new_node_with(
        "source code",
        HashMap::from([("fillcolor", "#c3ffd8".to_string())]),
    );
    let ast = front.new_node_with("AST", HashMap::from([("fillcolor", "yellow".to_string())]));
    front.new_edge_with(code, ast, HashMap::from([("label", "parsing".to_string())]));
    front.build();

    let mut middle = builder.new_cluster("middle end");
    let ir = middle.new_node_with(
        "IR",
        HashMap::from([
            ("fillcolor", "salmon".to_string()),
            ("shape", "diamond".to_string()),
        ]),
    );
    middle.new_edge_with(
        ast,
        ir,
        HashMap::from([
            ("label", "lowering".to_string()),
            ("style", "dotted".to_string()),
        ]),
    );
    middle.build();

    let text = render_digraph(&builder.build());
    goldie::assert!(text)
}
