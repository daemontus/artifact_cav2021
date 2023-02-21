use crate::log_message;
use crate::process::{Process, RoundRobinScheduler, Scheduler};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

impl RoundRobinScheduler {
    pub fn new(graph: &SymbolicAsyncGraph, universe: &GraphColoredVertices) -> RoundRobinScheduler {
        RoundRobinScheduler {
            active_variables: graph.as_network().variables().collect(),
            universe: universe.clone(),
            processes: Vec::new(),
            discarded: None,
        }
    }
}

impl Scheduler for RoundRobinScheduler {
    fn step(&mut self, graph: &SymbolicAsyncGraph) -> usize {
        // First, apply discarded states to processes
        if let Some(discarded) = &self.discarded {
            for process in self.processes.iter_mut() {
                process.discard_states(discarded);
            }
            self.discarded = None;
        }

        // Second, perform one step in each process, but return only those that are not finished.
        // We have to do it like this to preserve ownership constraints, as each process needs
        // to run with a mutable reference to the scheduler.
        let process_count_start = self.processes.len();
        let drained_processes = self.processes.drain(0..).collect::<Vec<_>>();
        for mut process in drained_processes {
            if !process.step(self, graph) {
                self.processes.push(process);
            }
        }
        let terminated = process_count_start - self.processes.len();
        if terminated > 0 {
            log_message(&format!("Remaining processes: {};", self.processes.len()));
        }

        process_count_start
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
            "Remaining universe: {}",
            self.universe.approx_cardinality()
        ));
        if let Some(discarded) = self.discarded.as_mut() {
            *discarded = discarded.union(set);
        } else {
            self.discarded = Some(set.clone());
        }
    }

    fn spawn_process(&mut self, process: Box<dyn Process<Self>>) {
        self.processes.push(process);
    }

    fn universe(&self) -> &GraphColoredVertices {
        &self.universe
    }

    fn variables(&self) -> &[VariableId] {
        &self.active_variables
    }
}
