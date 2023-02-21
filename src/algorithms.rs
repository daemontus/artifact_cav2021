use crate::process::{
    FwdProcess, PriorityScheduler, Process, ReachAfterPostProcess, RoundRobinScheduler, Scheduler,
};
use crate::{log_message, log_progress};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

/// Performs transition guided reduction and the processes are executed with priority based
/// interleaving (priority being the size of the symbolic representation).
pub fn priority_reduction(
    graph: &SymbolicAsyncGraph,
    universe: &GraphColoredVertices,
) -> (GraphColoredVertices, Vec<VariableId>) {
    let mut scheduler = PriorityScheduler::new(graph, &universe);
    for variable in graph.as_network().variables() {
        scheduler.spawn(ReachAfterPostProcess::new(
            variable,
            graph,
            scheduler.universe(),
        ));
    }

    let mut iter: usize = 0;
    loop {
        let i = scheduler.step(graph);
        if i == 0 {
            break;
        }
        iter += i;
        log_progress(|| format!("Iteration: {}", iter));
    }
    log_message(&format!("Total iterations: {}", iter));

    scheduler.finalize()
}

/// Performs transition guided reduction, but the processes are executed with
/// round robin interleaving.
pub fn round_robin_reduction(
    graph: &SymbolicAsyncGraph,
    universe: &GraphColoredVertices,
) -> (GraphColoredVertices, Vec<VariableId>) {
    let mut scheduler = RoundRobinScheduler::new(graph, &universe);
    for variable in graph.as_network().variables() {
        scheduler.spawn(ReachAfterPostProcess::new(
            variable,
            graph,
            scheduler.universe(),
        ));
    }

    let mut iter: usize = 0;
    loop {
        let i = scheduler.step(graph);
        if i == 0 {
            break;
        }
        iter += i;
        log_progress(|| format!("Iteration: {}", iter));
    }
    log_message(&format!("Total iterations: {}", iter));

    scheduler.finalize()
}

/// Performs transition guided reduction, but each process is executed sequentially.
pub fn sequential_reduction(
    graph: &SymbolicAsyncGraph,
    mut universe: GraphColoredVertices,
) -> (GraphColoredVertices, Vec<VariableId>) {
    let mut active_variables: Vec<VariableId> = graph.as_network().variables().collect();
    for var in graph.as_network().variables() {
        log_message(&format!(
            "Reducing {:?}. Remaining: {}",
            var,
            universe.approx_cardinality()
        ));
        let var_can_post = graph.var_can_post(var, &universe);
        let reach_from_post = reach_fwd(graph, &active_variables, &var_can_post, &universe);

        // Remove basin of the reachable area.
        if reach_from_post != universe {
            let reach_basin = reach_bwd(graph, &active_variables, &reach_from_post, &universe)
                .minus(&reach_from_post);
            if !reach_basin.is_empty() {
                log_message(&format!(
                    "Eliminated reach basin {}.",
                    reach_basin.approx_cardinality()
                ));
                universe = universe.minus(&reach_basin);
            }
        }

        let post_extended_component =
            reach_bwd(graph, &active_variables, &var_can_post, &reach_from_post);
        let bottom_region = reach_from_post.minus(&post_extended_component);

        // Remove basin of the bottom area.
        if !bottom_region.is_empty() {
            let bottom_basin = reach_bwd(graph, &active_variables, &bottom_region, &universe)
                .minus(&bottom_region);
            if !bottom_basin.is_empty() {
                log_message(&format!(
                    "Eliminated bottom basin {}.",
                    bottom_basin.approx_cardinality()
                ));
                universe = universe.minus(&bottom_basin);
            }
        }

        if graph.var_can_post(var, &universe).is_empty() {
            active_variables = active_variables.into_iter().filter(|v| *v != var).collect();
            log_message(&format!(
                "Variable eliminated. {} remaining.",
                active_variables.len()
            ));
        }
    }
    (universe, active_variables)
}

/// A "scheduler" that does not do anything so that we can use asynchronous processes
/// outside of interleaving algorithm.
struct FakeScheduler {
    variables: Vec<VariableId>,
}

impl Scheduler for FakeScheduler {
    fn step(&mut self, _graph: &SymbolicAsyncGraph) -> usize {
        unimplemented!()
    }

    fn finalize(self) -> (GraphColoredVertices, Vec<VariableId>) {
        unimplemented!()
    }

    fn discard_variable(&mut self, _variable: VariableId) {
        unimplemented!()
    }

    fn discard_states(&mut self, _set: &GraphColoredVertices) {
        unimplemented!()
    }

    fn spawn_process(&mut self, _process: Box<dyn Process<Self>>) {
        unimplemented!()
    }

    fn universe(&self) -> &GraphColoredVertices {
        unimplemented!()
    }

    fn variables(&self) -> &[VariableId] {
        &self.variables
    }
}

/// Perform exhaustive search for graph BSCCs, but only consider provided `universe` set
/// and transitions in the given `variables`.
pub fn find_attractors_lockstep(
    graph: &SymbolicAsyncGraph,
    variables: &[VariableId],
    mut universe: GraphColoredVertices,
) -> Vec<GraphColoredVertices> {
    let mut scheduler = FakeScheduler {
        variables: variables.to_vec(),
    };
    let mut result = Vec::new();
    while !universe.is_empty() {
        log_message(&format!(
            "Start universe {}({})",
            universe.approx_cardinality(),
            universe.as_bdd().size()
        ));
        let pivot = universe.pick_vertex();
        let bwd_set = reach_bwd(graph, variables, &pivot, &universe);
        let mut fwd = FwdProcess::new(&pivot, graph.unit_colored_vertices());
        let mut is_terminal = true;
        loop {
            if !fwd.reach_set().is_subset(&bwd_set) {
                is_terminal = false;
                break;
            }
            if fwd.step(&mut scheduler, graph) {
                break;
            }
            log_progress(|| format!("Fwd size {}", fwd.reach_set().as_bdd().size()));
        }
        if is_terminal {
            log_message(&format!(
                "Found attractor. State count {}",
                fwd.reach_set().approx_cardinality()
            ));
            result.push(fwd.reach_set().clone());
        }
        universe = universe.minus(&bwd_set);
    }
    return result;
}

/// Performs a saturating forwards reachability search.
pub fn reach_fwd(
    graph: &SymbolicAsyncGraph,
    variables: &[VariableId],
    initial: &GraphColoredVertices,
    universe: &GraphColoredVertices,
) -> GraphColoredVertices {
    if variables.is_empty() {
        return initial.clone();
    }
    let mut result = initial.clone();
    let last_variable = variables.len() - 1;
    let mut active_variable = last_variable;
    loop {
        let variable = variables[active_variable];
        let post = graph
            .var_post(variable, &result)
            .intersect(universe)
            .minus(&result);

        if !post.is_empty() {
            result = result.union(&post);
            active_variable = last_variable;
        } else {
            if active_variable == 0 {
                break;
            } else {
                active_variable -= 1;
            }
        }
    }
    return result;
}

/// Performs a saturating backwards reachability search.
pub fn reach_bwd(
    graph: &SymbolicAsyncGraph,
    variables: &[VariableId],
    initial: &GraphColoredVertices,
    universe: &GraphColoredVertices,
) -> GraphColoredVertices {
    if variables.is_empty() {
        return initial.clone();
    }
    let mut result = initial.clone();
    let last_variable = variables.len() - 1;
    let mut active_variable = last_variable;
    loop {
        let variable = variables[active_variable];
        let pre = graph
            .var_pre(variable, &result)
            .intersect(universe)
            .minus(&result);

        if !pre.is_empty() {
            result = result.union(&pre);
            active_variable = last_variable;
        } else {
            if active_variable == 0 {
                break;
            } else {
                active_variable -= 1;
            }
        }
    }
    return result;
}
