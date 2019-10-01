mda-rs
======

mda-rs is a Rust library for writing custom Mail Deliver Agents.

![](https://github.com/afrantzis/mda-rs/workflows/build/badge.svg)

### Documentation

The detailed module documentation, including code examples for all features,
can be found at [https://docs.rs/mda](https://docs.rs/mda).

### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
mda = "0.1"
```

If you are using Rust 2015 add the following to your crate root file (Rust 2018
doesn't require this):

```rust
extern crate mda;
```

See [examples/personal-mda.rs](examples/personal-mda.rs) for an example that
uses mda-rs.

### License

This project is licensed under the Mozilla Public License Version 2.0
([LICENSE](LICENSE) or https://www.mozilla.org/en-US/MPL/2.0/).
