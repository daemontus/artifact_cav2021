use crate::log_message;
use crate::process::{PriorityScheduler, Process, Scheduler};
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

impl PriorityScheduler {
    pub fn new(graph: &SymbolicAsyncGraph, universe: &GraphColoredVertices) -> PriorityScheduler {
        PriorityScheduler {
            active_variables: graph.network().variables().collect(),
            universe: universe.clone(),
            processes: Vec::new(),
            discarded: None,
        }
    }
}

impl Scheduler for PriorityScheduler {
    fn step(&mut self, graph: &SymbolicAsyncGraph) -> usize {
        // First, apply discarded states to processes
        if let Some(discarded) = &self.discarded {
            for (w, process) in self.processes.iter_mut() {
                process.discard_states(discarded);
                *w = process.weight();
            }
            self.discarded = None;
        }

        // Second, find the best process to follow based on current BDD complexity
        self.processes.sort_by_key(|(w, _)| *w);

        if self.processes.is_empty() {
            return 0;
        } else if self.processes.len() == 1 {
            // If there is only one process, finalize it (it may spawn something though).
            let (_, mut process) = self.processes.remove(0);
            let mut iter = 0;
            while !process.step(self, graph) {
                iter += 1;
            }
            return iter;
        } else {
            let (_, mut process) = self.processes.remove(0);
            let target_weight = self.processes[0].0;
            // Keep running process until it finishes or exceeds the weight of the
            // second smallest process. If it finishes, we just continue and return
            // self.processes.is_empty(); If it exceeds the weight of the second process,
            // we put it back into the process list and return false (since processes
            // are clearly not empty).
            let mut iter = 0;
            while !process.step(self, graph) {
                iter += 1;
                if process.weight() > target_weight {
                    self.processes.push((process.weight(), process));
                    return iter;
                }
            }

            log_message(&format!("Remaining processes: {}", self.processes.len()));
            return iter + 1;
        }
    }

    fn finalize(self) -> (GraphColoredVertices, Vec<VariableId>) {
        (self.universe, self.active_variables)
    }

    fn discard_variable(&mut self, variable: VariableId) {
        self.active_variables = self
            .active_variables
            .iter()
            .filter(|v| **v != variable)
            .cloned()
            .collect();
        log_message(&format!(
            "Remaining variables: {}",
            self.active_variables.len()
        ));
    }

    fn discard_states(&mut self, set: &GraphColoredVertices) {
        self.universe = self.universe.minus(set);
        log_message(&format!(
            "Remaining universe: {}({})",
            self.universe.approx_cardinality(),
            self.universe.as_bdd().size(),
        ));
        if let Some(discarded) = self.discarded.as_mut() {
            *discarded = discarded.union(set);
        } else {
            self.discarded = Some(set.clone());
        }
    }

    fn spawn_process(&mut self, process: Box<dyn Process<Self>>) {
        self.processes.push((process.weight(), process));
    }

    fn universe(&self) -> &GraphColoredVertices {
        &self.universe
    }

    fn variables(&self) -> &[VariableId] {
        &self.active_variables
    }
}
