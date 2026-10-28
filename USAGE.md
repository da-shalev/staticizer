# Usage

Any crate can register values, called *records*. All records of one type form a list. Your
main crate, the one with `main.rs`, declares a type, `MainApp`, and every list is read for
`MainApp`.

`MainApp` is what makes every list complete. Rust compiles every library before your main
crate, so a library on its own sees only its own records. To fix this, a library writes its
code generic over a type `A`, and your main crate fills `A` in with `MainApp`. That code is
then compiled inside your main crate, where every crate's records are visible.

## Setup

1. Add `staticizer` as a dependency.
2. Install your toolchain's `rustc-dev` component.
3. Put at least one `#[staticizer::register]` in your main crate. Without it, reading records
   fails to compile.

On stable Rust, also add this to your main crate's `.cargo/config.toml`:

```toml
[env]
RUSTC_BOOTSTRAP = "staticizer_macros"
```

## Example

The library `greeter` needs the main crate's name, so it declares a trait `App` with a
`NAME`. It registers a greeting that says that name, and has a function that prints every
greeting:

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

pub fn print_greetings<A: App>(_: A) {
    for greeting in staticizer::Records::<Greeting, A>::ITEMS {
        println!("Hello from {}", (greeting.0)());
    }
}
```

`Records::<Greeting, A>::ITEMS` is every `Greeting` record for `A`. `print_greetings` takes
the main crate's type as a value, so callers do not have to name it.

Your main crate declares `MainApp`, implements `greeter::App` for it, registers a greeting of
its own, and passes `MainApp` in:

```rust
struct MainApp;

impl greeter::App for MainApp {
    const NAME: &'static str = "MainApp";
}

struct Own;

#[staticizer::register]
impl staticizer::Record<greeter::Greeting, MainApp> for Own {
    const ITEM: &'static greeter::Greeting = &greeter::Greeting(|| "the main crate's own record");
}

fn main() {
    greeter::print_greetings(MainApp);
}
```

```text
Hello from MainApp
Hello from the main crate's own record
```

`print_greetings::<MainApp>` is compiled inside your main crate, so it prints both records:
`greeter`'s, which took its name from `MainApp`, and the main crate's own.

That is the whole pattern. A library declares a trait for the main crate's type, and makes
its records and reads generic over it. The trait holds whatever the library needs from the
main crate. A library that needs nothing declares it empty.

> [!CAUTION]
> **Every read must use `MainApp`**, directly or through a generic `A: App`. A read without
> it, such as `Records::<Greeting>::ITEMS`, is compiled inside the library. It misses records,
> gives no error, and can even differ between debug and release builds.

## Order

Debug and release builds can order the list differently, so do not rely on the order. To get
a fixed order, add a field to `Greeting` and sort by it.

## Find one record

`Position` gives the index of one record in the list, so you can read it without searching.
Name the record by the three types in its impl. For the main crate's
`impl Record<greeter::Greeting, MainApp> for Own`, that is
`Position::<greeter::Greeting, MainApp, Own>::OF`:

```rust
const OWN: usize = staticizer::Position::<greeter::Greeting, MainApp, Own>::OF;

fn main() {
    println!("{}", (staticizer::Records::<greeter::Greeting, MainApp>::ITEMS[OWN].0)());
}
```

```text
the main crate's own record
```

`OWN` is a constant, so the read is a plain array index.

A generic function that takes the type the impl is for, like `Own`, needs a `Record` bound on
it. Without the bound, it fails to compile. In `greeter`:

```rust
pub fn greeting_of<A: App, Entry: staticizer::Record<Greeting, A> + 'static>() -> &'static str {
    (staticizer::Records::<Greeting, A>::ITEMS[staticizer::Position::<Greeting, A, Entry>::OF].0)()
}
```

`greeter::greeting_of::<MainApp, Own>()` returns `"the main crate's own record"`.

A record whose impl is for a generic type, like
`impl<X> Record<greeter::Greeting, MainApp> for Wrapper<X>`, is not in the list, so it has no
position.
