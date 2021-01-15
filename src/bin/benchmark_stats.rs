use biodivine_lib_param_bn::{BooleanNetwork, Monotonicity};
use std::convert::TryFrom;
use std::path::PathBuf;

/// Takes a sample of a discrete distribution and normalizes it to `[0,1]` floating-point interval.
fn normalize_discrete_distribution(distribution: &Vec<usize>, factor: f64) -> Vec<f64> {
    //let max = distribution.iter().max().unwrap().clone() as f64;
    return distribution.iter().map(|i| (*i as f64) / factor).collect();
}

/// Computes a cumulative distribution function from the given distribution sample.
///
/// The cumulative distribution will always have 129 sample points between the `[min, max]`
/// values of the input distribution. The first value is the value from this interval, the second
/// value is the probability of original value being smaller or equal to this number.
fn interval_cumulative_distribution_density(
    distribution: &[f64],
    min: f64,
    max: f64,
) -> Vec<(f64, f64)> {
    let mut distribution = distribution.to_vec();
    distribution.sort_by(|a, b| a.partial_cmp(b).unwrap());
    //let min = distribution[0];
    //let max = distribution[distribution.len() - 1];
    let mut sample_values = subdivide((min, max)) // 2
        .into_iter()
        .flat_map(|i| subdivide(i)) // 4
        .flat_map(|i| subdivide(i)) // 8
        .flat_map(|i| subdivide(i)) // 16
        .flat_map(|i| subdivide(i)) // 32
        .flat_map(|i| subdivide(i)) // 64
        .flat_map(|i| subdivide(i)) // 128
        .map(|(_, b)| b)
        .collect::<Vec<_>>();
    sample_values.insert(0, min);
    let values = distribution.len() as f64;
    sample_values
        .into_iter()
        .map(|sample| {
            let smaller = distribution.iter().filter(|a| **a < sample).count() as f64;
            (sample, smaller / values)
        })
        .collect()
}

fn cumulative_distribution_density(distribution: &[f64]) -> Vec<(f64, f64)> {
    let min = distribution
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .clone();
    let max = distribution
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .clone();
    interval_cumulative_distribution_density(distribution, min, max)
}

fn subdivide(interval: (f64, f64)) -> Vec<(f64, f64)> {
    let mid = interval.0 + ((interval.1 - interval.0) / 2.0);
    vec![(interval.0, mid), (mid, interval.1)]
}

fn merge(d1: &[(f64, f64)], d2: &[(f64, f64)]) -> Vec<(f64, f64)> {
    d1.iter()
        .zip(d2.iter())
        .map(|((s1, v1), (s2, v2))| {
            if *s1 != *s2 {
                panic!("WTF: {:?} {:?}", d1, d2)
            }
            (*s1, *v1 + *v2)
        })
        .collect()
}

/// Takes an input path, reads all `.aeon` models in that path and dumps useful statistics about them.
///
/// At the moment, these include:
///  - network scc distribution
///  - network in-degree distribution
///  - network out-degree distribution
///  - average connectivity
///
/// Furthermore, for the entire dataset, it also dumps the distribution of average connectivity.
fn main() {
    let args: Vec<String> = std::env::args().into_iter().collect();
    if args.len() < 2 {
        eprintln!("Please give input path as first argument.");
    }
    let in_dir = PathBuf::from(args[1].clone());
    if !in_dir.exists() {
        panic!("Input directory does not exist.")
    }

    let mut bench_count = 0;
    let mut connectivity_distribution_sample = Vec::new();
    let mut total_positive = 0;
    let mut total_negative = 0;
    let mut max_in_degree_normalized_sample = Vec::new();
    let mut max_out_degree_normalized_sample = Vec::new();
    // Initially, the distribution is zero everywhere
    let mut in_degree_cumulative_average: Vec<(f64, f64)> =
        cumulative_distribution_density(&vec![0.0, 1.0])
            .into_iter()
            .map(|(sample, _)| (sample, 0.0))
            .collect();
    let mut out_degree_cumulative_average: Vec<(f64, f64)> = in_degree_cumulative_average.clone();

    let listing = std::fs::read_dir(in_dir.as_path()).unwrap();
    for bench in listing.into_iter().map(|it| it.unwrap()) {
        if !bench.file_name().to_str().unwrap().ends_with(".aeon") {
            continue; // skip non .aeon files
        }
        let bench_name = bench
            .path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string();
        let model_string = std::fs::read_to_string(bench.path()).unwrap();
        let model = BooleanNetwork::try_from(model_string.as_str()).unwrap();
        println!("Processing {};", bench_name);
        let num_vars = model.num_vars() as f64;
        let num_regs = model.as_graph().regulations().count() as f64;
        let average_connectivity = num_regs / num_vars;
        connectivity_distribution_sample.push(average_connectivity);
        total_positive += model
            .as_graph()
            .regulations()
            .filter(|it| it.get_monotonicity() == Some(Monotonicity::Activation))
            .count();
        total_negative += model
            .as_graph()
            .regulations()
            .filter(|it| it.get_monotonicity() == Some(Monotonicity::Inhibition))
            .count();
        println!("Average connectivity: {}", average_connectivity);
        let mut in_degree_distribution = model
            .variables()
            .map(|v| model.regulators(v).len())
            .collect::<Vec<_>>();
        // one fake zero will make cumulative density much nicer to work with, because after normalization, we will always have [0,1] min-max interval.
        in_degree_distribution.sort();
        in_degree_distribution.reverse();
        max_in_degree_normalized_sample.push((in_degree_distribution[0] as f64) / num_vars);
        let in_degree_normalized =
            normalize_discrete_distribution(&in_degree_distribution, num_vars - 1.0);
        let in_degree_cumulative_density =
            interval_cumulative_distribution_density(&in_degree_normalized, 0.0, 1.0);
        in_degree_cumulative_average =
            merge(&in_degree_cumulative_average, &in_degree_cumulative_density);
        println!("In-degree distribution:\n{:?}", in_degree_distribution);
        let mut out_degree_distribution = model
            .variables()
            .map(|v| model.targets(v).len())
            .collect::<Vec<_>>();
        out_degree_distribution.sort();
        out_degree_distribution.reverse();
        max_out_degree_normalized_sample.push(out_degree_distribution[0] as f64 / num_vars);
        let out_degree_normalized =
            normalize_discrete_distribution(&out_degree_distribution, num_vars - 1.0);
        let out_degree_cumulative_density =
            interval_cumulative_distribution_density(&out_degree_normalized, 0.0, 1.0);
        out_degree_cumulative_average = merge(
            &out_degree_cumulative_average,
            &out_degree_cumulative_density,
        );
        println!("Out-degree distribution:\n{:?}", out_degree_distribution);
        let mut scc_distribution = model
            .as_graph()
            .components()
            .into_iter()
            .map(|scc| scc.len())
            .collect::<Vec<_>>();
        scc_distribution.sort();
        scc_distribution.reverse();
        println!("SCC distribution:\n{:?}", scc_distribution);
        bench_count += 1;
    }

    let average_connectivity = connectivity_distribution_sample.iter().sum::<f64>()
        / (connectivity_distribution_sample.len() as f64);
    println!("Average connectivity: {}", average_connectivity);
    let connectivity_density = cumulative_distribution_density(&connectivity_distribution_sample);
    println!("{:?}", connectivity_density);
    println!("connectivity, P[connectivity < X]");
    for (sample, value) in connectivity_density {
        println!("{}, {}", sample, value);
    }

    let max_in_degree_density = cumulative_distribution_density(&max_in_degree_normalized_sample);
    println!("max-in-degree, P[max-in-degree < X]");
    for (sample, value) in max_in_degree_density {
        println!("{}, {}", sample, value);
    }

    let max_out_degree_density = cumulative_distribution_density(&max_out_degree_normalized_sample);
    println!("max-out-degree, P[max-out-degree < X]");
    for (sample, value) in max_out_degree_density {
        println!("{}, {}", sample, value);
    }

    in_degree_cumulative_average = in_degree_cumulative_average
        .into_iter()
        .map(|(a, b)| (a, b / bench_count as f64))
        .collect();

    println!("{:?}", in_degree_cumulative_average);
    println!("relative-in-degree, P[relative-in-degree < X]");
    for (sample, value) in in_degree_cumulative_average {
        println!("{}, {}", sample, value);
    }

    out_degree_cumulative_average = out_degree_cumulative_average
        .into_iter()
        .map(|(a, b)| (a, b / bench_count as f64))
        .collect();

    println!("{:?}", out_degree_cumulative_average);
    println!("relative-out-degree, P[relative-out-degree < X]");
    for (sample, value) in out_degree_cumulative_average {
        println!("{}, {}", sample, value);
    }

    println!(
        "Average monotonicity: {}",
        (total_positive as f64) / ((total_positive + total_negative) as f64)
    );
}
