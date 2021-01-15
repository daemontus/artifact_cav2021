use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

mod _impl_bwd_process;
mod _impl_extended_component_process;
mod _impl_fwd_process;
mod _impl_priority_scheduler;
mod _impl_reach_after_post_process;
mod _impl_round_robin_scheduler;

pub trait Process<S: Scheduler> {
    fn step(&mut self, scheduler: &mut S, graph: &SymbolicAsyncGraph) -> bool;
    fn weight(&self) -> usize;
    fn discard_states(&mut self, set: &GraphColoredVertices);
}

pub trait Scheduler: Sized {
    /// Returns the number of symbolic iterations performed by this step.
    /// If zero, it means there are no remaining processes.
    fn step(&mut self, graph: &SymbolicAsyncGraph) -> usize;
    fn finalize(self) -> (GraphColoredVertices, Vec<VariableId>);
    fn discard_variable(&mut self, variable: VariableId);
    fn discard_states(&mut self, set: &GraphColoredVertices);
    fn spawn_process(&mut self, process: Box<dyn Process<Self>>);
    fn universe(&self) -> &GraphColoredVertices;
    fn variables(&self) -> &[VariableId];

    fn spawn<P: 'static + Process<Self>>(&mut self, process: P) {
        self.spawn_process(Box::new(process));
    }
}

pub struct PriorityScheduler {
    active_variables: Vec<VariableId>,
    universe: GraphColoredVertices,
    processes: Vec<(usize, Box<dyn Process<PriorityScheduler>>)>,
    discarded: Option<GraphColoredVertices>,
}

pub struct RoundRobinScheduler {
    active_variables: Vec<VariableId>,
    universe: GraphColoredVertices,
    processes: Vec<Box<dyn Process<RoundRobinScheduler>>>,
    discarded: Option<GraphColoredVertices>,
}

pub struct ReachAfterPostProcess {
    variable: VariableId,
    fwd: FwdProcess,
}

pub struct ExtendedComponentProcess {
    variable: VariableId,
    fwd_set: GraphColoredVertices,
    bwd: BwdProcess,
}

struct FwdProcess {
    fwd: GraphColoredVertices,
    universe: GraphColoredVertices,
}

struct BwdProcess {
    bwd: GraphColoredVertices,
    universe: GraphColoredVertices,
}
