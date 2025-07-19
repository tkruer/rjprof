use clap::ArgMatches;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command as ProcessCommand, Stdio};

#[derive(Debug, Clone)]
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
    pub sort_by: SortOption,
    pub min_total_ns: Option<u64>,
    pub min_percentage: Option<f64>,
    pub colorized: bool,
    pub export_format: Option<ExportFormat>,
    pub human_readable: bool,
    pub exclude_packages: Vec<String>,
    pub include_packages: Vec<String>,
    pub min_self_time_ns: Option<u64>,
    pub profile_mode: ProfileMode,
}

#[derive(Debug, Clone)]
pub enum SortOption {
    TotalTime,
    SelfTime,
    Calls,
    Name,
    Percentage,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
}

#[derive(Debug, Clone)]
pub enum ProfileMode {
    All,           // Profile everything
    UserCode,      // Only user code (exclude JDK/framework)
    Hotspots,      // Focus on methods above threshold
    Allocation,    // Focus on allocation-heavy methods
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
            sort_by: SortOption::SelfTime,
            min_total_ns: None,
            min_percentage: None,
            colorized: true,
            export_format: None,
            human_readable: true,
            exclude_packages: get_default_excludes(),
            include_packages: vec![],
            min_self_time_ns: None,
            profile_mode: ProfileMode::UserCode,
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

    // Sort option
    if let Some(sort) = matches.get_one::<String>("sort") {
        config.sort_by = match sort.as_str() {
            "total" => SortOption::TotalTime,
            "self" => SortOption::SelfTime,
            "calls" => SortOption::Calls,
            "name" => SortOption::Name,
            "percentage" | "pct" => SortOption::Percentage,
            _ => return Err(format!("Invalid sort option: {}. Use: total, self, calls, name, percentage", sort)),
        };
    }

    // Threshold filtering
    if let Some(min_total) = matches.get_one::<String>("min-total") {
        config.min_total_ns = Some(parse_time_to_nanos(min_total)?);
    }

    if let Some(min_pct) = matches.get_one::<String>("min-percentage") {
        config.min_percentage = Some(min_pct.parse().map_err(|_| "Invalid percentage value")?);
    }

    // Output options
    config.colorized = !matches.get_flag("no-color");
    config.human_readable = !matches.get_flag("raw-times");

    // Export format
    if let Some(format) = matches.get_one::<String>("export") {
        config.export_format = Some(match format.as_str() {
            "json" => ExportFormat::Json,
            "csv" => ExportFormat::Csv,
            _ => return Err(format!("Invalid export format: {}. Use: json, csv", format)),
        });
    }

    // Package filtering
    if matches.get_flag("spring") {
        config.exclude_packages = get_spring_excludes();
    } else if let Some(excludes) = matches.get_many::<String>("exclude") {
        config.exclude_packages = excludes.cloned().collect();
    }
    
    if let Some(includes) = matches.get_many::<String>("include") {
        config.include_packages = includes.cloned().collect();
    }

    // Min self time threshold
    if let Some(min_self) = matches.get_one::<String>("min-self-time") {
        config.min_self_time_ns = Some(parse_time_to_nanos(min_self)?);
    }

    // Profile mode
    if let Some(mode) = matches.get_one::<String>("mode") {
        config.profile_mode = match mode.as_str() {
            "all" => ProfileMode::All,
            "user" | "usercode" => ProfileMode::UserCode,
            "hotspots" => ProfileMode::Hotspots,
            "allocation" | "alloc" => ProfileMode::Allocation,
            _ => return Err(format!("Invalid profile mode: {}. Use: all, user, hotspots, allocation", mode)),
        };
    }

    Ok(config)
}

fn parse_time_to_nanos(time_str: &str) -> Result<u64, String> {
    let time_str = time_str.trim().to_lowercase();
    
    if let Some(pos) = time_str.find(|c: char| c.is_alphabetic()) {
        let (number_part, unit_part) = time_str.split_at(pos);
        let number: f64 = number_part.parse().map_err(|_| "Invalid time number")?;
        
        let multiplier = match unit_part {
            "ns" => 1,
            "us" | "μs" => 1_000,
            "ms" => 1_000_000,
            "s" => 1_000_000_000,
            _ => return Err(format!("Invalid time unit: {}. Use: ns, us, ms, s", unit_part)),
        };
        
        Ok((number * multiplier as f64) as u64)
    } else {
        // Assume nanoseconds if no unit specified
        time_str.parse().map_err(|_| "Invalid time value".to_string())
    }
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
    // Set the global configuration for the profiling module
    crate::profiling::profiling::set_profiler_config(config.clone());
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

fn get_default_excludes() -> Vec<String> {
    vec![
        // JDK internals
        "java.*".to_string(),
        "javax.*".to_string(),
        "sun.*".to_string(),
        "com.sun.*".to_string(),
        "jdk.*".to_string(),
        
        // Spring Framework (common noise)
        "org.springframework.*".to_string(),
        "org.apache.catalina.*".to_string(),
        "org.apache.tomcat.*".to_string(),
        "org.eclipse.jetty.*".to_string(),
        
        // Common libraries that create noise
        "org.apache.logging.*".to_string(),
        "ch.qos.logback.*".to_string(),
        "org.slf4j.*".to_string(),
        "net.sf.cglib.*".to_string(),
        "org.apache.commons.*".to_string(),
        
        // Reflection/Proxying
        "$$EnhancerBySpringCGLIB*".to_string(),
        "$$FastClassBySpringCGLIB*".to_string(),
        "com.sun.proxy.*".to_string(),
        
        // Build tools
        "org.gradle.*".to_string(),
        "org.apache.maven.*".to_string(),
    ]
}

pub fn get_spring_excludes() -> Vec<String> {
    let mut excludes = get_default_excludes();
    excludes.extend(vec![
        "org.springframework.*".to_string(),
        "org.apache.catalina.*".to_string(),
        "org.apache.tomcat.*".to_string(),
        "org.hibernate.*".to_string(),
        "org.apache.coyote.*".to_string(),
        "org.eclipse.jetty.*".to_string(),
        "io.undertow.*".to_string(),
        "reactor.*".to_string(),
        "io.netty.*".to_string(),
    ]);
    excludes
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
