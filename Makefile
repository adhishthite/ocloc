# Makefile for ocloc - A blazingly fast lines-of-code counter
# Run 'make help' to see all available commands

# Variables
BINARY_NAME = ocloc
CARGO = cargo
INSTALL_PATH = ~/.cargo/bin

# Color codes for pretty output
RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[1;33m
BLUE = \033[0;34m
NC = \033[0m # No Color

# Default target
.DEFAULT_GOAL := help

## help: Display this help message
.PHONY: help
help:
	@echo "$(BLUE)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo "$(BLUE)                    ocloc Build System                       $(NC)"
	@echo "$(BLUE)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo ""
	@echo "$(YELLOW)Available targets:$(NC)"
	@echo ""
	@grep -E '^## ' Makefile | sed 's/## /  $(GREEN)make /' | sed 's/: /$(NC) - /'
	@echo ""
	@echo "$(YELLOW)Quick start:$(NC)"
	@echo "  $(GREEN)make build$(NC)    - Build in debug mode"
	@echo "  $(GREEN)make release$(NC)  - Build optimized release"
	@echo "  $(GREEN)make install$(NC)  - Install to system"
	@echo ""

## build: Build debug version
.PHONY: build
build:
	@echo "$(YELLOW)Building debug version...$(NC)"
	@$(CARGO) build
	@echo "$(GREEN)✓ Debug build complete$(NC)"

## release: Build optimized release version
.PHONY: release
release:
	@echo "$(YELLOW)Building release version...$(NC)"
	@$(CARGO) build --release
	@echo "$(GREEN)✓ Release build complete$(NC)"
	@echo "Binary location: target/release/$(BINARY_NAME)"

## install: Install ocloc to system (~/.cargo/bin)
.PHONY: install
install:
	@echo "$(YELLOW)Installing ocloc...$(NC)"
	@$(CARGO) install --path .
	@echo "$(GREEN)✓ Installed to $(INSTALL_PATH)/$(BINARY_NAME)$(NC)"

## uninstall: Remove ocloc from system
.PHONY: uninstall
uninstall:
	@echo "$(YELLOW)Uninstalling ocloc...$(NC)"
	@$(CARGO) uninstall $(BINARY_NAME)
	@echo "$(GREEN)✓ Uninstalled$(NC)"

## run: Run ocloc on current directory (debug mode)
.PHONY: run
run: build
	@echo "$(YELLOW)Running ocloc on current directory...$(NC)"
	@./target/debug/$(BINARY_NAME) .

## run-release: Run ocloc on current directory (release mode)
.PHONY: run-release
run-release: release
	@echo "$(YELLOW)Running ocloc on current directory...$(NC)"
	@./target/release/$(BINARY_NAME) .

## test: Run all tests
.PHONY: test
test:
	@echo "$(YELLOW)Running tests...$(NC)"
	@$(CARGO) test
	@echo "$(GREEN)✓ All tests passed$(NC)"

## test-verbose: Run tests with verbose output
.PHONY: test-verbose
test-verbose:
	@echo "$(YELLOW)Running tests (verbose)...$(NC)"
	@$(CARGO) test -- --nocapture

## bench: Run benchmarks
.PHONY: bench
bench:
	@echo "$(YELLOW)Running benchmarks...$(NC)"
	@$(CARGO) bench

## bench-small: Benchmark on yt-dlp (medium repo)
.PHONY: bench-small
bench-small: release
	@echo "$(YELLOW)Benchmarking on yt-dlp...$(NC)"
	@bash scripts/benchmark-small.sh

## bench-large: Benchmark on elasticsearch (large repo)
.PHONY: bench-large
bench-large: release
	@echo "$(YELLOW)Benchmarking on elasticsearch...$(NC)"
	@bash scripts/benchmark-large.sh

## fmt: Format code using rustfmt
.PHONY: fmt
fmt:
	@echo "$(YELLOW)Formatting code...$(NC)"
	@$(CARGO) fmt
	@echo "$(GREEN)✓ Code formatted$(NC)"

## fmt-check: Check if code is properly formatted
.PHONY: fmt-check
fmt-check:
	@echo "$(YELLOW)Checking code format...$(NC)"
	@$(CARGO) fmt -- --check
	@echo "$(GREEN)✓ Code is properly formatted$(NC)"

## lint: Run clippy linter
.PHONY: lint
lint:
	@echo "$(YELLOW)Running clippy...$(NC)"
	@$(CARGO) clippy -- -D warnings
	@echo "$(GREEN)✓ No linting issues$(NC)"

## fix: Auto-fix linting issues
.PHONY: fix
fix:
	@echo "$(YELLOW)Auto-fixing issues...$(NC)"
	@$(CARGO) fix --allow-dirty --allow-staged
	@$(CARGO) fmt
	@echo "$(GREEN)✓ Issues fixed$(NC)"

## check: Run format check, linter, and tests
.PHONY: check
check: fmt-check lint test
	@echo "$(GREEN)✓ All checks passed$(NC)"

## clean: Remove build artifacts
.PHONY: clean
clean:
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	@$(CARGO) clean
	@rm -f Cargo.lock
	@echo "$(GREEN)✓ Clean complete$(NC)"

## doc: Generate and open documentation
.PHONY: doc
doc:
	@echo "$(YELLOW)Generating documentation...$(NC)"
	@$(CARGO) doc --open

## doc-all: Generate documentation with dependencies
.PHONY: doc-all
doc-all:
	@echo "$(YELLOW)Generating complete documentation...$(NC)"
	@$(CARGO) doc --all --open

## setup-hooks: Install git hooks
.PHONY: setup-hooks
setup-hooks:
	@echo "$(YELLOW)Installing git hooks...$(NC)"
	@bash scripts/install-git-hooks.sh
	@echo "$(GREEN)✓ Git hooks installed$(NC)"

## ci: Run CI pipeline (format, lint, test, build)
.PHONY: ci
ci:
	@echo "$(BLUE)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo "$(BLUE)                    Running CI Pipeline                      $(NC)"
	@echo "$(BLUE)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo ""
	@echo "$(YELLOW)[1/5] Checking format...$(NC)"
	@$(CARGO) fmt -- --check
	@echo "$(GREEN)✓ Format check passed$(NC)\n"

	@echo "$(YELLOW)[2/5] Running clippy...$(NC)"
	@$(CARGO) clippy -- -D warnings
	@echo "$(GREEN)✓ Clippy passed$(NC)\n"

	@echo "$(YELLOW)[3/5] Running tests...$(NC)"
	@$(CARGO) test
	@echo "$(GREEN)✓ Tests passed$(NC)\n"

	@echo "$(YELLOW)[4/5] Building debug...$(NC)"
	@$(CARGO) build
	@echo "$(GREEN)✓ Debug build successful$(NC)\n"

	@echo "$(YELLOW)[5/5] Building release...$(NC)"
	@$(CARGO) build --release
	@echo "$(GREEN)✓ Release build successful$(NC)\n"

	@echo "$(GREEN)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo "$(GREEN)                  ✓ CI Pipeline Complete                     $(NC)"
	@echo "$(GREEN)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"

## dev: Run in development mode with auto-reload (requires cargo-watch)
.PHONY: dev
dev:
	@echo "$(YELLOW)Starting development mode...$(NC)"
	@echo "$(YELLOW)Installing cargo-watch if not present...$(NC)"
	@cargo install cargo-watch 2>/dev/null || true
	@echo "$(YELLOW)Watching for changes...$(NC)"
	@cargo watch -x 'run -- .'

## compare: Compare performance with cloc on current directory
.PHONY: compare
compare: release
	@echo "$(BLUE)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo "$(BLUE)                Performance Comparison: ocloc vs cloc        $(NC)"
	@echo "$(BLUE)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo ""
	@echo "$(YELLOW)Running cloc...$(NC)"
	@time -p cloc . --quiet 2>/dev/null || echo "$(RED)cloc not installed. Install with: brew install cloc$(NC)"
	@echo ""
	@echo "$(YELLOW)Running ocloc...$(NC)"
	@time -p ./target/release/$(BINARY_NAME) . 2>&1 | head -n 50
	@echo ""
	@echo "$(GREEN)Compare the 'real' time values above to see the speedup!$(NC)"

## version: Display version information
.PHONY: version
version:
	@echo "$(YELLOW)ocloc version:$(NC)"
	@$(CARGO) pkgid | cut -d '#' -f 2
	@echo ""
	@echo "$(YELLOW)Rust version:$(NC)"
	@rustc --version
	@echo ""
	@echo "$(YELLOW)Cargo version:$(NC)"
	@$(CARGO) --version

## update: Update dependencies
.PHONY: update
update:
	@echo "$(YELLOW)Updating dependencies...$(NC)"
	@$(CARGO) update
	@echo "$(GREEN)✓ Dependencies updated$(NC)"

## audit: Check for security vulnerabilities
.PHONY: audit
audit:
	@echo "$(YELLOW)Checking for security vulnerabilities...$(NC)"
	@cargo install cargo-audit 2>/dev/null || true
	@cargo audit
	@echo "$(GREEN)✓ Security audit complete$(NC)"

## size: Show binary size information
.PHONY: size
size: release
	@echo "$(YELLOW)Binary size information:$(NC)"
	@ls -lh target/release/$(BINARY_NAME)
	@echo ""
	@echo "$(YELLOW)Size breakdown:$(NC)"
	@size target/release/$(BINARY_NAME) 2>/dev/null || \
		(du -h target/release/$(BINARY_NAME) && \
		 echo "$(YELLOW)Install 'binutils' for detailed size breakdown$(NC)")

## all: Build everything (debug + release)
.PHONY: all
all: clean build release
	@echo "$(GREEN)✓ Complete build finished$(NC)"

# Catch-all target for invalid commands
%:
	@echo "$(RED)Error: Unknown target '$@'$(NC)"
	@echo "Run '$(GREEN)make help$(NC)' to see available targets"
	@exit 1
