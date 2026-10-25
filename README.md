# katzenpost-lean-sys
[![crates.io](https://img.shields.io/crates/v/katzenpost-lean-sys)](https://crates.io/crates/katzenpost-lean-sys)
[![docs.rs](https://img.shields.io/docsrs/katzenpost-lean-sys)](https://docs.rs/crate/katzenpost-lean-sys/latest)
[![lean version](https://img.shields.io/badge/lean-4.35.0-lightgray.svg)](#lean-version-requirements)

Rust bindings to [Lean 4](https://github.com/leanprover/lean4)'s C API

Katzenpost's fork of [lean-sys](https://github.com/digama0/lean-sys), for the Lean toolchain our
Lean packages use, published as `katzenpost-lean-sys`. Its library is still `lean_sys`:

```toml
[dependencies]
lean-sys = { package = "katzenpost-lean-sys", version = "0.1.0" }
```

A release is a tag `vX.Y.Z` naming the version in `Cargo.toml`; pushing it publishes the crate to
crates.io.

Functions and comments manually translated from those in the [`lean.h` header](https://github.com/leanprover/lean4/blob/master/src/include/lean/lean.h) provided with Lean 4
