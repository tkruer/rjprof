//! # librjprof
//! 
//! A Java profiling library using JVMTI (Java Virtual Machine Tool Interface).
//! This library provides low-level profiling capabilities for Java applications,
//! including method timing, allocation tracking, and call graph analysis.
//! 
//! ## Features
//! 
//! - Method entry/exit profiling with timing
//! - Memory allocation tracking
//! - Call graph analysis  
//! - Flamegraph generation
//! - Smart filtering to reduce framework noise
//! 
//! ## Usage
//! 
//! This library is typically used by attaching it as a JVMTI agent to a Java process:
//! 
//! ```bash
//! java -agentpath:librjprof.so MyJavaApp
//! ```
//! 
//! The library can also be configured programmatically through the profiler configuration.

pub mod bindings;
pub mod profiling;

// Re-export key types for easier access
pub use profiling::profiling::{set_profiler_config};

// Re-export configuration types from CLI (we'll need to move these)
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

/// Initialize the profiler with the given configuration.
/// This should be called before attaching the JVMTI agent.
pub fn configure_profiler(config: ProfilerConfig) {
    set_profiler_config(config);
}

/// Get the default profiler configuration with sensible defaults.
pub fn default_config() -> ProfilerConfig {
    ProfilerConfig::default()
}

/// Create a configuration optimized for Spring Boot applications.
pub fn spring_config() -> ProfilerConfig {
    let mut config = ProfilerConfig::default();
    config.exclude_packages = get_spring_excludes();
    config.profile_mode = ProfileMode::UserCode;
    config
}