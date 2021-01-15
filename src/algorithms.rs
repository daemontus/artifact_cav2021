use crate::process::{PriorityScheduler, ReachAfterPostProcess, RoundRobinScheduler, Scheduler};
use crate::{log_message, log_progress};
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;
use biodivine_lib_std::param_graph::Params;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub fn priority_reduction(
    graph: &SymbolicAsyncGraph,
    universe: &GraphColoredVertices,
) -> (GraphColoredVertices, Vec<VariableId>) {
    let mut scheduler = PriorityScheduler::new(graph, &universe);
    for variable in graph.network().variables() {
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

pub fn round_robin_reduction(
    graph: &SymbolicAsyncGraph,
    universe: &GraphColoredVertices,
) -> (GraphColoredVertices, Vec<VariableId>) {
    let mut scheduler = RoundRobinScheduler::new(graph, &universe);
    for variable in graph.network().variables() {
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

pub fn sequential_reduction(
    graph: &SymbolicAsyncGraph,
    mut universe: GraphColoredVertices,
) -> (GraphColoredVertices, Vec<VariableId>) {
    let mut active_variables: Vec<VariableId> = graph.network().variables().collect();
    for var in graph.network().variables() {
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

pub fn find_attractors(
    graph: &SymbolicAsyncGraph,
    variables: &[VariableId],
    mut universe: GraphColoredVertices,
) -> Vec<GraphColoredVertices> {
    let mut random = StdRng::seed_from_u64(1234567890);
    let mut result = Vec::new();
    log_message(&format!(
        "Started attractor search in universe of size {}.",
        universe.approx_cardinality()
    ));
    while !universe.is_empty() {
        //let pivot = universe.pick_vertex();
        let pivot = random_pivot(graph, &universe, &mut random);
        let pivot_basin = reach_bwd(graph, variables, &pivot, &universe);
        let pivot_component = reach_fwd(graph, variables, &pivot, &pivot_basin);
        let component_post = graph.post(&pivot_component).minus(&pivot_component);
        let is_terminal = pivot_component.colors().minus(&component_post.colors());
        if !is_terminal.is_empty() {
            let attr = pivot_component.intersect_colors(&is_terminal);
            log_message(&format!(
                "Found attractor. State count {}",
                attr.vertices().approx_cardinality()
            ));
            result.push(attr);
        }
        universe = universe.minus(&pivot_basin);
        log_progress(|| format!("Remaining universe: {};", universe.approx_cardinality()));
    }
    return result;
}

pub fn random_pivot(
    graph: &SymbolicAsyncGraph,
    set: &GraphColoredVertices,
    random: &mut StdRng,
) -> GraphColoredVertices {
    let mut pivot = set.clone();
    for v in graph.network().variables() {
        let value = random.gen_bool(0.5);
        let v_set = graph.fix_network_variable(v, value);
        let applied = pivot.intersect(&v_set);
        if !applied.is_empty() {
            pivot = applied;
        } else {
            pivot = pivot.intersect(&graph.fix_network_variable(v, !value));
        }
    }
    if pivot.approx_cardinality() != 1.0 {
        eprintln!("WTF. Pivot selection fail.");
    }
    pivot
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
