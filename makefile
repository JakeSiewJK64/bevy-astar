BINARY_NAME := your_binary_name
TARGET := wasm32-unknown-unknown
WASM_DIR := wasm
ZIP_NAME := $(BINARY_NAME).zip

.PHONY:
	all setup build package clean

all: 
	setup build package

setup:
	rustup target add $(TARGET)
	cargo install -f wasm-bindgen-cli

build:
	cargo build --release --target $(TARGET)

package:
	mkdir -p $(WASM_DIR)
	wasm-bindgen --no-typescript --out-name bevy_game --out-dir $(WASM_DIR) --target web target/$(TARGET)/release/$(BINARY_NAME).wasm
	cp -r assets $(WASM_DIR)/ || true
	cd $(WASM_DIR) && zip --recurse-paths ../$(ZIP_NAME) .

clean:
	rm -rf $(WASM_DIR)
	rm -f $(ZIP_NAME)
