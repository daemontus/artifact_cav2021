use biodivine_lib_param_bn::{BinaryOp, BooleanNetwork, FnUpdate};
use std::convert::TryFrom;
use std::path::PathBuf;

/// This binary takes an input directory with `.aeon` benchmarks and converts them all to `.bnet`
/// files, putting the output files into a designated output directory (second argument).
///
/// Any variables with unknown update functions will just cause a panic...
fn main() {
    let args: Vec<String> = std::env::args().into_iter().collect();
    if args.len() < 3 {
        eprintln!("Please give input/output path as first and second argument.");
    }
    let in_dir = PathBuf::from(args[1].clone());
    if !in_dir.exists() {
        panic!("Input directory does not exist.")
    }
    let out_dir = PathBuf::from(args[2].clone());
    if !out_dir.exists() {
        std::fs::create_dir_all(&out_dir).unwrap();
    }

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
        let out_file = out_dir.join(&format!("{}.bnet", bench_name));
        std::fs::write(out_file, network_to_bnet(&model)).unwrap();
        println!("Copied {}.", bench_name);
    }
}

fn network_to_bnet(network: &BooleanNetwork) -> String {
    let mut model = format!("targets,factors\n");
    for v in network.variables() {
        let v_id: usize = v.into();
        let line = format!(
            "v{}, {}\n",
            v_id,
            fnupdate_to_bnet_string(network.get_update_function(v).as_ref().unwrap())
        );
        model.push_str(&line);
    }
    model
}

fn fnupdate_to_bnet_string(fn_update: &FnUpdate) -> String {
    match fn_update {
        FnUpdate::Param(_, _) => panic!("Parameters not allowed."),
        FnUpdate::Const(value) => {
            if *value {
                // There is always v1
                format!("v1 | !v1",)
            } else {
                format!("v1 & !v1",)
            }
        }
        FnUpdate::Var(id) => {
            let id: usize = (*id).into();
            format!("v{}", id)
        }
        FnUpdate::Not(inner) => format!("!{}", fnupdate_to_bnet_string(inner)),
        FnUpdate::Binary(op, l, r) => {
            let left = fnupdate_to_bnet_string(l);
            let right = fnupdate_to_bnet_string(r);
            let op = match *op {
                BinaryOp::And => "&",
                BinaryOp::Or => "|",
                _ => panic!("{:?} not supported.", op),
            };
            format!("({} {} {})", left, op, right)
        }
    }
}
