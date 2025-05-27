mod builder;
mod entity;
mod graph;
mod render;

pub use builder::*;
pub use entity::*;
pub use graph::*;
pub use render::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_a_graph() {
        let mut root_builder = Graph::new();
        let a = root_builder.new_node("a");

        let mut left_builder = root_builder.new_cluster("left");
        let b = left_builder.new_node("b");
        left_builder.new_edge(a, b);
        let left = left_builder.build();

        let mut right_builder = root_builder.new_cluster("right");
        let c = right_builder.new_node("c");
        right_builder.new_edge(a, c);
        let right = right_builder.build();

        root_builder.new_edge(left, right);
        let graph = root_builder.build();

        println!("{}", render_digraph(&graph));
    }
}
