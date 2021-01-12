use crate::process::{FwdProcess, Process, Scheduler};
use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, GraphColoredVertices};

impl FwdProcess {

    pub fn new(initial: &GraphColoredVertices, universe: &GraphColoredVertices) -> FwdProcess {
        FwdProcess {
            fwd: initial.intersect(universe),
            universe: universe.clone(),
        }
    }

    pub fn reach_set(&self) -> &GraphColoredVertices {
        &self.fwd
    }
}

impl<S: Scheduler> Process<S> for FwdProcess {

    fn step(&mut self, scheduler: &mut S, graph: &SymbolicAsyncGraph) -> bool {
        let variables = scheduler.variables();
        if variables.is_empty() { return true; }   // Just in case...
        let mut i_var = variables.len() - 1;
        loop {  // Only return after we have successfully changed something.
            let var = variables[i_var];

            let post = graph
                .var_post(var, &self.fwd)
                .intersect(&self.universe)
                .minus(&self.fwd);

            if post.is_empty() {
                if i_var == 0 {  // We are done!
                    return true;
                } else {                        // Go to next variable.
                    i_var -= 1;
                }
            } else {                            // Apply post and reset.
                self.fwd = self.fwd.union(&post);
                return false;
            }
        }
    }

    fn weight(&self) -> usize {
        self.fwd.as_bdd().size()
    }

    fn discard_states(&mut self, set: &GraphColoredVertices) {
        self.fwd = self.fwd.minus(set);
        self.universe = self.universe.minus(set);
    }

}