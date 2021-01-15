use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;
use cav2021_artifact::algorithms::{find_attractors, round_robin_reduction};
use cav2021_artifact::log_message;
use std::convert::TryFrom;
use std::io::Read;

fn main() {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();

    let model = BooleanNetwork::try_from(buffer.as_str()).unwrap();
    let graph = SymbolicAsyncGraph::new(model).unwrap();

    let (universe, variables) = round_robin_reduction(&graph, graph.unit_vertices());
    let attractors = find_attractors(&graph, &variables, universe);

    for (i, attr) in attractors.into_iter().enumerate() {
        log_message(&format!(
            "Attractor #{}: {} (using {} nodes)",
            i + 1,
            attr.approx_cardinality(),
            attr.as_bdd().size()
        ));
    }
}
