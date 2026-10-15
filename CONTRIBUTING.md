# Contributing

This repository owns the shared native plugin contract, not the Host application
or a concrete plugin. Start with [README.md](README.md) for source integration and
the exact-version delivery model. Keep changes focused on an identified protocol,
ownership, error, or verification need.

## Structure and API stability

The crate root re-exports the public API. `protocol.rs` owns immutable descriptors
and borrowed invocation inputs; `registration.rs` owns guarded synchronous
dispatch; `host.rs` owns minimal Host capabilities; `session.rs` owns foreground
and native-view contracts; `error.rs` owns bounded static errors. Keep these
implementation modules private. Do not add Host or concrete-plugin dependencies,
a generic service manager, runtime discovery, loaders, installers, IPC, background
tasks, or protocol negotiation.

Package version, Git source revision, and `PLUGIN_PROTOCOL_VERSION` are separate
identifiers. Review changes to public types/traits, error codes, invocation
borrowing, thread guarantees, and lifecycle order with every bundled consumer.
Preserve current signatures and semantics unless a protocol change is explicitly
authorized. Never repair incompatible consumers with a silent adapter. Host
release assembly must resolve one SDK package identity.

## Rust and comment requirements

Use Rust 1.99.0, edition 2024, default rustfmt, and the committed lockfile. Workspace
members explicitly inherit selected lint policy. No production nightly or blanket
restriction-lint set is required.

All project-owned documentation and comments are English. Document modules, types,
traits, functions, methods, public fields, errors, and unsafe caller obligations.
Inside functions, explain every logical stage and each conditional branch or
meaningful match arm, including fallbacks and early errors. Explain the purpose,
bounds, ownership, and cancellation consequences of loops, closures, callbacks,
and resource transitions. A function summary does not cover its internal branches.
Comments must explain intent and invariants rather than translate syntax.

Every unsafe block or unsafe implementation has a specific adjacent `SAFETY`
comment. Unsafe APIs have a `# Safety` section. Keep pointer provenance, owner,
lifetime, aliasing, main-thread, and native exception/unwind constraints explicit
where applicable. Preserve third-party/generated-code maintenance boundaries.

Use ownership before shared mutability. Keep `Rc`, `RefCell`, locks, and callbacks
justified by a real contract. Never add unsafe `Send`/`Sync` to native view handles.
Do not hold mutable borrows across potentially reentrant AppKit calls. `Drop`
performs bounded non-panicking cleanup; explicit methods handle fallible save/close.

## Validation

Run the format, Clippy, tests, doc-tests, rustdoc, and Apple Silicon build commands
in [README.md](README.md#build-and-verify). Source changes require tests for a real
contract or regression boundary. Portable tests use isolated fake participants;
they must not fabricate a byte buffer and claim it is a valid AppKit `NSView`.

Record the compiler, target, profile, exact command, and observed result. Local
path overrides can test candidates but cannot prove the public pinned Git graph.
SDK, plugin, and Host checks are independent; the Host must additionally inspect
Cargo metadata and run native composition/lifecycle tests.

Review internal comments and actual ownership paths separately from compiler/lint
results. Clippy checks safety-comment structure, not correctness or complete
semantic explanation. Portable session/Rc observations do not establish native
deallocation, real UI behavior, memory, energy, or release packaging.

## Source and publication

Include source, MIT attribution, lockfile, and maintained documentation. Exclude
credentials, private environment files, build/cache outputs, personal journals,
and temporary patches. Review the actual staged tree; ignore rules do not remove
already tracked material. SDK GitHub releases and any future registry workflow
require explicit review. This repository currently has `publish = false` and does
not publish to crates.io. Do not commit, push, or publish as an incidental cleanup.
