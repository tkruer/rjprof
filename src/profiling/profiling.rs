use core::ffi::c_char;
use std::os::raw::c_void;
use std::ptr;

use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

use crate::bindings::gen_bindings::*;
use crate::cli::cli_tooling::{ProfilerConfig, SortOption, ExportFormat, ProfileMode};

thread_local! {
    static ENTRY_TIMES: RefCell<HashMap<jmethodID, u64>> = RefCell::new(HashMap::new());
    static CALL_STACK: RefCell<Vec<jmethodID>> = RefCell::new(Vec::new());
}

/// Newtype wrapper for JVMTI method IDs, so we can safely share across threads.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct MethodId(jmethodID);
unsafe impl Send for MethodId {}
unsafe impl Sync for MethodId {}

/// Per-method call count and total time.
#[derive(Clone, Copy, Debug)]
struct MethodStats {
    count: u64,
    total_nanos: u64,
    self_nanos: u64,
}

/// Per-method allocation statistics
#[derive(Clone, Copy, Default, Debug)]
struct AllocationStats {
    object_count: u64,
    total_bytes: u64,
}

/// Per-class allocation statistics
#[derive(Clone, Default, Debug)]
struct ClassAllocationStats {
    object_count: u64,
    total_bytes: u64,
    class_name: String,
}

/// Call relationship statistics
#[derive(Clone, Copy, Debug)]
struct CallRelation {
    call_count: u64,
    total_time_nanos: u64,
}

/// Call graph edge (caller -> callee)
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct CallEdge {
    caller: MethodId,
    callee: MethodId,
}

/// Represents a call stack frame for flamegraph generation
#[derive(Clone, Debug)]
struct StackFrame {
    method_id: MethodId,
    start_time: u64,
    children: Vec<StackFrame>,
}

/// Flamegraph stack sample
#[derive(Clone, Debug)]
struct FlameStackSample {
    stack: Vec<String>, // Method names from root to leaf
    self_time: u64,     // Time spent in the leaf method
}

static METHOD_STATS: Lazy<Mutex<HashMap<MethodId, MethodStats>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static ALLOCATION_STATS: Lazy<Mutex<HashMap<MethodId, AllocationStats>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static CLASS_ALLOCATION_STATS: Lazy<Mutex<HashMap<String, ClassAllocationStats>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static CALL_GRAPH: Lazy<Mutex<HashMap<CallEdge, CallRelation>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// For flamegraph generation, we need to track the complete call stacks
static FLAMEGRAPH_SAMPLES: Lazy<Mutex<Vec<FlameStackSample>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

// Global JVMTI env for method info lookup
static mut GLOBAL_JVMTI_ENV: *mut jvmtiEnv = std::ptr::null_mut();

// Global configuration store
static GLOBAL_CONFIG: Lazy<Mutex<Option<ProfilerConfig>>> = Lazy::new(|| Mutex::new(None));

// Track method entry times with call stack depth for self-time calculation
thread_local! {
    static METHOD_ENTRY_STACK: RefCell<Vec<(jmethodID, u64)>> = RefCell::new(Vec::new());
    static FLAMEGRAPH_STACK: RefCell<Vec<StackFrame>> = RefCell::new(Vec::new());
}

pub fn set_profiler_config(config: ProfilerConfig) {
    *GLOBAL_CONFIG.lock().unwrap() = Some(config);
}

fn should_profile_method(class_name: &str, method_name: &str, config: &ProfilerConfig) -> bool {
    let full_name = format!("{}.{}", class_name, method_name);
    
    match config.profile_mode {
        ProfileMode::All => true,
        ProfileMode::UserCode => should_include_user_code(class_name, config),
        ProfileMode::Hotspots => true, // Apply hotspot filtering later
        ProfileMode::Allocation => true, // Apply allocation filtering later
    }
}

fn should_include_user_code(class_name: &str, config: &ProfilerConfig) -> bool {
    // If includes are specified, only allow those
    if !config.include_packages.is_empty() {
        return config.include_packages.iter().any(|pattern| {
            matches_pattern(class_name, pattern)
        });
    }
    
    // Otherwise, exclude common framework packages
    !config.exclude_packages.iter().any(|pattern| {
        matches_pattern(class_name, pattern)
    })
}

fn matches_pattern(class_name: &str, pattern: &str) -> bool {
    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        class_name.starts_with(prefix)
    } else {
        class_name == pattern
    }
}

fn should_track_allocation(class_name: &str, config: &ProfilerConfig) -> bool {
    match config.profile_mode {
        ProfileMode::All | ProfileMode::Allocation => true,
        ProfileMode::UserCode => should_include_user_code(class_name, config),
        ProfileMode::Hotspots => should_include_user_code(class_name, config),
    }
}

extern "C" fn method_entry_callback(
    jvmti_env: *mut jvmtiEnv,
    _jni_env: *mut JNIEnv,
    _thread: jthread,
    method: jmethodID,
) {
    unsafe {
        // Quick filtering check - get method info to determine if we should profile
        let (class_name, method_name, _) = get_method_info(jvmti_env, method);
        
        let config = {
            let config_guard = GLOBAL_CONFIG.lock().unwrap();
            config_guard.as_ref().cloned().unwrap_or_default()
        };
        
        if !should_profile_method(&class_name, &method_name, &config) {
            return;
        }

        let mut nano: jlong = 0;
        (**jvmti_env).GetTime.unwrap()(jvmti_env, &mut nano);
        let entry_time = nano as u64;

        // Track call graph relationships
        CALL_STACK.with(|stack| {
            let mut stack_ref = stack.borrow_mut();
            if let Some(&caller) = stack_ref.last() {
                let edge = CallEdge {
                    caller: MethodId(caller),
                    callee: MethodId(method),
                };

                let mut call_graph = CALL_GRAPH.lock().unwrap();
                let relation = call_graph.entry(edge).or_insert(CallRelation {
                    call_count: 0,
                    total_time_nanos: 0,
                });
                relation.call_count += 1;
            }
            stack_ref.push(method);
        });

        // Track method entry for timing
        METHOD_ENTRY_STACK.with(|stack| {
            stack.borrow_mut().push((method, entry_time));
        });

        // Track for flamegraph
        FLAMEGRAPH_STACK.with(|stack| {
            let frame = StackFrame {
                method_id: MethodId(method),
                start_time: entry_time,
                children: Vec::new(),
            };
            stack.borrow_mut().push(frame);
        });
    }
}

extern "C" fn method_exit_callback(
    jvmti_env: *mut jvmtiEnv,
    _jni_env: *mut JNIEnv,
    thread: jthread,
    method: jmethodID,
    _was_popped_by_exception: jboolean,
    _return_value: jvalue,
) {
    unsafe {
        let mut nano_exit: jlong = 0;
        (**jvmti_env).GetTime.unwrap()(jvmti_env, &mut nano_exit);
        let exit_time = nano_exit as u64;

        // Pop from call stack
        CALL_STACK.with(|stack| {
            let mut stack_ref = stack.borrow_mut();
            if let Some(popped) = stack_ref.pop() {
                if popped == method {
                    if let Some(&caller) = stack_ref.last() {
                        let edge = CallEdge {
                            caller: MethodId(caller),
                            callee: MethodId(method),
                        };

                        let mut call_graph = CALL_GRAPH.lock().unwrap();
                        if let Some(relation) = call_graph.get_mut(&edge) {
                            // Timing info updated in method stats
                        }
                    }
                }
            }
        });

        // Calculate timing and update stats
        METHOD_ENTRY_STACK.with(|stack| {
            let mut stack_ref = stack.borrow_mut();
            if let Some((entry_method, entry_time)) = stack_ref.pop() {
                if entry_method == method {
                    let total_duration = exit_time.saturating_sub(entry_time);
                    let child_time = 0u64; // Simplified for now

                    let mut stats = METHOD_STATS.lock().unwrap();
                    let entry = stats.entry(MethodId(method)).or_insert(MethodStats {
                        count: 0,
                        total_nanos: 0,
                        self_nanos: 0,
                    });
                    entry.count += 1;
                    entry.total_nanos += total_duration;
                    entry.self_nanos += total_duration.saturating_sub(child_time);

                    // Update call graph timing
                    CALL_STACK.with(|call_stack| {
                        let call_stack_ref = call_stack.borrow();
                        if let Some(&caller) = call_stack_ref.last() {
                            let edge = CallEdge {
                                caller: MethodId(caller),
                                callee: MethodId(method),
                            };

                            let mut call_graph = CALL_GRAPH.lock().unwrap();
                            if let Some(relation) = call_graph.get_mut(&edge) {
                                relation.total_time_nanos += total_duration;
                            }
                        }
                    });
                }
            }
        });

        // Handle flamegraph stack
        FLAMEGRAPH_STACK.with(|stack| {
            let mut stack_ref = stack.borrow_mut();
            if let Some(frame) = stack_ref.pop() {
                if frame.method_id.0 == method {
                    let duration = exit_time.saturating_sub(frame.start_time);

                    // Calculate self-time (time not spent in children)
                    let child_time: u64 = frame
                        .children
                        .iter()
                        .map(|child| child.start_time) // This would need proper duration tracking
                        .sum();

                    let self_time = duration.saturating_sub(child_time);

                    // Only create flamegraph sample if we have meaningful self-time
                    if self_time > 0 {
                        // Build the stack trace
                        let mut stack_trace = Vec::new();

                        // Add all parent frames
                        for parent_frame in stack_ref.iter() {
                            if let Some(method_name) =
                                get_method_name_safe(jvmti_env, parent_frame.method_id.0)
                            {
                                stack_trace.push(method_name);
                            }
                        }

                        // Add current frame
                        if let Some(method_name) = get_method_name_safe(jvmti_env, method) {
                            stack_trace.push(method_name);
                        }

                        // Add sample to flamegraph data
                        let sample = FlameStackSample {
                            stack: stack_trace,
                            self_time,
                        };

                        FLAMEGRAPH_SAMPLES.lock().unwrap().push(sample);
                    }
                }
            }
        });
    }
}

extern "C" fn vm_object_alloc_callback(
    jvmti_env: *mut jvmtiEnv,
    _jni_env: *mut JNIEnv,
    thread: jthread,
    object: jobject,
    object_klass: jclass,
    size: jlong,
) {
    unsafe {
        // Get class name for the allocated object
        let class_name = {
            let mut class_sig_ptr: *mut c_char = std::ptr::null_mut();
            let res = (**jvmti_env).GetClassSignature.unwrap()(
                jvmti_env,
                object_klass,
                &mut class_sig_ptr,
                std::ptr::null_mut(),
            );

            if res == jvmtiError_JVMTI_ERROR_NONE && !class_sig_ptr.is_null() {
                let class_sig = CStr::from_ptr(class_sig_ptr).to_string_lossy();
                let formatted = if class_sig.starts_with('L') && class_sig.ends_with(';') {
                    class_sig[1..class_sig.len() - 1].replace('/', ".")
                } else if class_sig.starts_with('[') {
                    format!("Array: {}", class_sig)
                } else {
                    class_sig.into_owned()
                };

                (**jvmti_env).Deallocate.unwrap()(jvmti_env, class_sig_ptr as *mut u8);
                formatted
            } else {
                "<unknown>".to_string()
            }
        };

        // Check if we should track this allocation
        let config = {
            let config_guard = GLOBAL_CONFIG.lock().unwrap();
            config_guard.as_ref().cloned().unwrap_or_default()
        };
        
        if !should_track_allocation(&class_name, &config) {
            return;
        }

        // Update class allocation stats
        {
            let mut class_stats = CLASS_ALLOCATION_STATS.lock().unwrap();
            let entry =
                class_stats
                    .entry(class_name.clone())
                    .or_insert_with(|| ClassAllocationStats {
                        object_count: 0,
                        total_bytes: 0,
                        class_name: class_name.clone(),
                    });
            entry.object_count += 1;
            entry.total_bytes += size as u64;
        }

        // Attribute allocation to current method
        CALL_STACK.with(|stack| {
            let stack_ref = stack.borrow();
            if let Some(&current_method) = stack_ref.last() {
                let mut alloc_stats = ALLOCATION_STATS.lock().unwrap();
                let entry = alloc_stats
                    .entry(MethodId(current_method))
                    .or_insert_with(Default::default);
                entry.object_count += 1;
                entry.total_bytes += size as u64;
            }
        });
    }
}

fn get_method_name_safe(jvmti_env: *mut jvmtiEnv, method: jmethodID) -> Option<String> {
    let (class_name, method_name, _) = get_method_info(jvmti_env, method);
    if class_name != "<unknown-class>" && method_name != "<unknown>" {
        Some(format!("{}.{}", class_name, method_name))
    } else {
        None
    }
}

fn get_method_info(jvmti_env: *mut jvmtiEnv, method: jmethodID) -> (String, String, String) {
    unsafe {
        let mut declaring_class: jclass = std::ptr::null_mut();
        let res =
            (**jvmti_env).GetMethodDeclaringClass.unwrap()(jvmti_env, method, &mut declaring_class);

        let class_name = if res == jvmtiError_JVMTI_ERROR_NONE {
            let mut class_sig_ptr: *mut c_char = std::ptr::null_mut();
            let res = (**jvmti_env).GetClassSignature.unwrap()(
                jvmti_env,
                declaring_class,
                &mut class_sig_ptr,
                std::ptr::null_mut(),
            );

            if res == jvmtiError_JVMTI_ERROR_NONE && !class_sig_ptr.is_null() {
                let class_sig = CStr::from_ptr(class_sig_ptr).to_string_lossy();
                let formatted = if class_sig.starts_with('L') && class_sig.ends_with(';') {
                    class_sig[1..class_sig.len() - 1].replace('/', ".")
                } else {
                    class_sig.into_owned()
                };

                (**jvmti_env).Deallocate.unwrap()(jvmti_env, class_sig_ptr as *mut u8);
                formatted
            } else {
                "<unknown-class>".to_string()
            }
        } else {
            "<unknown-class>".to_string()
        };

        let mut name_ptr: *mut c_char = std::ptr::null_mut();
        let mut sig_ptr: *mut c_char = std::ptr::null_mut();
        let res = (**jvmti_env).GetMethodName.unwrap()(
            jvmti_env,
            method,
            &mut name_ptr,
            &mut sig_ptr,
            std::ptr::null_mut(),
        );

        let (method_name, method_sig) = if res == jvmtiError_JVMTI_ERROR_NONE {
            let name = if !name_ptr.is_null() {
                let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
                (**jvmti_env).Deallocate.unwrap()(jvmti_env, name_ptr as *mut u8);
                name
            } else {
                "<unknown>".to_string()
            };

            let sig = if !sig_ptr.is_null() {
                let sig = CStr::from_ptr(sig_ptr).to_string_lossy().into_owned();
                (**jvmti_env).Deallocate.unwrap()(jvmti_env, sig_ptr as *mut u8);
                sig
            } else {
                String::new()
            };

            (name, sig)
        } else {
            ("<unknown>".to_string(), String::new())
        };

        (class_name, method_name, method_sig)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_time(nanos: u64) -> String {
    if nanos < 1000 {
        format!("{}ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.1}μs", nanos as f64 / 1000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.1}ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", nanos as f64 / 1_000_000_000.0)
    }
}

fn format_time_conditional(nanos: u64, human_readable: bool) -> String {
    if human_readable {
        format_time(nanos)
    } else {
        format!("{}ns", nanos)
    }
}

fn colorize_time_percentage(text: &str, percentage: f64, colorized: bool) -> String {
    if !colorized {
        return text.to_string();
    }
    
    if percentage >= 20.0 {
        format!("\x1b[31m{}\x1b[0m", text) // Red for >20%
    } else if percentage >= 5.0 {
        format!("\x1b[33m{}\x1b[0m", text) // Yellow for >5%
    } else if percentage >= 1.0 {
        format!("\x1b[36m{}\x1b[0m", text) // Cyan for >1%
    } else {
        text.to_string() // Normal for <1%
    }
}

#[derive(Clone, Debug)]
struct EnhancedMethodStats {
    method_id: MethodId,
    stats: MethodStats,
    method_name: String,
    class_name: String,
    percentage: f64,
}

fn export_to_json(stats: &[EnhancedMethodStats], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap;
    
    let mut json_data = HashMap::new();
    json_data.insert("profiling_results", stats.iter().map(|s| {
        let mut method_data = HashMap::new();
        method_data.insert("class_name", s.class_name.clone());
        method_data.insert("method_name", s.method_name.clone());
        method_data.insert("call_count", s.stats.count.to_string());
        method_data.insert("total_time_ns", s.stats.total_nanos.to_string());
        method_data.insert("self_time_ns", s.stats.self_nanos.to_string());
        method_data.insert("avg_total_ns", (s.stats.total_nanos / s.stats.count).to_string());
        method_data.insert("avg_self_ns", (s.stats.self_nanos / s.stats.count).to_string());
        method_data.insert("percentage", format!("{:.2}", s.percentage));
        method_data
    }).collect::<Vec<_>>());
    
    let json_string = format!("{:#?}", json_data); // Simple debug format for now
    std::fs::write(output_path, json_string)?;
    println!("📊 Results exported to JSON: {}", output_path);
    Ok(())
}

fn export_to_csv(stats: &[EnhancedMethodStats], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut csv_content = String::new();
    csv_content.push_str("class_name,method_name,call_count,total_time_ns,self_time_ns,avg_total_ns,avg_self_ns,percentage\n");
    
    for stat in stats {
        csv_content.push_str(&format!(
            "{},{},{},{},{},{},{},{:.2}\n",
            stat.class_name,
            stat.method_name,
            stat.stats.count,
            stat.stats.total_nanos,
            stat.stats.self_nanos,
            stat.stats.total_nanos / stat.stats.count,
            stat.stats.self_nanos / stat.stats.count,
            stat.percentage
        ));
    }
    
    std::fs::write(output_path, csv_content)?;
    println!("📊 Results exported to CSV: {}", output_path);
    Ok(())
}

fn generate_flamegraph_svg(
    samples: &[FlameStackSample],
) -> Result<String, Box<dyn std::error::Error>> {
    use std::collections::HashMap;

    // Aggregate samples by stack trace
    let mut aggregated: HashMap<Vec<String>, u64> = HashMap::new();

    for sample in samples {
        *aggregated.entry(sample.stack.clone()).or_insert(0) += sample.self_time;
    }

    // Sort by total time for better visualization
    let mut sorted_samples: Vec<_> = aggregated.into_iter().collect();
    sorted_samples.sort_by_key(|(_, time)| std::cmp::Reverse(*time));

    // Generate folded stack format for flamegraph
    let mut folded_output = String::new();
    for (stack, time) in sorted_samples {
        let stack_str = stack.join(";");
        folded_output.push_str(&format!("{} {}\n", stack_str, time));
    }

    Ok(folded_output)
}

fn write_flamegraph_data(jvmti_env: *mut jvmtiEnv) -> Result<(), Box<dyn std::error::Error>> {
    let samples = FLAMEGRAPH_SAMPLES.lock().unwrap();

    if samples.is_empty() {
        println!("No flamegraph samples collected");
        return Ok(());
    }

    // Generate folded stack format
    let folded_data = generate_flamegraph_svg(&samples)?;

    // Write to file
    let mut file = File::create("flamegraph.folded")?;
    file.write_all(folded_data.as_bytes())?;

    println!("🔥 Flamegraph data written to 'flamegraph.folded'");
    println!("   Generate SVG with: flamegraph.pl flamegraph.folded > flamegraph.svg");
    println!("   Or use: inferno-flamegraph flamegraph.folded > flamegraph.svg");

    // Also write a simple text summary
    let mut summary_file = File::create("flamegraph_summary.txt")?;
    writeln!(summary_file, "Flamegraph Summary")?;
    writeln!(summary_file, "==================")?;
    writeln!(summary_file, "Total samples: {}", samples.len())?;

    let total_time: u64 = samples.iter().map(|s| s.self_time).sum();
    writeln!(summary_file, "Total time: {}", format_time(total_time))?;

    // Top methods by self-time
    let mut method_times: HashMap<String, u64> = HashMap::new();
    for sample in samples.iter() {
        if let Some(method) = sample.stack.last() {
            *method_times.entry(method.clone()).or_insert(0) += sample.self_time;
        }
    }

    let mut sorted_methods: Vec<_> = method_times.into_iter().collect();
    sorted_methods.sort_by_key(|(_, time)| std::cmp::Reverse(*time));

    writeln!(summary_file, "\nTop 10 methods by self-time:")?;
    for (method, time) in sorted_methods.iter().take(10) {
        writeln!(summary_file, "{}: {}", method, format_time(*time))?;
    }

    Ok(())
}

extern "C" fn vm_death_callback(jvmti_env: *mut jvmtiEnv, _jni_env: *mut JNIEnv) {
    let config = {
        let config_guard = GLOBAL_CONFIG.lock().unwrap();
        config_guard.as_ref().cloned().unwrap_or_default()
    };

    println!("\n🔍 === ENHANCED PERFORMANCE ANALYSIS ===");
    
    // Display filtering info
    println!("Profile mode: {:?}", config.profile_mode);
    if !config.exclude_packages.is_empty() {
        println!("Excluding: {}", config.exclude_packages.join(", "));
    }
    if !config.include_packages.is_empty() {
        println!("Including only: {}", config.include_packages.join(", "));
    }
    if let Some(min_self) = config.min_self_time_ns {
        println!("Min self-time filter: {}", format_time_conditional(min_self, config.human_readable));
    }

    // Generate flamegraph data
    if config.flamegraph {
        if let Err(e) = write_flamegraph_data(jvmti_env) {
            eprintln!("Error writing flamegraph data: {}", e);
        }
    }

    // Collect and process method statistics
    let raw_stats: Vec<(MethodId, MethodStats)> = {
        let guard = METHOD_STATS.lock().unwrap();
        guard.iter().map(|(&m, st)| (m, *st)).collect()
    };

    if raw_stats.is_empty() {
        println!("No method statistics collected.");
        return;
    }

    // Calculate total time for percentage calculations
    let total_self_time: u64 = raw_stats.iter().map(|(_, st)| st.self_nanos).sum();

    // Create enhanced stats with percentage and method names
    let mut enhanced_stats: Vec<EnhancedMethodStats> = raw_stats
        .into_iter()
        .map(|(method_id, stats)| {
            let (class_name, method_name, _) = get_method_info(jvmti_env, method_id.0);
            let percentage = if total_self_time > 0 {
                (stats.self_nanos as f64 / total_self_time as f64) * 100.0
            } else {
                0.0
            };
            
            EnhancedMethodStats {
                method_id,
                stats,
                method_name,
                class_name,
                percentage,
            }
        })
        .collect();

    // Apply filtering
    if let Some(min_total) = config.min_total_ns {
        enhanced_stats.retain(|s| s.stats.total_nanos >= min_total);
    }
    if let Some(min_pct) = config.min_percentage {
        enhanced_stats.retain(|s| s.percentage >= min_pct);
    }
    if let Some(min_self_time) = config.min_self_time_ns {
        enhanced_stats.retain(|s| s.stats.self_nanos >= min_self_time);
    }

    // Apply mode-specific filtering
    match config.profile_mode {
        ProfileMode::All => {}, // No additional filtering
        ProfileMode::UserCode => {
            enhanced_stats.retain(|s| should_include_user_code(&s.class_name, &config));
        },
        ProfileMode::Hotspots => {
            enhanced_stats.retain(|s| s.percentage >= 1.0); // Only hotspots >1%
        },
        ProfileMode::Allocation => {
            // For allocation mode, we might want to show allocation-heavy methods
            // This could be enhanced to correlate with allocation stats
        }
    }

    // Apply sorting
    match config.sort_by {
        SortOption::TotalTime => enhanced_stats.sort_by_key(|s| std::cmp::Reverse(s.stats.total_nanos)),
        SortOption::SelfTime => enhanced_stats.sort_by_key(|s| std::cmp::Reverse(s.stats.self_nanos)),
        SortOption::Calls => enhanced_stats.sort_by_key(|s| std::cmp::Reverse(s.stats.count)),
        SortOption::Name => enhanced_stats.sort_by(|a, b| {
            format!("{}.{}", a.class_name, a.method_name)
                .cmp(&format!("{}.{}", b.class_name, b.method_name))
        }),
        SortOption::Percentage => enhanced_stats.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap()),
    }

    let display_count = std::cmp::min(enhanced_stats.len(), 20);

    // Display method performance statistics
    println!(
        "\n⏱️  === Top {} methods (sorted by {:?}) ===",
        display_count,
        config.sort_by
    );
    
    println!(
        "{:<45} {:>8} {:>10} {:>10} {:>10} {:>8}",
        "Method", "Calls", "Self Time", "Total Time", "Avg Self", "% Total"
    );
    println!("{}", "─".repeat(100));

    for stat in enhanced_stats.iter().take(display_count) {
        let method_str = format!("{}.{}", stat.class_name, stat.method_name);
        let method_display = if method_str.len() > 45 {
            format!("{}...", &method_str[..42])
        } else {
            method_str
        };

        let avg_self = stat.stats.self_nanos / stat.stats.count;
        let self_time_str = format_time_conditional(stat.stats.self_nanos, config.human_readable);
        let total_time_str = format_time_conditional(stat.stats.total_nanos, config.human_readable);
        let avg_self_str = format_time_conditional(avg_self, config.human_readable);
        let percentage_str = format!("{:>6.2}%", stat.percentage);

        let colored_percentage = colorize_time_percentage(&percentage_str, stat.percentage, config.colorized);
        let colored_method = if stat.percentage >= 20.0 && config.colorized {
            colorize_time_percentage(&method_display, stat.percentage, config.colorized)
        } else {
            method_display
        };

        println!(
            "{:<45} {:>8} {:>10} {:>10} {:>10} {}",
            colored_method,
            stat.stats.count,
            self_time_str,
            total_time_str,
            avg_self_str,
            colored_percentage
        );
    }

    // Export functionality
    if let Some(ref export_format) = config.export_format {
        let result = match export_format {
            ExportFormat::Json => export_to_json(&enhanced_stats, "profiling_results.json"),
            ExportFormat::Csv => export_to_csv(&enhanced_stats, "profiling_results.csv"),
        };
        
        if let Err(e) = result {
            eprintln!("Export error: {}", e);
        }
    }

    // Call graph analysis (if enabled)
    if config.call_graph {
        display_call_graph_analysis(jvmti_env, &config);
    }

    // Allocation analysis (if enabled)
    if config.allocation_tracking {
        display_allocation_analysis(jvmti_env, &config);
    }

    // Summary statistics
    println!("\n📊 === Summary Statistics ===");
    println!("Total methods analyzed: {}", enhanced_stats.len());
    println!("Total self-time: {}", format_time_conditional(total_self_time, config.human_readable));
    
    if let Some(top_method) = enhanced_stats.first() {
        println!(
            "Hottest method: {}.{} ({:.2}%)",
            top_method.class_name, top_method.method_name, top_method.percentage
        );
    }
}

fn display_call_graph_analysis(jvmti_env: *mut jvmtiEnv, config: &ProfilerConfig) {
    let call_graph = CALL_GRAPH.lock().unwrap();
    let mut call_relations: Vec<(CallEdge, CallRelation)> =
        call_graph.iter().map(|(&edge, &rel)| (edge, rel)).collect();
    call_relations.sort_by_key(|&(_, rel)| std::cmp::Reverse(rel.total_time_nanos));

    let top_calls = std::cmp::min(call_relations.len(), 15);
    if !call_relations.is_empty() {
        println!(
            "\n📞 === Top {} call relationships by total time ===",
            top_calls
        );
        
        println!(
            "{:<35} {:<35} {:>8} {:>10}",
            "Caller", "Callee", "Calls", "Avg Time"
        );
        println!("{}", "─".repeat(90));
        
        for (edge, rel) in call_relations.iter().take(top_calls) {
            let (caller_class, caller_method, _) = get_method_info(jvmti_env, edge.caller.0);
            let (callee_class, callee_method, _) = get_method_info(jvmti_env, edge.callee.0);

            let caller_short = format!("{}.{}", caller_class, caller_method);
            let callee_short = format!("{}.{}", callee_class, callee_method);
            let avg_time = rel.total_time_nanos / rel.call_count;

            let caller_display = if caller_short.len() > 35 {
                format!("{}...", &caller_short[..32])
            } else {
                caller_short
            };
            
            let callee_display = if callee_short.len() > 35 {
                format!("{}...", &callee_short[..32])
            } else {
                callee_short
            };

            println!(
                "{:<35} {:<35} {:>8} {:>10}",
                caller_display,
                callee_display,
                rel.call_count,
                format_time_conditional(avg_time, config.human_readable)
            );
        }
    }
}

fn display_allocation_analysis(jvmti_env: *mut jvmtiEnv, config: &ProfilerConfig) {
    // Method allocation stats
    let mut alloc_stats: Vec<(MethodId, AllocationStats)> = {
        let guard = ALLOCATION_STATS.lock().unwrap();
        guard.iter().map(|(&m, st)| (m, *st)).collect()
    };
    alloc_stats.sort_by_key(|&(_, st)| std::cmp::Reverse(st.total_bytes));
    let top_alloc = std::cmp::min(alloc_stats.len(), 10);

    if !alloc_stats.is_empty() {
        println!(
            "\n🏭 === Top {} methods by memory allocation ===",
            top_alloc
        );
        
        println!(
            "{:<45} {:>8} {:>12}",
            "Method", "Objects", "Total Bytes"
        );
        println!("{}", "─".repeat(70));
        
        for (MethodId(method), st) in alloc_stats.iter().take(top_alloc) {
            let (class_name, method_name, _) = get_method_info(jvmti_env, *method);
            let method_str = format!("{}.{}", class_name, method_name);
            let method_display = if method_str.len() > 45 {
                format!("{}...", &method_str[..42])
            } else {
                method_str
            };
            
            println!(
                "{:<45} {:>8} {:>12}",
                method_display,
                st.object_count,
                format_bytes(st.total_bytes)
            );
        }
    }

    // Class allocation stats
    let mut class_stats: Vec<ClassAllocationStats> = {
        let guard = CLASS_ALLOCATION_STATS.lock().unwrap();
        guard.values().cloned().collect()
    };
    class_stats.sort_by_key(|st| std::cmp::Reverse(st.total_bytes));
    let top_classes = std::cmp::min(class_stats.len(), 10);

    if !class_stats.is_empty() {
        println!(
            "\n📦 === Top {} classes by memory allocation ===",
            top_classes
        );
        
        println!(
            "{:<40} {:>8} {:>12}",
            "Class", "Objects", "Total Bytes"
        );
        println!("{}", "─".repeat(65));
        
        for st in class_stats.iter().take(top_classes) {
            let class_display = if st.class_name.len() > 40 {
                format!("{}...", &st.class_name[..37])
            } else {
                st.class_name.clone()
            };
            
            println!(
                "{:<40} {:>8} {:>12}",
                class_display,
                st.object_count,
                format_bytes(st.total_bytes)
            );
        }
    }
}

extern "C" fn vm_init_callback(jvmti_env: *mut jvmtiEnv, _jni_env: *mut JNIEnv, _thread: jthread) {
    unsafe {
        GLOBAL_JVMTI_ENV = jvmti_env;

        let mut thread_count: jint = 0;
        let mut threads: *mut jthread = ptr::null_mut();
        let err = (**jvmti_env).GetAllThreads.unwrap()(jvmti_env, &mut thread_count, &mut threads);

        println!("✅ [VM_INIT] JVM thread count: {}", thread_count);
        println!("📊 Call graph analysis & allocation tracking enabled");
        println!("🔥 Flamegraph generation enabled");
    }
}

#[no_mangle]
pub extern "C" fn Agent_OnAttach(vm: *mut JavaVM, _options: *mut c_char, _reserved: *mut c_void) {
    unsafe {
        let mut jvmti: *mut jvmtiEnv = ptr::null_mut();

        let get_env_fn = (**vm).GetEnv.unwrap();
        let res = get_env_fn(
            vm,
            (&mut jvmti) as *mut *mut jvmtiEnv as *mut *mut c_void,
            JVMTI_VERSION_1_2 as jint,
        );

        let mut caps = std::mem::zeroed::<jvmtiCapabilities>();
        caps.set_can_generate_method_entry_events(1);
        caps.set_can_generate_method_exit_events(1);
        caps.set_can_generate_vm_object_alloc_events(1);

        let err = (**jvmti).AddCapabilities.unwrap()(jvmti, &caps);
        if err != jvmtiError_JVMTI_ERROR_NONE {
            eprintln!("Failed to add JVMTI capabilities: {}", err);
        }

        let callbacks = jvmtiEventCallbacks {
            VMInit: Some(vm_init_callback),
            VMDeath: Some(vm_death_callback),
            MethodEntry: Some(method_entry_callback),
            MethodExit: Some(method_exit_callback),
            VMObjectAlloc: Some(vm_object_alloc_callback),
            ..std::mem::zeroed()
        };

        let err = (**jvmti).SetEventCallbacks.unwrap()(
            jvmti,
            &callbacks,
            std::mem::size_of::<jvmtiEventCallbacks>() as jint,
        );
        if err != jvmtiError_JVMTI_ERROR_NONE {
            eprintln!("Failed to set JVMTI event callbacks: {}", err);
        }

        let events = [
            jvmtiEvent_JVMTI_EVENT_VM_INIT,
            jvmtiEvent_JVMTI_EVENT_VM_DEATH,
            jvmtiEvent_JVMTI_EVENT_METHOD_ENTRY,
            jvmtiEvent_JVMTI_EVENT_METHOD_EXIT,
            jvmtiEvent_JVMTI_EVENT_VM_OBJECT_ALLOC,
        ];

        for &event in &events {
            let err = (**jvmti).SetEventNotificationMode.unwrap()(
                jvmti,
                jvmtiEventMode_JVMTI_ENABLE,
                event,
                ptr::null_mut(),
            );
            if err != jvmtiError_JVMTI_ERROR_NONE {
                eprintln!("Failed to enable event {}: {}", event, err);
            }
        }

        println!("🔗 Agent attached with call graph analysis, waiting for VM_INIT...");
    }
}

#[no_mangle]
pub extern "C" fn Agent_OnLoad(vm: *mut JavaVM, options: *mut c_char, reserved: *mut c_void) {
    Agent_OnAttach(vm, options, reserved);
}
