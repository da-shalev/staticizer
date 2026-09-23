# Usage

Register input values, implement `Build` for the result type, and read the result
with `staticizer::output::<T>()`. Output discovery is automatic, including calls
inside dependencies. The compiler builds each output from the application's
registrations and those of its dependencies.

## Setup

Install the compiler wrapper from the Staticizer checkout:

```sh
cargo +nightly install --path . --locked --force
```

Add this to the application's `.cargo/config.toml`:

```toml
[build]
rustc-wrapper = "staticizer"
```

The wrapper must be on `PATH`. Use the same nightly for Staticizer and the
application, with `rustc-dev` and `llvm-tools-preview` installed. Inside
`nix develop`, omit `+nightly`.

After updating the wrapper, run `cargo clean` in the application to rebuild dependency metadata; Cargo does not track changes to the wrapper executable.

## Example: add values declared in two crates

Both crates depend on `staticizer`. The application also depends on a library named `totals`.

**`totals/src/lib.rs`** declares 20 and defines how to add all registered amounts during compilation:

```rust
pub struct Amount(pub u32);
pub struct Total(pub u32);

#[staticizer::register("amounts")]
static BASE: Amount = Amount(20);

impl staticizer::Build for Total {
    const VALUE: &'static Self = &{
        let amounts = staticizer::Records::<Amount>::ITEMS;
        let mut sum = 0;
        let mut i = 0;
        while i < amounts.len() {
            sum += amounts[i].0;
            i += 1;
        }
        Total(sum)
    };
}

pub fn total() -> u32 {
    staticizer::output::<Total>().0
}
```

**The application's `src/main.rs`** declares another 22 and reads the result:

```rust
#[staticizer::register("amounts")]
static EXTRA: totals::Amount = totals::Amount(22);

fn main() {
    println!("Total: {}", totals::total());
}
```

Run `cargo +nightly run`. Output:

```text
Total: 42
```

The compiler adds the library's 20 and the application's 22. At runtime,
`totals::total()` reads the finished result.

Change `Amount(22)` to `Amount(5)` and rebuild:

```text
Total: 25
```

Use the same pattern to construct a graph or schedule: collect typed declarations
with `Records::<T>::ITEMS`, transform them in `Build::VALUE`, and read the finished
structure through `output::<T>()`.
