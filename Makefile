# Makefile for rjprof
.PHONY: all build clean test run-hello run-spring help install dev check

# Default target
all: build

# Build the entire workspace
build:
	@echo "🔨 Building rjprof workspace..."
	RUSTFLAGS="-Awarnings" cargo build --release --workspace
	@echo "✅ Build complete!"

# Build and copy binaries to convenient locations
install: build
	@echo "📦 Installing binaries..."
	mkdir -p bin
	cp target/release/rjprof bin/
	cp target/release/librjprof.dylib bin/ 2>/dev/null || cp target/release/librjprof.so bin/ 2>/dev/null || cp target/release/rjprof.dll bin/ 2>/dev/null || true
	@echo "✅ Binaries installed to bin/ directory"

# Development build (faster, with debug info)
dev:
	@echo "🔧 Development build..."
	cargo build --workspace
	@echo "✅ Dev build complete!"

# Check code without building
check:
	@echo "🔍 Checking code..."
	cargo check --workspace
	@echo "✅ Code check complete!"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf bin/
	@echo "✅ Clean complete!"

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test --workspace
	@echo "✅ Tests complete!"

# Quick test with Hello World example
run-hello: build
	@echo "🚀 Running Hello World example..."
	./target/release/rjprof \
		--jar examples/HelloApp.jar \
		--mode user \
		--sort self \
		--verbose

# Quick test with Spring demo
run-spring: build build-spring
	@echo "🌱 Running Spring demo with profiling..."
	./target/release/rjprof \
		--jar examples/spring-demo/target/spring-demo-1.0.0.jar \
		--spring \
		--mode user \
		--sort self \
		--min-pct 0.1 \
		--verbose

# Build Spring demo JAR
build-spring:
	@echo "☕ Building Spring demo..."
	cd examples/spring-demo && mvn clean package -q
	@echo "✅ Spring demo built!"

# Build test JARs
build-examples:
	@echo "☕ Building example JARs..."
	@if [ -d "examples/HelloWorld" ]; then \
		echo "Building HelloWorld..."; \
		cd examples/HelloWorld && javac src/main/java/com/helloworld/Main.java && \
		jar cfm ../HelloApp.jar ../manifest.txt -C src/main/java .; \
	fi
	@echo "✅ Examples built!"

# Generate flamegraph from last run
flamegraph:
	@echo "🔥 Generating flamegraph..."
	@if [ -f "profiler_output/flamegraph.folded" ]; then \
		cd profiler_output && \
		(flamegraph.pl flamegraph.folded > flamegraph.svg 2>/dev/null || \
		 inferno-flamegraph flamegraph.folded > flamegraph.svg 2>/dev/null || \
		 echo "❌ No flamegraph generator found. Install flamegraph.pl or inferno-flamegraph"); \
		if [ -f "flamegraph.svg" ]; then echo "✅ Flamegraph generated: profiler_output/flamegraph.svg"; fi; \
	else \
		echo "❌ No profiling data found. Run a profiling session first."; \
	fi

# Profile a custom JAR file
profile:
	@if [ -z "$(JAR)" ]; then \
		echo "❌ Usage: make profile JAR=path/to/your.jar [ARGS='--spring --mode user']"; \
		exit 1; \
	fi
	@echo "🔍 Profiling $(JAR)..."
	./target/release/rjprof --jar $(JAR) $(ARGS) --verbose

# Show help
help:
	@echo "📖 rjprof Makefile Help"
	@echo ""
	@echo "Build targets:"
	@echo "  build         - Build release version of all crates"
	@echo "  dev           - Build debug version (faster)"
	@echo "  install       - Build and copy binaries to bin/"
	@echo "  check         - Check code without building"
	@echo "  clean         - Clean all build artifacts"
	@echo ""
	@echo "Test targets:"
	@echo "  test          - Run Rust tests"
	@echo "  run-hello     - Quick test with HelloWorld example"
	@echo "  run-spring    - Test with Spring demo application"
	@echo ""
	@echo "Utility targets:"
	@echo "  build-spring  - Build Spring demo JAR"
	@echo "  build-examples- Build example JARs"
	@echo "  flamegraph    - Generate SVG flamegraph from last run"
	@echo "  profile       - Profile custom JAR (Usage: make profile JAR=app.jar ARGS='--spring')"
	@echo "  help          - Show this help"
	@echo ""
	@echo "Examples:"
	@echo "  make build                                    # Build everything"
	@echo "  make run-hello                               # Quick test"
	@echo "  make profile JAR=myapp.jar ARGS='--spring'   # Profile custom app"
	@echo "  make flamegraph                              # Generate flamegraph"

# Legacy targets for backwards compatibility
rust-build: build
jar: build-examples