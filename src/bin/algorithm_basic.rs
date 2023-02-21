use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;
use cav2021_artifact::algorithms::find_attractors_lockstep;
use cav2021_artifact::log_message;
use std::convert::TryFrom;
use std::io::Read;

fn main() {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();

    let model = BooleanNetwork::try_from(buffer.as_str()).unwrap();
    let graph = SymbolicAsyncGraph::new(model).unwrap();
    let variables = graph.as_network().variables().collect::<Vec<_>>();

    let attractors = find_attractors_lockstep(&graph, &variables, graph.mk_unit_colored_vertices());

    for (i, attr) in attractors.into_iter().enumerate() {
        log_message(&format!(
            "Attractor #{}: {}",
            i + 1,
            attr.approx_cardinality()
        ));
    }
}
