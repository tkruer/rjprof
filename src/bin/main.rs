// src/main.rs
use clap::{Arg, Command};
use rjprof::cli::cli_tooling::{generate_flamegraph_svg, parse_config, run_profiler};

fn main() {
    let matches = Command::new("rjprof")
        .version("1.0.0")
        .author("Tyler Kruer tyler@tkruer.com>")
        .about("Rust-based Java profiler with flamegraph generation")
        .arg(
            Arg::new("jar")
                .short('j')
                .long("jar")
                .value_name("JAR_FILE")
                .help("JAR file to profile")
                .required(true),
        )
        .arg(
            Arg::new("java-opts")
                .short('J')
                .long("java-opts")
                .value_name("OPTS")
                .help("Additional Java options (can be used multiple times)")
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("stack-size")
                .short('s')
                .long("stack-size")
                .value_name("SIZE")
                .help("Stack size (default: 256k)")
                .default_value("256k"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("DIR")
                .help("Output directory for profiling results")
                .default_value("./profiler_output"),
        )
        .arg(
            Arg::new("agent-path")
                .short('a')
                .long("agent-path")
                .value_name("PATH")
                .help("Path to the profiler agent library (auto-detected if not specified)"),
        )
        .arg(
            Arg::new("no-flamegraph")
                .long("no-flamegraph")
                .help("Disable flamegraph generation")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-allocation")
                .long("no-allocation")
                .help("Disable allocation tracking")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-call-graph")
                .long("no-call-graph")
                .help("Disable call graph analysis")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("sampling-interval")
                .long("sampling-interval")
                .value_name("MS")
                .help("Sampling interval in milliseconds (for future sampling support)"),
        )
        .arg(
            Arg::new("java-executable")
                .long("java")
                .value_name("PATH")
                .help("Path to Java executable")
                .default_value("java"),
        )
        .arg(
            Arg::new("generate-flamegraph")
                .long("generate-flamegraph")
                .help("Generate SVG flamegraph after profiling (requires flamegraph.pl or inferno)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("sort")
                .long("sort")
                .value_name("FIELD")
                .help("Sort results by field: total, self, calls, name, percentage")
                .default_value("self"),
        )
        .arg(
            Arg::new("min-total")
                .long("min-total")
                .value_name("TIME")
                .help("Hide methods with total time below threshold (e.g., 1ms, 100us)"),
        )
        .arg(
            Arg::new("min-percentage")
                .long("min-pct")
                .value_name("PERCENT")
                .help("Hide methods below percentage of total time (e.g., 0.1)"),
        )
        .arg(
            Arg::new("no-color")
                .long("no-color")
                .help("Disable colorized output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("raw-times")
                .long("raw-times")
                .help("Show raw nanosecond times instead of human-readable")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("export")
                .long("export")
                .value_name("FORMAT")
                .help("Export results to file: json, csv"),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .value_name("PATTERNS")
                .help("Exclude packages/classes (e.g., java.*,spring.*)")
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("include")
                .long("include")
                .value_name("PATTERNS")
                .help("Include only these packages/classes (e.g., com.myapp.*)")
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("min-self-time")
                .long("min-self-time")
                .value_name("TIME")
                .help("Hide methods with self-time below threshold (e.g., 100us)"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_name("MODE")
                .help("Profile mode: all, user, hotspots, allocation")
                .default_value("user"),
        )
        .arg(
            Arg::new("spring")
                .long("spring")
                .help("Enable Spring-optimized filtering (excludes common Spring noise)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let config = match parse_config(&matches) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if matches.get_flag("verbose") {
        println!("🔧 Configuration:");
        println!("  JAR file: {}", config.jar_file);
        println!("  Agent path: {}", config.agent_path);
        println!("  Output directory: {}", config.output_dir);
        println!("  Stack size: {}", config.stack_size);
        println!("  Java executable: {}", config.java_executable);
        println!(
            "  Features: flamegraph={}, allocation={}, call-graph={}",
            config.flamegraph, config.allocation_tracking, config.call_graph
        );
    }

    if let Err(e) = run_profiler(&config, matches.get_flag("verbose")) {
        eprintln!("Error running profiler: {}", e);
        std::process::exit(1);
    }

    if matches.get_flag("generate-flamegraph") {
        if let Err(e) = generate_flamegraph_svg(&config) {
            eprintln!("Warning: Failed to generate flamegraph SVG: {}", e);
        }
    }

    println!(
        "✅ Profiling complete! Results saved to: {}",
        config.output_dir
    );
}
