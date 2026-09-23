# Staticizer

Staticizer collects data across Rust crates and lets you turn it into the needed static structure.

## What it adds beyond linkme

[linkme](https://docs.rs/linkme/) collects declarations from multiple crates into a slice during linking. Your program can read that slice at runtime, but a Rust const function cannot use the complete slice to construct another structure.

Staticizer lets you process the collected data with const code to build a graph, validate dependencies, calculate an execution order, or produce another data structure—all statically.

**Minimum required nightly: `nightly-2026-08-19`.** Staticizer uses `rustc_private` to access compiler APIs and `linkage` so the final application can replace dependency outputs with the complete result. Build the wrapper and application with the same nightly toolchain. Release builds, ThinLTO, and fat LTO are tested on Linux.

## Usage

Please feel free to open any issue(s) you have using staticizer.

[Setup and code example](USAGE.md)
