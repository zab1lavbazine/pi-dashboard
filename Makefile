SHELL := /bin/sh

APP_NAME ?= pi-dashboard
PI_TARGET ?= aarch64-unknown-linux-gnu
PI_HOST ?= bareldan@debil4ik
PI_DIR ?= /home/bareldan/pi-dashboard
PI_DASHBOARD_COMPUTER_CONFIG ?= $(abspath config/computer.yaml)

PI_BINARY := target/$(PI_TARGET)/release/$(APP_NAME)
PI_BINARY_NEW := $(PI_DIR)/$(APP_NAME).new
PI_BINARY_ACTIVE := $(PI_DIR)/$(APP_NAME)

.DEFAULT_GOAL := help

.PHONY: help run run-egui run-tauri build check test clean build-pi build-pi-clean prepare-pi \
	upload-pi upload-config upload-player-config upload-resources upload-all \
	activate-pi deploy-pi

help:
	@echo "Local targets:"
	@echo "  make run                 Run egui (default frontend)"
	@echo "  make run-egui            Run the egui frontend"
	@echo "  make run-tauri           Run Tauri dev (requires tauri-cli v2)"
	@echo "  make build               Build a local release binary"
	@echo "  make check               Format-check, compile, and test"
	@echo "  make clean               Remove Cargo build output"
	@echo ""
	@echo "Raspberry Pi targets:"
	@echo "  make build-pi            Cross-compile the release binary"
	@echo "  make build-pi-clean      Clean, then cross-compile"
	@echo "  make upload-pi           Build and upload as pi-dashboard.new"
	@echo "  make upload-config       Upload media config; preserve player settings"
	@echo "  make upload-player-config  Upload and reset player.yaml explicitly"
	@echo "  make upload-resources    Upload the resources directory"
	@echo "  make upload-all          Upload binary, config, and resources"
	@echo "  make activate-pi         Upload and atomically activate the binary"
	@echo "  make deploy-pi           Upload everything and activate the binary"
	@echo ""
	@echo "Override deployment values when needed:"
	@echo "  make deploy-pi PI_HOST=user@host PI_DIR=/opt/pi-dashboard"

run: run-egui

run-egui:
	cargo run -p pi-dashboard

run-tauri:
	cd apps/tauri && PI_DASHBOARD_COMPUTER_CONFIG="$(PI_DASHBOARD_COMPUTER_CONFIG)" cargo tauri dev

build:
	cargo build --release

check:
	cargo fmt --all -- --check
	cargo check --workspace
	cargo test --workspace

test:
	cargo test --workspace

clean:
	cargo clean

build-pi:
	cargo build --release --target $(PI_TARGET)

build-pi-clean:
	cargo clean
	$(MAKE) build-pi

prepare-pi:
	ssh $(PI_HOST) "mkdir -p '$(PI_DIR)' '$(PI_DIR)/config/media' '$(PI_DIR)/resources'"

upload-pi: build-pi prepare-pi
	scp $(PI_BINARY) $(PI_HOST):$(PI_BINARY_NEW)

upload-config: prepare-pi
	scp -r config/. $(PI_HOST):$(PI_DIR)/config/

upload-player-config: prepare-pi
	scp config/player.yaml $(PI_HOST):$(PI_DIR)/config/player.yaml

upload-resources: prepare-pi
	scp -r resources/. $(PI_HOST):$(PI_DIR)/resources/

upload-all: upload-pi upload-config upload-resources

activate-pi: upload-pi
	ssh $(PI_HOST) "chmod +x '$(PI_BINARY_NEW)' && mv '$(PI_BINARY_NEW)' '$(PI_BINARY_ACTIVE)'"

deploy-pi:
	$(MAKE) upload-config
	$(MAKE) upload-resources
	$(MAKE) activate-pi
