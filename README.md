# Staticizer

Staticizer collects typed data from across your Rust crates and lets you turn it into static structures at compile time.

## What it adds beyond linkme and link-section

[linkme](https://docs.rs/linkme/) and [link-section](https://docs.rs/link-section/) gather declarations from multiple crates into a slice at link time. Your program can read that slice at runtime, but const code cannot use it to build another structure.

Staticizer hands the collected data to const code, so you can build a graph, validate dependencies, compute an execution order, or produce any other data structure entirely at compile time.

## Why not build it at startup instead

- **Fewer points of failure at runtime:** Nothing is built at startup, so nothing can fail there, and mistakes your const code finds, such as a dependency cycle, fail the build instead.
- **Simpler code:** There is no setup code, lazy initialization or startup error handling to write. For the const code itself, [konst](https://docs.rs/konst) provides loops and iterator-style helpers.
- **Can remove lookups:** What you build is a constant, so code can use it directly instead of looking it up at runtime.
- **No heap and no setup:** The structures are constants in your binary; nothing is allocated or built when the program starts.
- **Library code built for your application:** A library can register code that is compiled for your application's types, which nothing built at runtime can do.

## Requirements

- Nightly, or stable with [one setting](USAGE.md#setup), plus the `rustc-dev` component. Nothing else to install.
- Your code never touches compiler APIs. The hook, one line of unsafe code, is entirely inside Staticizer.
- Works on the latest nightly and stable, on Linux, macOS and Windows, in release builds with LTO.
- CI tests it against the latest nightly every day.
- Incremental builds pick up records added, changed or removed in dependencies.
- rust-analyzer reports no errors in code that uses it.

## Usage

Please feel free to open any issue(s) you have using staticizer.

[Setup and code example](USAGE.md)

## License

Licensed under the [MIT license](LICENSE).
