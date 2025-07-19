# rjprof

A high-performance Java profiler, similar to [async-profiler](https://github.com/async-profiler/async-profiler).

## 🚀 Quick Start

```bash
# Build everything
make build

# Quick test with Hello World
make run-hello

# Test with Spring Boot application (builds and runs automatically)
make run-spring

# Profile your own JAR
make profile JAR=myapp.jar ARGS="--spring --mode user"
```

## 📁 Project Structure

This project uses a Cargo workspace with two main components:

- **`librjprof/`** - Core profiling library using JVMTI
- **`rjprof-cli/`** - Command-line interface and tools
- **`examples/`** - Sample applications for testing

## 🛠️ Building

### Simple Build (Recommended)
```bash
make build      # Release build
make dev        # Debug build (faster)
make install    # Build and copy binaries to bin/
```

### Manual Build
```bash
cargo build --release --workspace
```

### Requirements
- **Rust** (latest stable)
- **Java Development Kit** (JDK 8+) with `JAVA_HOME` set
- **Maven** (for Spring demo)
- Optional: **flamegraph.pl** or **inferno-flamegraph** for SVG generation

## 🔧 Usage

### Basic Profiling
```bash
# Profile any JAR file
./target/release/rjprof --jar myapp.jar

# Profile with Spring filtering (removes framework noise)
./target/release/rjprof --jar myapp.jar --spring

# Focus on user code only
./target/release/rjprof --jar myapp.jar --mode user

# Filter by performance thresholds
./target/release/rjprof --jar myapp.jar --min-pct 0.5 --min-total 1ms
```

### Advanced Options
```bash
# Custom package filtering
./target/release/rjprof --jar myapp.jar --exclude "java.*,org.apache.*" --include "com.mycompany.*"

# Different sorting and output options
./target/release/rjprof --jar myapp.jar --sort total --export json --no-color

# Verbose output with flamegraph generation
./target/release/rjprof --jar myapp.jar --verbose --generate-flamegraph
```

### Profile Modes
- **`all`** - Profile everything (can be noisy)
- **`user`** - Focus on user code, exclude JDK/framework *(default)*
- **`hotspots`** - Only methods above performance thresholds
- **`allocation`** - Focus on allocation-heavy methods

### Makefile Targets

| Target | Description |
|--------|-------------|
| `make build` | Build release version |
| `make dev` | Build debug version (faster) |
| `make test` | Run all tests |
| `make run-hello` | Quick test with Hello World |
| `make run-spring` | Test with Spring Boot demo |
| `make flamegraph` | Generate SVG from last run |
| `make clean` | Clean build artifacts |
| `make help` | Show all available targets |

## 📊 Features

### Smart Filtering
- **Spring Boot optimized** - Automatically filters common Spring/framework noise
- **Custom package filtering** - Include/exclude specific packages
- **Performance thresholds** - Hide methods below time/percentage limits
- **Profile modes** - Focus on different aspects of performance

### Output Formats
- **Colorized console output** - Easy-to-read performance reports
- **JSON export** - Machine-readable data
- **CSV export** - Spreadsheet-compatible format
- **Flamegraph generation** - Visual performance analysis

### Advanced Profiling
- **Method entry/exit timing** - Precise performance measurement
- **Memory allocation tracking** - Find allocation hotspots
- **Call graph analysis** - Understand call relationships
- **Sampling support** - Low-overhead profiling option

## 📋 Examples

### Profile Spring Boot Application
```bash
# Automatic: builds Spring demo and profiles it
make run-spring

# Manual: profile existing Spring JAR
./target/release/rjprof \
  --jar myspring.jar \
  --spring \
  --mode user \
  --min-pct 0.1 \
  --sort self \
  --generate-flamegraph
```

### Custom Filtering for Microservice
```bash
./target/release/rjprof \
  --jar microservice.jar \
  --exclude "java.*,javax.*,org.springframework.*,org.apache.*" \
  --include "com.mycompany.*" \
  --mode hotspots \
  --min-total 5ms \
  --export json
```

### Memory Allocation Analysis
```bash
./target/release/rjprof \
  --jar myapp.jar \
  --mode allocation \
  --sort calls \
  --min-self-time 100us
```

## 🔥 Flamegraph Generation

Generate visual flamegraphs for performance analysis:

```bash
# Run profiler with flamegraph generation
./target/release/rjprof --jar myapp.jar --generate-flamegraph

# Or generate from existing data
make flamegraph
```

Flamegraphs are saved to `profiler_output/flamegraph.svg`.

## 🧪 Development

### Running Tests
```bash
make test                    # Run all tests
cargo test --workspace      # Manual test execution
```

### Code Quality
```bash
make check                   # Check code without building
cargo clippy --workspace    # Run linter
cargo fmt --workspace       # Format code
```

## 📈 Performance Tips

1. **Use `--mode user`** for most applications to focus on your code
2. **Enable `--spring`** for Spring Boot apps to reduce noise
3. **Set thresholds** with `--min-pct` and `--min-total` to hide trivial methods
4. **Generate flamegraphs** for visual analysis of performance bottlenecks
5. **Export to JSON** for integration with other analysis tools

## 🔍 Troubleshooting

### Agent Not Found
```bash
# Manually specify agent path
./target/release/rjprof --jar myapp.jar --agent-path ./target/release/librjprof.dylib
```

### JAVA_HOME Issues
```bash
# Ensure JAVA_HOME is set correctly
export JAVA_HOME=$(/usr/libexec/java_home)  # macOS
export JAVA_HOME=/usr/lib/jvm/default       # Linux
```

### Build Issues
```bash
# Clean and rebuild
make clean
make build

# Check dependencies
cargo check --workspace
```

## 📄 License

Licensed under either of:
- Apache License, Version 2.0
- MIT License

---

![Flamegraph Example](examples/flamegraph.svg)
