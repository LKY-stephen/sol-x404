#!/bin/bash

cargo build-bpf
cd tests
cargo test-sbf