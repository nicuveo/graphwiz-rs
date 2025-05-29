use goldie;
use graphwiz::attributes as attrs;
use graphwiz::*;
use std::collections::HashMap;

#[test]
fn example() {
    let mut root = Graph::new_builder();
    root.defaults_mut(Kind::Node).extend(HashMap::from([
        (attrs::FILLCOLOR, "lavender".to_string()),
        (attrs::STYLE, "filled".to_string()),
    ]));

    let a = root.new_node("a");
    let b = root.new_node("b");
    let ab = root.new_edge(a, b);
    root.attributes_mut(ab)
        .insert(attrs::STYLE, "dotted".to_string());

    let mut cluster = root.new_cluster("box");
    let c = cluster.new_node_with(
        "c",
        HashMap::from([
            (attrs::SHAPE, "circle".to_string()),
            (attrs::FILLCOLOR, "cornflowerblue".to_string()),
        ]),
    );
    cluster.build();

    root.new_edge(c, a);
    root.new_edge(c, b);
    let graph = root.build();
    goldie::assert!(render_digraph(&graph));
}
