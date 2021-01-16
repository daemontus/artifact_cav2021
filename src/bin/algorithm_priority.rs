use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;
use cav2021_artifact::algorithms::{priority_reduction, find_attractors_lockstep};
use cav2021_artifact::log_message;
use std::convert::TryFrom;
use std::io::Read;

fn main() {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();

    let model = BooleanNetwork::try_from(buffer.as_str()).unwrap();
    let graph = SymbolicAsyncGraph::new(model).unwrap();

    let (universe, variables) = priority_reduction(&graph, graph.unit_vertices());
    let attractors = find_attractors_lockstep(&graph, &variables, universe);

    for (i, attr) in attractors.into_iter().enumerate() {
        log_message(&format!(
            "Attractor #{}: {} (using {} nodes)",
            i + 1,
            attr.approx_cardinality(),
            attr.as_bdd().size()
        ));
        let mut c: usize = 0;
        let mut states = attr.as_bdd().sat_valuations();
        while c < 10_000 {
            if states.next().is_some() {
                c += 1;
            } else {
                break;
            }
        }
        log_message(&format!("Counted state space: {}{}", c, if c == 10_000 { "+" } else { "" }));
    }
}
