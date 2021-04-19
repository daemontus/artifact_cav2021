use biodivine_lib_param_bn::BooleanNetwork;
use std::convert::TryFrom;
use std::path::PathBuf;

/// This binary will just run over all benchmarks in `./benchmarks_sbml` and dump their `.aeon`
/// versions to a designated output folder. This is useful when you want to start a new benchmark set.
fn main() {
    let benchmarks = std::fs::read_dir("./benchmarks_sbml").unwrap();
    let args: Vec<String> = std::env::args().into_iter().collect();
    if args.len() < 2 {
        eprintln!("Please give output path as argument.");
    }
    let out_dir = PathBuf::from(args[1].clone());
    if !out_dir.exists() {
        std::fs::create_dir_all(&out_dir).unwrap();
    }

    for bench_dir in benchmarks.into_iter().map(|it| it.unwrap()) {
        if !bench_dir.file_type().unwrap().is_dir() {
            continue;
        }
        let bench_name = bench_dir.file_name().to_str().unwrap().to_string();
        let model_path = bench_dir.path().join("model.aeon");
        let model_string = std::fs::read_to_string(model_path).unwrap();
        // Check that the network is ok
        let model = BooleanNetwork::try_from(model_string.as_str()).unwrap();
        // Dump it to out-folder
        let out_file = out_dir.join(&format!("{}.aeon", bench_name));
        std::fs::write(out_file, model.to_string()).unwrap();
        println!("Copied {}.", bench_name);
    }
}
