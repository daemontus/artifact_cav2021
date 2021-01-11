use std::io::Read;
use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, GraphColoredVertices};
use biodivine_lib_param_bn::BooleanNetwork;
use cav2021_artifact::algorithms::find_attractors;
use std::convert::TryFrom;

fn main() {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();

    let model = BooleanNetwork::try_from(buffer.as_str()).unwrap();
    let graph = SymbolicAsyncGraph::new(model).unwrap();
    let variables = graph.network().variables().collect::<Vec<_>>();

    let attractors = find_attractors(&graph, &variables, graph.mk_unit_vertices());

    for (i, attr) in attractors.into_iter().enumerate() {
        println!("Attractor #{}: {}", i+1, attr.approx_cardinality());
    }
}