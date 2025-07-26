#!/bin/bash

# rjprof.sh - Easy wrapper script for rjprof Java profiler
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RJPROF_BIN="$SCRIPT_DIR/target/release/rjprof"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored message
print_msg() {
    echo -e "${2}${1}${NC}"
}

# Print usage information
usage() {
    cat << EOF
🚀 rjprof - Java Performance Profiler

USAGE:
    $0 <command> [options]

COMMANDS:
    build               Build the profiler
    profile <jar>       Profile a JAR file
    spring <jar>        Profile Spring Boot application
    demo               Run demo with Hello World
    spring-demo        Run Spring Boot demo
    flamegraph         Generate flamegraph from last run
    clean              Clean build artifacts
    help               Show this help

EXAMPLES:
    $0 build                          # Build the profiler
    $0 profile myapp.jar              # Basic profiling
    $0 spring myspring.jar            # Spring-optimized profiling
    $0 profile myapp.jar --verbose    # Verbose output
    $0 demo                           # Quick demo

PROFILE OPTIONS:
    --spring           Enable Spring filtering
    --mode <mode>      Profile mode: all, user, hotspots, allocation
    --sort <field>     Sort by: total, self, calls, name, percentage
    --min-pct <pct>    Hide methods below percentage threshold
    --min-total <time> Hide methods below time threshold (e.g., 1ms)
    --export <format>  Export format: json, csv
    --verbose          Verbose output
    --no-color         Disable colorized output
    --generate-flamegraph  Generate SVG flamegraph

For more options, run: $RJPROF_BIN --help
EOF
}

# Check if rjprof binary exists
check_binary() {
    if [ ! -f "$RJPROF_BIN" ]; then
        print_msg "❌ rjprof binary not found. Building..." $YELLOW
        build_profiler
    fi
}

# Build the profiler
build_profiler() {
    print_msg "🔨 Building rjprof..." $BLUE
    cd "$SCRIPT_DIR"
    RUSTFLAGS="-Awarnings" cargo build --release --workspace
    print_msg "✅ Build complete!" $GREEN
}

# Profile a JAR file with basic settings
profile_jar() {
    local jar_file="$1"
    shift
    
    if [ ! -f "$jar_file" ]; then
        print_msg "❌ JAR file not found: $jar_file" $RED
        exit 1
    fi
    
    check_binary
    print_msg "🔍 Profiling $jar_file..." $BLUE
    "$RJPROF_BIN" --jar "$jar_file" --mode user --sort self "$@"
}

# Profile Spring Boot application with optimized settings
profile_spring() {
    local jar_file="$1"
    shift
    
    if [ ! -f "$jar_file" ]; then
        print_msg "❌ JAR file not found: $jar_file" $RED
        exit 1
    fi
    
    check_binary
    print_msg "🌱 Profiling Spring Boot application: $jar_file..." $BLUE
    "$RJPROF_BIN" --jar "$jar_file" --spring --mode user --sort self --min-pct 0.1 "$@"
}

# Run Hello World demo
run_demo() {
    check_binary
    print_msg "🚀 Running Hello World demo..." $BLUE
    
    if [ ! -f "$SCRIPT_DIR/examples/HelloApp.jar" ]; then
        print_msg "❌ HelloApp.jar not found. Please build examples first." $RED
        exit 1
    fi
    
    "$RJPROF_BIN" --jar "$SCRIPT_DIR/examples/HelloApp.jar" --mode user --sort self --verbose
}

# Run Spring Boot demo
run_spring_demo() {
    check_binary
    print_msg "🌱 Building and running Spring Boot demo..." $BLUE
    
    # Build Spring demo if needed
    if [ ! -f "$SCRIPT_DIR/examples/spring-demo/target/spring-demo-1.0.0.jar" ]; then
        print_msg "☕ Building Spring demo..." $YELLOW
        cd "$SCRIPT_DIR/examples/spring-demo"
        mvn clean package -q
        cd "$SCRIPT_DIR"
    fi
    
    "$RJPROF_BIN" \
        --jar "$SCRIPT_DIR/examples/spring-demo/target/spring-demo-1.0.0.jar" \
        --spring \
        --mode user \
        --sort self \
        --min-pct 0.1 \
        --verbose
}

# Generate flamegraph from last profiling run
generate_flamegraph() {
    print_msg "🔥 Generating flamegraph..." $BLUE
    
    if [ ! -f "$SCRIPT_DIR/profiler_output/flamegraph.folded" ]; then
        print_msg "❌ No profiling data found. Run a profiling session first." $RED
        exit 1
    fi
    
    cd "$SCRIPT_DIR/profiler_output"
    
    if command -v flamegraph.pl &> /dev/null; then
        flamegraph.pl flamegraph.folded > flamegraph.svg
        print_msg "✅ Flamegraph generated: profiler_output/flamegraph.svg" $GREEN
    elif command -v inferno-flamegraph &> /dev/null; then
        inferno-flamegraph flamegraph.folded > flamegraph.svg
        print_msg "✅ Flamegraph generated: profiler_output/flamegraph.svg" $GREEN
    else
        print_msg "❌ No flamegraph generator found. Install flamegraph.pl or inferno-flamegraph" $RED
        exit 1
    fi
}

# Clean build artifacts
clean_artifacts() {
    print_msg "🧹 Cleaning build artifacts..." $BLUE
    cd "$SCRIPT_DIR"
    cargo clean
    rm -rf bin/
    print_msg "✅ Clean complete!" $GREEN
}

# Main command dispatcher
main() {
    if [ $# -eq 0 ]; then
        usage
        exit 1
    fi
    
    case "$1" in
        build)
            build_profiler
            ;;
        profile)
            if [ $# -lt 2 ]; then
                print_msg "❌ Usage: $0 profile <jar-file> [options]" $RED
                exit 1
            fi
            profile_jar "${@:2}"
            ;;
        spring)
            if [ $# -lt 2 ]; then
                print_msg "❌ Usage: $0 spring <jar-file> [options]" $RED
                exit 1
            fi
            profile_spring "${@:2}"
            ;;
        demo)
            run_demo
            ;;
        spring-demo)
            run_spring_demo
            ;;
        flamegraph)
            generate_flamegraph
            ;;
        clean)
            clean_artifacts
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            print_msg "❌ Unknown command: $1" $RED
            echo
            usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"