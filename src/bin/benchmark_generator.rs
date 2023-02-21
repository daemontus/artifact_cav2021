use biodivine_lib_param_bn::{
    BinaryOp, BooleanNetwork, FnUpdate, Monotonicity, RegulatoryGraph, VariableId,
};
use cav2021_artifact::connectivity_distribution::connectivity_distribution;
use cav2021_artifact::in_degree_relative_distribution::in_degree_relative_distribution;
use cav2021_artifact::out_degree_relative_distribution::out_degree_relative_distribution;
use cav2021_artifact::SampledDistribution;
use core::panic;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::min;
use std::collections::HashSet;
use std::path::PathBuf;

const P_REG_POSITIVE: f64 = 0.8066337893732103;

const OUTPUT_FORMAT: &str = "bnet"; // Possible values: aeon/bnet/sbml
const INPUT_NODE_FORMAT: &str = "identity"; // Possible values: true/false/identity/random
const NETWORK_COUNT: usize = 100;
const RANDOM_SEED: u64 = 123456789;

fn main() {
    let args: Vec<String> = std::env::args().into_iter().collect();
    if args.len() < 4 {
        eprintln!("Please give number of variable range and output path as arguments.");
    }
    let num_vars_min = args[1].parse::<usize>().unwrap();
    let num_vars_max = args[2].parse::<usize>().unwrap();
    let out_dir = PathBuf::from(args[3].clone());
    if !out_dir.exists() {
        std::fs::create_dir_all(&out_dir).unwrap();
    }

    let connectivity = connectivity_distribution();
    //let max_in_degree_distribution = max_in_degree_distribution();
    //let max_out_degree_distribution = max_out_degree_distribution();
    let relative_in_degree_distribution = in_degree_relative_distribution();
    let relative_out_degree_distribution = out_degree_relative_distribution();

    let mut random = StdRng::seed_from_u64(RANDOM_SEED);

    let mut i_model = 1;
    while i_model <= NETWORK_COUNT {
        let num_vars = random.gen_range(num_vars_min..num_vars_max);
        let sampled = connectivity.sample(&mut random);
        let regulations = ((num_vars as f64) * sampled).round() as usize;
        println!("Connectivity: {} (sampled {})", regulations, sampled);

        let mut in_degrees = make_degree_vector_from_relative_distribution(
            &relative_in_degree_distribution,
            num_vars,
            regulations,
            &mut random,
        );
        let mut out_degrees = make_degree_vector_from_relative_distribution(
            &relative_out_degree_distribution,
            num_vars,
            regulations,
            &mut random,
        );

        // At this point, the actual number of regulations by in/out degrees can be different.
        // For convenience reasons, we pick the smaller one and continue with that.
        let regulations: usize = min(in_degrees.iter().sum(), out_degrees.iter().sum());

        let variables = (1..(num_vars + 1))
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>();
        let mut rg = RegulatoryGraph::new(variables.clone());
        for _ in 0..regulations {
            let source = pick_from_degree_vector(&out_degrees, &mut random);
            let target = pick_from_degree_vector(&in_degrees, &mut random);
            if rg
                .find_regulation(
                    VariableId::from_index(source),
                    VariableId::from_index(target),
                )
                .is_none()
            {
                out_degrees[source] -= 1;
                in_degrees[target] -= 1;
                let monotonicity = if random.gen_bool(P_REG_POSITIVE) {
                    Monotonicity::Activation
                } else {
                    Monotonicity::Inhibition
                };
                rg.add_regulation(
                    &variables[source],
                    &variables[target],
                    true,
                    Some(monotonicity),
                )
                .unwrap();
            }
        }
        // If "identity" input nodes are required, we need to
        // add self-loop regulations to all input nodes.
        // These do not "count" towards our desired regulation count.
        if INPUT_NODE_FORMAT == "identity" {
            for var in rg.variables() {
                if rg.regulators(var).is_empty() {
                    let name = rg.get_variable_name(var).clone();
                    rg.add_regulation(&name, &name, true, Some(Monotonicity::Activation))
                        .unwrap();
                }
            }
        }

        let mut bn = BooleanNetwork::new(rg.clone());
        for v in bn.variables() {
            let regulators = bn.regulators(v);
            if regulators.is_empty() && !bn.targets(v).is_empty() {
                // Only add constant functions for variables that will appear in the network
                // (if there are no targets, the variable will be just lost in translation)
                let function = match INPUT_NODE_FORMAT {
                    "true" => FnUpdate::Const(true),
                    "false" => FnUpdate::Const(false),
                    "identity" => FnUpdate::Var(v),
                    "random" => FnUpdate::Const(random.gen_bool(0.5)),
                    _ => panic!("Unknown node input format: {}.", INPUT_NODE_FORMAT),
                };
                bn.add_update_function(v, function).unwrap();
            } else if !regulators.is_empty() {
                let r = regulators[0];
                let fst_is_activation = rg.find_regulation(r, v).unwrap().get_monotonicity()
                    == Some(Monotonicity::Activation);
                let mut fn_update = if fst_is_activation {
                    FnUpdate::Var(r)
                } else {
                    FnUpdate::Not(Box::new(FnUpdate::Var(r)))
                };
                for r in regulators.iter().cloned().skip(1) {
                    let op = if random.gen_bool(0.5) {
                        BinaryOp::And
                    } else {
                        BinaryOp::Or
                    };
                    let is_activation = rg.find_regulation(r, v).unwrap().get_monotonicity()
                        == Some(Monotonicity::Activation);
                    let var = if is_activation {
                        FnUpdate::Var(r)
                    } else {
                        FnUpdate::Not(Box::new(FnUpdate::Var(r)))
                    };
                    fn_update = FnUpdate::Binary(op, Box::new(fn_update), Box::new(var));
                }
                bn.add_update_function(v, fn_update).unwrap();
            }
        }

        let actual_var_count = bn
            .variables()
            .filter(|v| !bn.regulators(*v).is_empty() || !bn.targets(*v).is_empty())
            .count();
        // At this point, the network must be at the very least weakly connected:
        if !is_weak_connected(&bn, actual_var_count) {
            println!("Not weakly connected. Skipping...");
            continue;
        }

        let output_file = out_dir.join(&format!(
            "{}_{}_{}.{}",
            i_model, actual_var_count, regulations, OUTPUT_FORMAT
        ));
        let output_content = match OUTPUT_FORMAT {
            "aeon" => bn.to_string(),
            "bnet" => bn.to_bnet(true).unwrap(),
            "sbml" => bn.to_sbml(None),
            _ => panic!("Unknown output format {}.", OUTPUT_FORMAT),
        };
        std::fs::write(output_file, output_content).unwrap();
        println!("{} generated...", i_model);
        i_model += 1;
    }
}

fn is_weak_connected(network: &BooleanNetwork, expected: usize) -> bool {
    let mut reachable = HashSet::new();
    let mut todo = vec![network.variables().next().unwrap()];
    while let Some(var) = todo.pop() {
        reachable.insert(var);
        for r in network.as_graph().regulators(var) {
            if !reachable.contains(&r) {
                todo.push(r);
            }
        }
        for r in network.as_graph().targets(var) {
            if !reachable.contains(&r) {
                todo.push(r);
            }
        }
    }
    println!("Reachable: {} Expected: {}", reachable.len(), expected);
    reachable.len() == expected
}

fn pick_from_degree_vector(degree_vector: &[usize], random: &mut StdRng) -> usize {
    let total: usize = degree_vector.iter().sum();
    let mut picked = random.gen_range(0..total);
    let mut i = 0;
    while i < degree_vector.len() && picked >= degree_vector[i] {
        picked -= degree_vector[i];
        i += 1;
    }
    i
}

/// Takes a `[0,1]` relative distribution and produces a vector of `[0,length]` values of
/// length `length` that should follow the given distribution and their sum should be as close
/// as possible to `total`.
fn make_degree_vector_from_relative_distribution(
    distribution: &SampledDistribution,
    length: usize,
    total: usize,
    random: &mut StdRng,
) -> Vec<usize> {
    // First, pick random values from the distribution - these are [0,1].
    let samples = vector_of_samples(distribution, length, random);
    // Now we make actual "degree-like" values from them by multiplying with `length - 1`.
    let degree_samples = apply_multiplicative_factor(&samples, (length - 1) as f64);
    // Then turn them back into [0,1] values, but this time relative to the sum of the whole vector.
    let normalized_degree_samples = normalize_samples(&degree_samples);
    // Finally, use these [0,1] values to generate degrees that should get us very close to `total`
    normalized_degree_samples
        .into_iter()
        .map(|f| (f * (total as f64)).round() as usize)
        .collect()
}

fn vector_of_samples(
    distribution: &SampledDistribution,
    size: usize,
    random: &mut StdRng,
) -> Vec<f64> {
    (0..size).map(|_| distribution.sample(random)).collect()
}

fn apply_multiplicative_factor(samples: &[f64], factor: f64) -> Vec<f64> {
    samples.iter().map(|s| s * factor).collect()
}

fn normalize_samples(samples: &[f64]) -> Vec<f64> {
    let total = samples.iter().sum::<f64>();
    samples.iter().map(|s| *s / total).collect()
}
