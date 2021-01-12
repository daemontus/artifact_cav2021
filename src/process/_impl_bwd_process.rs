use crate::process::{Process, Scheduler, BwdProcess};
use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, GraphColoredVertices};

impl BwdProcess {
    pub fn new(initial: &GraphColoredVertices, universe: &GraphColoredVertices) -> BwdProcess {
        BwdProcess {
            bwd: initial.intersect(universe),
            universe: universe.clone(),
        }
    }

    pub fn reach_set(&self) -> &GraphColoredVertices {
        &self.bwd
    }
}

impl<S: Scheduler> Process<S> for BwdProcess {

    fn step(&mut self, scheduler: &mut S, graph: &SymbolicAsyncGraph) -> bool {
        let variables = scheduler.variables();
        if variables.is_empty() { return true; }   // Just in case...
        let mut i_var = variables.len() - 1;
        loop {  // Only return after we have successfully changed something.
            let var = variables[i_var];

            let pre = graph
                .var_pre(var, &self.bwd)
                .intersect(&self.universe)
                .minus(&self.bwd);

            if pre.is_empty() {
                if i_var == 0 {  // We are done!
                    return true;
                } else {                        // Go to next variable.
                    i_var -= 1;
                }
            } else {                            // Apply post and reset.
                self.bwd = self.bwd.union(&pre);
                return false;
            }
        }
    }

    fn weight(&self) -> usize {
        self.bwd.as_bdd().size()
    }

    fn discard_states(&mut self, set: &GraphColoredVertices) {
        self.bwd = self.bwd.minus(set);
        self.universe = self.universe.minus(set);
    }

}