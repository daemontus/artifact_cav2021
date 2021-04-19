use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

mod _impl_bwd_process;
mod _impl_extended_component_process;
mod _impl_fwd_process;
mod _impl_priority_scheduler;
mod _impl_reach_after_post_process;
mod _impl_round_robin_scheduler;

/// Process is one unit of work that can be executed via a `Scheduler`.
pub trait Process<S: Scheduler> {
    /// Perform one step of this process.
    fn step(&mut self, scheduler: &mut S, graph: &SymbolicAsyncGraph) -> bool;
    /// A "weight" of a process is the symbolic size of its memory.
    fn weight(&self) -> usize;
    /// Mark given set of states as no longer necessary for consideration.
    fn discard_states(&mut self, set: &GraphColoredVertices);
}

/// Manages a collection of `Processes`.
pub trait Scheduler: Sized {
    /// Perform a step in some process.
    ///
    /// Returns the number of symbolic iterations performed by this step.
    /// If zero, it means there are no remaining processes.
    fn step(&mut self, graph: &SymbolicAsyncGraph) -> usize;
    /// Destroy processes and return computed data.
    fn finalize(self) -> (GraphColoredVertices, Vec<VariableId>);
    /// No longer use transitions in the given variable.
    fn discard_variable(&mut self, variable: VariableId);
    /// Remove given states from consideration.
    fn discard_states(&mut self, set: &GraphColoredVertices);
    /// Add a new process.
    fn spawn_process(&mut self, process: Box<dyn Process<Self>>);
    /// Get current "universe" of processes.
    fn universe(&self) -> &GraphColoredVertices;
    /// Get currently considered transition variables.
    fn variables(&self) -> &[VariableId];

    fn spawn<P: 'static + Process<Self>>(&mut self, process: P) {
        self.spawn_process(Box::new(process));
    }
}

/// A scheduler that uses process "weight" to assign each process a dynamic priority.
pub struct PriorityScheduler {
    active_variables: Vec<VariableId>,
    universe: GraphColoredVertices,
    processes: Vec<(usize, Box<dyn Process<PriorityScheduler>>)>,
    discarded: Option<GraphColoredVertices>,
}

/// A scheduler that executed processes fairly.
pub struct RoundRobinScheduler {
    active_variables: Vec<VariableId>,
    universe: GraphColoredVertices,
    processes: Vec<Box<dyn Process<RoundRobinScheduler>>>,
    discarded: Option<GraphColoredVertices>,
}

/// Process that computes the set of vertices reachable *after* a certain
/// transition is fired. Once it is completed, it spawns the `ExtendedComponentProcess`.
pub struct ReachAfterPostProcess {
    variable: VariableId,
    fwd: FwdProcess,
}

/// Computes the extended component of a given forward set and then eliminates the basin
/// of the graph region *under* this extended component.
pub struct ExtendedComponentProcess {
    variable: VariableId,
    fwd_set: GraphColoredVertices,
    bwd: BwdProcess,
}

/// Basic forward reachability process.
pub struct FwdProcess {
    fwd: GraphColoredVertices,
    universe: GraphColoredVertices,
}

/// Basic backward reachability process.
pub struct BwdProcess {
    bwd: GraphColoredVertices,
    universe: GraphColoredVertices,
}
