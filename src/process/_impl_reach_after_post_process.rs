use crate::algorithms::reach_bwd;
use crate::log_message;
use crate::process::{
    ExtendedComponentProcess, FwdProcess, Process, ReachAfterPostProcess, Scheduler,
};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

impl ReachAfterPostProcess {
    pub fn new(
        variable: VariableId,
        graph: &SymbolicAsyncGraph,
        universe: &GraphColoredVertices,
    ) -> ReachAfterPostProcess {
        let var_can_post = graph.var_can_post(variable, universe);
        ReachAfterPostProcess {
            variable,
            fwd: FwdProcess::new(&var_can_post, universe),
        }
    }
}

impl<S: Scheduler> Process<S> for ReachAfterPostProcess {
    fn step(&mut self, scheduler: &mut S, graph: &SymbolicAsyncGraph) -> bool {
        if self.fwd.step(scheduler, graph) {
            let fwd_set = self.fwd.reach_set();

            if fwd_set != scheduler.universe() {
                log_message("Compute FWD set basin.");
                let basin = reach_bwd(graph, scheduler.variables(), fwd_set, scheduler.universe())
                    .minus(fwd_set);
                if !basin.is_empty() {
                    scheduler.discard_states(&basin);
                }
            }

            scheduler.spawn(ExtendedComponentProcess::new(
                self.variable,
                fwd_set,
                scheduler.universe(),
                graph,
            ));

            true
        } else {
            false
        }
    }

    fn weight(&self) -> usize {
        Process::<S>::weight(&self.fwd)
    }

    fn discard_states(&mut self, set: &GraphColoredVertices) {
        Process::<S>::discard_states(&mut self.fwd, set);
    }
}
