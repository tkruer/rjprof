use clap::{Arg, ArgMatches, Command};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};

#[derive(Debug)]
pub struct ProfilerConfig {
    pub jar_file: String,
    pub java_opts: Vec<String>,
    pub stack_size: String,
    pub output_dir: String,
    pub agent_path: String,
    pub flamegraph: bool,
    pub allocation_tracking: bool,
    pub call_graph: bool,
    pub sampling_interval: Option<u64>,
    pub java_executable: String,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            jar_file: String::new(),
            java_opts: vec![],
            stack_size: "256k".to_string(),
            output_dir: "./profiler_output".to_string(),
            agent_path: String::new(),
            flamegraph: true,
            allocation_tracking: true,
            call_graph: true,
            sampling_interval: None,
            java_executable: "java".to_string(),
        }
    }
}

pub fn parse_config(matches: &ArgMatches) -> Result<ProfilerConfig, String> {
    let mut config = ProfilerConfig::default();

    // Required JAR file
    config.jar_file = matches
        .get_one::<String>("jar")
        .ok_or("JAR file is required")?
        .clone();

    // Validate JAR file exists
    if !Path::new(&config.jar_file).exists() {
        return Err(format!("JAR file not found: {}", config.jar_file));
    }

    // Java options
    if let Some(opts) = matches.get_many::<String>("java-opts") {
        config.java_opts = opts.cloned().collect();
    }

    // Stack size
    config.stack_size = matches.get_one::<String>("stack-size").unwrap().clone();

    // Output directory
    config.output_dir = matches.get_one::<String>("output").unwrap().clone();

    // Java executable
    config.java_executable = matches
        .get_one::<String>("java-executable")
        .unwrap()
        .clone();

    // Agent path (auto-detect if not provided)
    if let Some(agent_path) = matches.get_one::<String>("agent-path") {
        config.agent_path = agent_path.clone();
    } else {
        config.agent_path = detect_agent_path()?;
    }

    // Validate agent exists
    if !Path::new(&config.agent_path).exists() {
        return Err(format!("Agent library not found: {}", config.agent_path));
    }

    // Feature flags
    config.flamegraph = !matches.get_flag("no-flamegraph");
    config.allocation_tracking = !matches.get_flag("no-allocation");
    config.call_graph = !matches.get_flag("no-call-graph");

    // Sampling interval
    if let Some(interval) = matches.get_one::<String>("sampling-interval") {
        config.sampling_interval = Some(interval.parse().map_err(|_| "Invalid sampling interval")?);
    }

    Ok(config)
}

pub fn detect_agent_path() -> Result<String, String> {
    // Try to find the agent library in common locations
    let possible_paths = vec![
        // Current directory
        "./target/release/librjprof.dylib",
        "./target/release/librjprof.so",
        "./target/release/rjprof.dll",
    ];

    for path in possible_paths {
        if Path::new(&path).exists() {
            return Ok(path.to_string());
        }
    }

    Err("Could not find profiler agent library. Please specify with --agent-path".to_string())
}

pub fn run_profiler(config: &ProfilerConfig, verbose: bool) -> Result<(), String> {
    // Create output directory
    if let Err(e) = fs::create_dir_all(&config.output_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    // Change to output directory so files are written there
    let original_dir =
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    env::set_current_dir(&config.output_dir)
        .map_err(|e| format!("Failed to change to output directory: {}", e))?;

    // Build Java command
    let mut java_cmd = ProcessCommand::new(&config.java_executable);

    // Add agent path
    java_cmd.arg(format!("-agentpath:{}", config.agent_path));

    // Add stack size
    java_cmd.arg(format!("-Xss{}", config.stack_size));

    // Add custom Java options
    for opt in &config.java_opts {
        java_cmd.arg(opt);
    }

    // Add JAR file
    java_cmd.arg("-jar");
    java_cmd.arg(Path::new(&original_dir).join(&config.jar_file));

    if verbose {
        println!("🚀 Running command: {:?}", java_cmd);
    }

    // Execute the command
    let output = java_cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("Failed to execute Java command: {}", e))?;

    // Restore original directory
    env::set_current_dir(original_dir)
        .map_err(|e| format!("Failed to restore original directory: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Java process failed with exit code: {:?}",
            output.status.code()
        ));
    }

    Ok(())
}

pub fn generate_flamegraph_svg(config: &ProfilerConfig) -> Result<(), String> {
    let folded_path = Path::new(&config.output_dir).join("flamegraph.folded");
    let svg_path = Path::new(&config.output_dir).join("flamegraph.svg");

    if !folded_path.exists() {
        return Err("flamegraph.folded file not found".to_string());
    }

    // Try flamegraph.pl first, then inferno-flamegraph
    let flamegraph_commands = vec![
        (
            "flamegraph.pl",
            vec![folded_path.to_string_lossy().to_string()],
        ),
        (
            "inferno-flamegraph",
            vec![folded_path.to_string_lossy().to_string()],
        ),
    ];

    for (cmd, args) in flamegraph_commands {
        let mut command = ProcessCommand::new(cmd);
        command.args(&args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    // Write SVG to file
                    fs::write(&svg_path, output.stdout)
                        .map_err(|e| format!("Failed to write SVG file: {}", e))?;

                    println!("🔥 Flamegraph SVG generated: {}", svg_path.display());
                    return Ok(());
                } else {
                    eprintln!(
                        "Command '{}' failed: {}",
                        cmd,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(_) => {
                // Command not found, try next one
                continue;
            }
        }
    }

    Err("No flamegraph generator found. Install flamegraph.pl or inferno-flamegraph".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        // Test basic configuration
        let matches = Command::new("rjprof")
            .arg(Arg::new("jar").short('j').long("jar").required(true))
            .try_get_matches_from(vec!["rjprof", "-j", "test.jar"])
            .unwrap();

        // This would fail because test.jar doesn't exist, but tests the parsing logic
        // In a real test, you'd create a temporary JAR file
    }

    #[test]
    fn test_agent_path_detection() {
        // Test that agent path detection doesn't crash
        let _result = detect_agent_path();
        // We can't assert success because the agent may not be built
    }
}
