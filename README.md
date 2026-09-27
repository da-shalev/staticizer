# Staticizer

Staticizer collects typed data from across your Rust crates and lets you turn it into static structures at compile time.

## What it adds beyond linkme

[linkme](https://docs.rs/linkme/) gathers declarations from multiple crates into a slice at link time. Your program can read that slice at runtime, but const code cannot use it to build another structure.

Staticizer hands the collected data to const code, so you can build a graph, validate dependencies, compute an execution order, or produce any other data structure entirely at compile time.

## Requirements

- **Nightly only.** Staticizer hooks into rustc through unstable compiler APIs. It builds the hook with your project's nightly the first time it sees that nightly, so the wrapper never has to match your toolchain. The toolchain needs the `rustc-dev` and `llvm-tools` components. CI tests the nightly pinned in `flake.nix` and the latest nightly.
- **Uses a small amount of unsafe code.**
- **Tested on Linux, macOS and Windows** by the [cross-crate tests](tests/cross-crate). Release builds and LTO are also tested on Linux.

## Usage

Please feel free to open any issue(s) you have using staticizer.

[Setup and code example](USAGE.md)
