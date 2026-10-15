# SDK agent instructions

Read [README.md](README.md) and [CONTRIBUTING.md](CONTRIBUTING.md) before changing
source. Preserve unrelated edits; user instructions take precedence.

- This independent repository owns the shared static Rust protocol only. Do not
  depend on Host or concrete plugin implementations. Keep existing root API
  exports stable and implementation modules private.
- Use Rust 1.99.0/edition 2024, exact protocol equality, synchronous registration,
  and one SDK Cargo source identity across a Host assembly. Source extraction does
  not authorize runtime loading, installers, IPC, processes, negotiation, or
  compatibility layers.
- Allocate no plugin resources in dormant catalog metadata. The concrete active
  session owns resources; explicit save/close handles failures; the Host releases
  session ownership after its native teardown barrier.
- Use detailed English responsibility/contract docs and local explanations for
  every logical section, conditional branch, match arm, loop/closure/callback,
  failure exit, and resource transition. One function header is insufficient.
  Each unsafe block/implementation has a precise adjacent English `SAFETY`
  justification, and unsafe APIs expose caller obligations.
- Main-thread native ownership is not made safe by `Rc` or TLS alone. Never add
  unsafe `Send`/`Sync` to opaque native handles or fabricate an `NSView` in a pure
  test. Portable contract tests do not prove native release or performance.
- Run the applicable pinned fmt/Clippy/tests/doc checks, inspect dependency/source
  identity, and report commands and actual observations. Keep unexecuted remote
  CI and untested macOS versions explicit.
- Retain MIT attribution to nilo vance and applicable upstream notices. Do not
  include secrets, generated build products, local overrides, or private journals.
  Do not commit, push, or publish without authorization for the concrete action.
