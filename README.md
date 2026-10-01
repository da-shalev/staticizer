# Staticizer

Staticizer collects typed data from across your Rust crates and lets you turn it into static structures at compile time.

## What it adds beyond linkme and link-section

[linkme](https://docs.rs/linkme/) and [link-section](https://docs.rs/link-section/) gather declarations from multiple crates into a slice at link time. Your program can read that slice at runtime, but const code cannot use it to build another structure.

Staticizer hands the collected data to const code, so you can build a graph, validate dependencies, compute an execution order, or produce any other data structure entirely at compile time.

## Requirements

- **Nightly, or stable with [one setting](USAGE.md#setup).** The toolchain needs the `rustc-dev` component; there is nothing to install. CI tests the latest nightly and stable.
- **Uses one line of unsafe code**, to install its hook in the compiler.
- **Tested on Linux, macOS and Windows** by the [cross-crate tests](tests/cross-crate), in release builds with LTO.

## Usage

Please feel free to open any issue(s) you have using staticizer.

[Setup and code example](USAGE.md)
