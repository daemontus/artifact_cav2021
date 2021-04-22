# [Paper 163] Artifact Evaluation Package

This is a replicability package for the CAV2021 paper 163, *"Computing Bottom SCCs Symbolically Using Transition Guided Reduction"*. It contains the implementation of algorithms presented in the paper, as well as all benchmark models used for experiments. This readme then contains instructions on how to re-compute main experimental results of the paper. The instructions should be compatible with unix-based systems (Linux, MacOS). The main binaries are also compatible with Windows, so you should be able to run individual tests, but the benchmark automation requires unix-specific utilities.

There is also a virtual machine for this evaluation package available at [zenodo](10.5281/zenodo.4709882) (username and password are `elementary`). However, this package only requires installation of rather basic dependencies (Python 3, Rust compiler, and LaTeX to generate the figures) with no extra configuration, so we encourage you to just follow the instructions directly on your machine. 

> The original paper evaluates three benchmark sets (together over 300 models) and uses a timeout of 1 hour for each computation. As a consequence of this timeout, comparisons with slower algorithms can take a very long time (>24h) to complete. To make the evaluation feasible, we provide the option to set custom timeout (1 minute is usually sufficient to demonstrate the point). The generated figures will be thus cut-off at the chosen timeout, whereas in the paper, they extend to the full one hour span.

## Installing Dependencies

> On the artifact virtual machine, all dependencies are pre-installed and you can skip this step.

- This package uses `python3` for running benchmark automation scripts. Please make sure you have Python 3 installed and available (the exact version used by authors is `3.9.2`). 

- To run individual benchmarks, we use `timeout` (or `gtimeout` on MacOS) to enforce a time limit for each benchmark, so please also make sure you have `coreutils` installed. On Linux, these are usually included out of the box, on MacOS, you can use [brew](https://brew.sh) to install them using `brew install coreutils`. 

- To also run comparisons with CABEAN, you should have an environmental variable `CABEAN_BIN` set to the path of the CABEAN binary (if you don't want to run CABEAN, you can skip this step). CABEAN can be downloaded [here](https://satoss.uni.lu/software/CABEAN/). The `2.0.0` release is also included in this artifact inside the `CABEAN` directory, so that you can set `CABEAN_BIN` to either `CABEAN/MacOS/cabean` or `CABEAN/WindowsLinux/cabean` if you want to use the bundled version.

- Finally, you will need the Rust compiler. We recommend following the instructions on [rustlang.org](https://www.rust-lang.org/learn/get-started) (default configuration should be sufficient) instead of using a package manager, however either method should work ok. When using the official installer, everything is stored in `~/.cargo`, so admin privilages are not necessary. Once you are done, you can uninstall the compiler by running `rustup self uninstall`. The tested version of Rust is `1.50.0` (Feb 2021), but the project should be backwards compatible at least up to the "Rust 2018" edition...

- Rust will automatically download and cache all other libraries necessary to compile the project. You should therefore have internet access while running the commands for the first time. You can force rust do download all dependencies by running `cargo fetch`.

- To generate the comaprison figures, you need LaTeX. Please follow installation instructions for your platform. Alternatively, you can simply compare `.csv` output files instead of figures, in which case LaTeX is not necessary.

- You can run `python3 env_test.py` to verify that your machine is set up correctly. You should get an output similar to this:

  ```
  >>>>>>>>>> PRE-BENCHMARK CHECKS
  CABEAN path: CABEAN-release-2.0.0/BinaryFile/MacOS/cabean
  CABEAN executable ok.
  Timeout utility ok.
  Rust compiler installed.
  rustc 1.50.0 (cb75ad5db 2021-02-10)
  >>>>>>>>>> CHECK COMPLETED
  ```

## Benchmark models

We evaluate three benchmark sets. Since our implementation requires `.aeon` files and CABEAN requires `.bnet` file format, benchmark sets contain multiple copies of the same model, but in different formats.

- `benchmarks_real_life` is a collection of 125 real life models (up to 350 variables) from diferent model databases and tools.  In `benchmarks_sbml`, there are corresponding original SBML files as well as links to the sources where the models were obtained.
- `benchmarks_random` are 100 randomly generated models ranging from 30 to 1000 variables.
- **[optional]** `benchmarks_random_1000` are 100 randomly generated models, all with approximately 1000 variables. This benchmark set is quite resource-intensive (even the best algorithm will take several hours to complete the whole run), so we mark this as an "optional" evaluation step.

#### [Optional] Random model generator

If you want to inspect the random model generator, the source is available in `./src/bin/benchmark_generator.rs` and can be executed in the following way:

```bash
# Generate 100 models with 20-200 variables into a `my_benchmark` directory.
# The file names will be `[benchmark_id]_[variables]_[regulations].aeon`
cargo run --release --bin benchmark_generator -- 20 200 my_benchmark
```

To convert models from `.aeon` to `.bnet`, you can use another binary (source located in `./src/bin/aeon_to_bnet.rs`):

```bash
# Convert all `.aeon` files in `./my_benchmark` to `.bnet`, and
# place them into `./my_benchmark_bnet`.
cargo run --release --bin aeon_to_bnet -- my_benchmark my_benchmark_bnet
```

## Running benchmarks

To execute all benchmarks in a particular directory, please run:

```bash
# Arguments: timeout, benchmark folder, algorithm, plus optional `-i` flag 
# that enables "interactive" mode.
python3 bench_run.py 1m <bench_folder> ITGR 
```

The script will automatically compile the test binaries, or check that CABEAN is installed if you are trying to use it. It then takes three mandatory arguments:

- The timeout string: you can use for example `10s`, `1m` , `10m`, `1h`, ..., but `1m` or `30s` is usually sufficient.
- The benchmark folder. Warning: this is not an arbirary path; you *have to* run the benchmark script inside the artifact root folder.
- Algorithm: This can be `CABEAN`, `ITGR`, `TGR` or `BASIC`. See paper for description of each.

- Additionally, you can add an `-i` flag, which will then force the script to prompt you before each benchmark and give you the option to skip it.

The benchmark will print every command that is being executed, so for debugging, you can also copy these commands and execute them separately. We recommend you first start an interactive benchmark run to verify everything is working (you can always terminate an interactive run by typing `abort` before the next benchmark), for example:

```bash
python3 bench_run.py 10s benchmarks_real_life ITGR -i
```

If everything works, you can execute the same command without `-i` and it should go through all benchmarks automatically.

As output, the run will create a new folder `benchmarks_real_life_run_[timestamp]` that contains the output of each command as a separate file, as well as two `.csv` files that contain (1) the runtime of each command (`*_times.csv`) and (2) the number of commands that finished *before* a certain time (`*_aggregated.csv`).

#### Real life benchmarks

To compute results for the real life models, execute the following four commands:

```bash
python3 bench_run.py 1m benchmarks_real_life ITGR
python3 bench_run.py 1m benchmarks_real_life TGR
python3 bench_run.py 1m benchmarks_real_life CABEAN
python3 bench_run.py 1m benchmarks_real_life BASIC
```

The first three runs should be relatively fast, finishing in several minutes. The last run is the slowest algorithm and can take roughly one hour to complete all instances. If you do not have enough time, you can also replace `1m` with `30s`, or even a smaller value, but keep in mind that then your data will be cut off at that timepoint.

> If you have enough CPU cores available, feel free to run the commands in parallel so that all four experiments can run concurrently.

#### Random benchmarks

Similarly, you can compute the (smaller) random benchmarks using these commands:

```bash
python3 bench_run.py 30s benchmarks_random ITGR
python3 bench_run.py 30s benchmarks_random TGR
python3 bench_run.py 30s benchmarks_random CABEAN
python3 bench_run.py 30s benchmarks_random BASIC
```

These benchmarks contain larger models that will timeout more, and will therefore take longer to complete. We thus recommend a `30s` timeout. With `1m` timeout, the slower algorithms can take up to 100 minutes to complete. With `30s`, this should be a slightly more reasonable 50 minutes.

>  **[optional]** You can also try computing instances in the `benchmarks_random_1000`, but to the best of our knowledge, `TGR`, `CABEAN` or `BASIC` cannot compute even a single benchmark in this set. For the `ITGR` algorithm, the runtime should be usually around 10-15 minutes per model.

## Compare results

Once the results are computed, you can compare them to the files in `expected_real_life_1m` and `expected_random_1m` folders. Here, we included our output from the experiments in the format as captured by the benchmark runner.

If you wish to compare the results visually, you can copy the `*_aggregated.csv` files from your test runs to the `figures` folder, where we have a prepared `figures.tex` document which will automatically generate graphs similar to what is presented in the paper. Note that the `figures.tex` assumes all 8 `.csv` files are present. If you don't have all the `.csv` files yet, you can comment out the corresponding `\addplot`  commands to render incomplete plots. We also include an `expected_figures_1m.pdf` so that you can compare the results to the expected plots.

The plots in `figures.tex` correspond to Figures 2 (left) and Figure 3 (left and right) as presented in the paper. We do not have a fully automated script to produce Figure 2 (right) since it requires matching CABEAN and ITGR data together. However, you can roughly compare the results based on the `.csv` files (i.e. it should be visible that for the same models, ITGR is faster and by how much). Remember that your plots will have a cut off at the timeout you used when computing your benchmarks. Note that the absolute speed will be different on every computer (especially in a virtual machine), but the overall trend should be preserved.

The point of the figures is to illustrate that: (a) `ITGR` is much faster than `CABEAN`, (b) `ITGR` can easily compute all real life instances (with the one minute timeout, one model is not computed, this model requires a 15 minute timeout), (c) Interleaving ensures the technique scales to large models, i.e. note the strong lead of `ITGR` with respect to `TGR` in the random benchmarks.

## Availibility and Extendability

This artifact and this tutorial are available on [Github](https://github.com/daemontus/artifact_cav2021) and via a pre-configured virtual machine available at [zenodo](10.5281/zenodo.4709882) (username and password are `elementary`).

The implementation is based on two of our already published Rust libraries: [biodivine-lib-bdd](https://crates.io/crates/biodivine-lib-bdd) and [biodivine-lib-param-bn](https://crates.io/crates/biodivine-lib-param-bn) which facilitate the symbolic encoding of Boolean networks. The actual algorithms are then contained in `src/algorithms.rs` and `src/process/*` (implementation of reduction via process-based interleaving). The implementation itself is fairly minimal (well under 1000 LOC) and contains basic comments that explain the function of individual components.

Everything is open-source and available with the permissive MIT License.

The implementation is used (with additional modifications) in our tool [AEON](https://biodivine.fi.muni.cz/aeon), which you can also use to visualize and modify the Boolean networks included in this artifact. To simply open and modify the `.aeon` files (for example by changing the Boolean update functions for individual variables), you can use the online interface available directly on the website. To also compute and visualise the BSCCs using AEON, you will need a native *compute engine* that AEON can connect to (download link for which can be found in the online interface). A full introduction to AEON is beyond the scope of this readme, but feel free to reach out at `sybila@fi.muni.cz` if you run into any problems when using the tool.