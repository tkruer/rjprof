#!/bin/bash
# Documentation generation script for rjprof

set -e

echo "🦀 Generating Rust documentation for rjprof..."

# Clean previous docs
echo "🧹 Cleaning previous documentation..."
cargo clean --doc

# Generate documentation with all features
echo "📚 Building documentation..."
RUSTDOCFLAGS="--cfg docsrs" cargo doc \
    --workspace \
    --all-features \
    --no-deps \
    --document-private-items

echo "✅ Documentation generated successfully!"
echo "📖 Open documentation: file://$(pwd)/target/doc/rjprof/index.html"

# Optionally open in browser (macOS/Linux)
if command -v open &> /dev/null; then
    echo "🌐 Opening documentation in browser..."
    open "target/doc/rjprof/index.html"
elif command -v xdg-open &> /dev/null; then
    echo "🌐 Opening documentation in browser..."
    xdg-open "target/doc/rjprof/index.html"
fi