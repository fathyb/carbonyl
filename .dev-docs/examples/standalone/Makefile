# Makefile for Mongoose FFI Standalone Test
#
# This builds a simple C program that tests the Mongoose FFI bridge
# without requiring Chromium.

# Paths
ROOT_DIR := ../../..
TARGET_DIR := $(ROOT_DIR)/target/release
FFI_HEADER := $(ROOT_DIR)/libmongoose/ffi_exports.h

# Compiler settings
CC := gcc
CFLAGS := -Wall -Wextra -I$(ROOT_DIR)
LDFLAGS := -L$(TARGET_DIR) -lmongoose -lpthread -ldl -lm

# Targets
.PHONY: all clean run build-lib

all: test_ffi

# Build the Rust library first
build-lib:
	@echo "Building Rust library..."
	cd $(ROOT_DIR) && cargo build --lib --release

# Build the test program
test_ffi: test_ffi.c $(FFI_HEADER) build-lib
	@echo "Building standalone test..."
	$(CC) $(CFLAGS) -o test_ffi test_ffi.c $(LDFLAGS)
	@echo "✅ Build complete: ./test_ffi"

# Run the test
run: test_ffi
	@echo "Running FFI tests..."
	@echo ""
	LD_LIBRARY_PATH=$(TARGET_DIR) ./test_ffi

# Clean build artifacts
clean:
	rm -f test_ffi
	@echo "✅ Cleaned"

# Help
help:
	@echo "Mongoose FFI Standalone Test - Build Instructions"
	@echo ""
	@echo "Available targets:"
	@echo "  make all        - Build the test program (default)"
	@echo "  make run        - Build and run the tests"
	@echo "  make clean      - Remove build artifacts"
	@echo "  make build-lib  - Build only the Rust library"
	@echo ""
	@echo "Example:"
	@echo "  make run"
