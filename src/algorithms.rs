use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, GraphColoredVertices};
use biodivine_lib_param_bn::VariableId;
use biodivine_lib_std::param_graph::Params;

pub fn find_attractors(
    graph: &SymbolicAsyncGraph,
    variables: &[VariableId],
    mut universe: GraphColoredVertices
) -> Vec<GraphColoredVertices> {
    let mut result = Vec::new();
    println!("Started attractor search in universe of size {}.", universe.approx_cardinality());
    while !universe.is_empty() {
        let pivot = universe.pick_vertex();
        println!("Picked pivot;");
        let pivot_basin = reach_bwd(graph, variables, &pivot, &universe);
        println!("Pivot basin: {};", pivot_basin.approx_cardinality());
        let pivot_component = reach_fwd(graph, variables, &pivot, &pivot_basin);
        println!("Pivot component: {};", pivot_component.approx_cardinality());
        let component_post = graph.post(&pivot_component).minus(&pivot_component);
        let is_terminal = pivot_component.colors().minus(&component_post.colors());
        if !is_terminal.is_empty() {
            let attr = pivot_component.intersect_colors(&is_terminal);
            println!("Found attractor. State count {}", attr.vertices().approx_cardinality());
            result.push(attr);
        }
        universe = universe.minus(&pivot_basin);
        println!("Remaining universe: {};", universe.approx_cardinality());
    }
    return result;
}

/// Performs a saturating forwards reachability search.
fn reach_fwd(
    graph: &SymbolicAsyncGraph,
    variables: &[VariableId],
    initial: &GraphColoredVertices,
    universe: &GraphColoredVertices
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
fn reach_bwd(
    graph: &SymbolicAsyncGraph,
    variables: &[VariableId],
    initial: &GraphColoredVertices,
    universe: &GraphColoredVertices
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