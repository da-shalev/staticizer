# Staticizer

Staticizer collects typed data from across your Rust crates and lets you turn it into static structures at compile time.

## What it adds beyond linkme

[linkme](https://docs.rs/linkme/) gathers declarations from multiple crates into a slice at link time. Your program can read that slice at runtime, but const code cannot use it to build another structure.

Staticizer hands the collected data to const code, so you can build a graph, validate dependencies, compute an execution order, or produce any other data structure entirely at compile time.

## Requirements

- **Nightly, or stable with one setting.** Staticizer's `#[register]` macro hooks into rustc through unstable compiler APIs; on stable, [one setting](USAGE.md#setup) allows them. Cargo builds the macro with your project's toolchain like any other dependency, so there is nothing to install. The toolchain needs the `rustc-dev` component. CI tests the latest nightly and the latest stable.
- **Uses a small amount of unsafe code.**
- **Tested on Linux, macOS and Windows** by the [cross-crate tests](tests/cross-crate). Release builds and LTO are also tested on Linux.

## Usage

Please feel free to open any issue(s) you have using staticizer.

[Setup and code example](USAGE.md)
