# Usage

Register a value with a `Record` impl marked `#[staticizer::register]`, and read every
registered value in const code through `Records::<T>::ITEMS`.

## Setup

Add `staticizer` as a dependency. The toolchain needs the `rustc-dev` component.

The application must contain a `#[staticizer::register]` itself, since
that is what installs Staticizer's hook in its compiler.

On stable, add this to the application's `.cargo/config.toml`. It lets Staticizer's macro
crate, and no other crate, use unstable compiler APIs:

```toml
[env]
RUSTC_BOOTSTRAP = "staticizer_macros"
```

## Example

The library `totals` registers 20 and adds up every registered amount:

```rust
pub struct Amount(pub u32);

struct Base;

#[staticizer::register]
impl staticizer::Record<Amount> for Base {
    const ITEM: &'static Amount = &Amount(20);
}

pub const fn total() -> u32 {
    let amounts = staticizer::Records::<Amount>::ITEMS;
    let mut sum = 0;
    let mut i = 0;
    while i < amounts.len() {
        sum += amounts[i].0;
        i += 1;
    }
    sum
}
```

The application registers 22 and reads the total:

```rust
struct Extra;

#[staticizer::register]
impl staticizer::Record<totals::Amount> for Extra {
    const ITEM: &'static totals::Amount = &totals::Amount(22);
}

const TOTAL: u32 = totals::total();

fn main() {
    println!("Total: {TOTAL}");
}
```

```text
Total: 42
```

# Going further

## Library functions called at runtime miss the application's records

With the first example:

```rust
const TOTAL: u32 = totals::total();

fn main() {
    println!("{TOTAL}"); // 42
    println!("{}", totals::total()); // 20
}
```

The call in `main` runs `total()` as it was compiled inside `totals`, before the
application's 22 existed. `TOTAL` is computed while the application is compiled, so it
includes the 22. To get every record, compute the result in a constant in the application.

## Records that depend on the application

A library can register a value it cannot build on its own, because it depends on a type
the application defines. The record is generic over that type, and is built when the
application reads the records for its own type.

### Why would I want this?

An engine library provides systems, functions that run every frame and read the game's
data. Only the game decides where that data is kept.

A plain record is compiled inside the library, before the game exists, so its system
cannot know where the data is and has to look it up every time it runs.

A record generic over the game's type is compiled by the game itself. By then the game has
decided where its data is, so the system goes straight to it, with no lookup.

### Example

The library `greeter` registers a greeting built from any `App`:

```rust
pub trait App: 'static {
    const NAME: &'static str;
}

pub struct Greeting(pub fn() -> &'static str);

struct Hello;

#[staticizer::register]
impl<A: App> staticizer::Record<Greeting, A> for Hello {
    const ITEM: &'static Greeting = &Greeting(|| A::NAME);
}
```

The application defines its `App`, registers a greeting of its own, and reads the records
for its type:

```rust
struct Game;

impl greeter::App for Game {
    const NAME: &'static str = "Game";
}

#[staticizer::register]
impl staticizer::Record<greeter::Greeting, Game> for Game {
    const ITEM: &'static greeter::Greeting = &greeter::Greeting(|| "the application's own record");
}

fn main() {
    for greeting in staticizer::Records::<greeter::Greeting, Game>::ITEMS {
        println!("Hello from {}", (greeting.0)());
    }
}
```

```text
Hello from the application's own record
Hello from Game
```

## Order

Records come in the same order on every build, but the order means nothing. To order
them, give the record type a field to sort by.
