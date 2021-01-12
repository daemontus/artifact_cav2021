use crate::process::{ExtendedComponentProcess, BwdProcess, Process, Scheduler};
use biodivine_lib_param_bn::VariableId;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use crate::algorithms::reach_bwd;
use crate::log_message;

impl ExtendedComponentProcess {
    pub fn new(
        variable: VariableId,
        fwd_set: &GraphColoredVertices,
        universe: &GraphColoredVertices,
        graph: &SymbolicAsyncGraph
    ) -> ExtendedComponentProcess {
        let var_can_post = graph.var_can_post(variable, universe);
        ExtendedComponentProcess {
            variable,
            fwd_set: fwd_set.clone(),
            bwd: BwdProcess::new(&var_can_post, fwd_set),
        }
    }
}

impl<S: Scheduler> Process<S> for ExtendedComponentProcess {
    fn step(&mut self, scheduler: &mut S, graph: &SymbolicAsyncGraph) -> bool {
        if self.bwd.step(scheduler, graph) {
            let extended_component = self.bwd.reach_set();

            let bottom_region = self.fwd_set.minus(&extended_component);

            if !bottom_region.is_empty() {
                log_message("Compute BOTTOM set basin.");
                let basin = reach_bwd(
                    graph, scheduler.variables(), &bottom_region, scheduler.universe()
                ).minus(&bottom_region);
                if !basin.is_empty() {
                    scheduler.discard_states(&basin);
                }
            }

            let var_can_post = graph.var_can_post(self.variable, scheduler.universe());
            if var_can_post.is_empty() {
                scheduler.discard_variable(self.variable);
            }

            true
        } else {
            false
        }
    }

    fn weight(&self) -> usize {
        Process::<S>::weight(&self.bwd)
    }

    fn discard_states(&mut self, set: &GraphColoredVertices) {
        Process::<S>::discard_states(&mut self.bwd, set);
    }
}