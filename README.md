Miniserde
=========

[<img alt="github" src="https://img.shields.io/badge/github-dtolnay/miniserde-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dtolnay/miniserde)
[<img alt="crates.io" src="https://img.shields.io/crates/v/miniserde.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/miniserde)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-miniserde-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/miniserde)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dtolnay/miniserde/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/dtolnay/miniserde/actions?query=branch%3Amaster)

*Prototype of a data structure serialization library with several opposite
design goals from [Serde](https://serde.rs).*

As a prototype, this library is not a production quality engineering artifact
the way Serde is. At the same time, it is more than a proof of concept and
should be totally usable for the range of use cases that it targets, which is
qualified below.

```toml
[dependencies]
miniserde = "0.1"
```

Version requirement: rustc 1.61+

### Example

```rust
use miniserde::{json, Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Example {
    code: u32,
    message: String,
}

fn main() -> miniserde::Result<()> {
    let example = Example {
        code: 200,
        message: "reminiscent of Serde".to_owned(),
    };

    let j = json::to_string(&example);
    println!("{}", j);

    let out: Example = json::from_str(&j)?;
    println!("{:?}", out);

    Ok(())
}
```

Here are some similarities and differences compared to Serde.

### Similar: Stupidly good performance

Seriously this library is way faster than it deserves to be. With very little
profiling and optimization so far and opportunities for improvement, this
library is on par with serde\_json for some use cases, slower by a factor of 1.5
for most, and slower by a factor of 2 for some. That is remarkable considering
the other advantages below.

### Similar: Strongly typed data

Just like Serde, we provide a derive macro for a Serialize and Deserialize
trait. You derive these traits on your own data structures and use
`json::to_string` to convert any Serialize type to JSON and `json::from_str` to
parse JSON into any Deserialize type. Like serde\_json there is a `Value` enum
for embedding untyped components.

### Different: Minimal design

This library does not tackle as expansive of a range of use cases as Serde does.
Feature requests are practically guaranteed to be rejected. If your use case is
not already covered, please use Serde.

The implementation is less code by a factor of 12 compared to serde +
serde\_derive + serde\_json, and less code even than the `json` crate which
provides no derive macro and cannot manipulate strongly typed data.

### Different: No monomorphization

There are no nontrivial generic methods. All serialization and deserialization
happens in terms of trait objects. Thus no code is compiled more than once
across different generic parameters. In contrast, serde\_json needs to stamp out
a fair amount of generic code for each choice of data structure being serialized
or deserialized.

Without monomorphization, the derived impls compile lightning fast and occupy
very little size in the executable.

### Different: No recursion

Serde depends on recursion for serialization as well as deserialization. Every
level of nesting in your data means more stack usage until eventually you
overflow the stack. Some formats set a cap on nesting depth to prevent stack
overflows and just refuse to deserialize deeply nested data.

In miniserde neither serialization nor deserialization involves recursion. You
can safely process arbitrarily nested data without being exposed to stack
overflows. Not even the Drop impl of our json `Value` type is recursive so you
can safely nest them arbitrarily.

### Different: No deserialization error messages

When deserialization fails, the error type is a unit struct containing no
information. This is a legit strategy and not just laziness. If your use case
does not require error messages, good, you save on compiling and having your
instruction cache polluted by error handling code. If you do need error
messages, then upon error you can pass the same input to serde\_json to receive
a line, column, and helpful description of the failure. This keeps error
handling logic out of caches along the performance-critical codepath.

### Different: Infallible serialization

Serialization always succeeds. This means we cannot serialize some data types
that Serde can serialize, such as `Mutex` which may fail to serialize due to
poisoning. Also we only serialize to `String`, not to something like an i/o
stream which may be fallible.

### Different: JSON only

The same approach in this library could be made to work for other data formats,
but it is not a goal to enable that through what this library exposes.

### Different: Structs and unit variants only

The miniserde derive macros will refuse anything other than a braced struct with
named fields or an enum with C-style variants. Tuple structs are not supported,
and enums with data in their variants are not supported.

### Different: No customization

Serde has tons of knobs for configuring the derived serialization and
deserialization logic through attributes. Or for the ultimate level of
configurability you can handwrite arbitrarily complicated implementations of its
traits.

Miniserde provides just one attribute which is `rename`, and severely restricts
the kinds of on-the-fly manipulation that are possible in custom impls. If you
need any of this, use Serde -- it's a great library.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
