# Omni Bar Plugin SDK

The shared Rust contract for Omni Bar and its statically linked native plugins.
This repository owns the protocol, borrowed invocation inputs, Host capabilities,
foreground-session interfaces, and stable error codes. The SDK package is
`omni-plugin-sdk` **3.0.0** and its exact `PLUGIN_PROTOCOL_VERSION` is **3**.

Omni Bar links approved plugins into the same macOS application and DMG. There is
no runtime installer, dynamic-library ABI, plugin daemon, filesystem discovery,
IPC transport, compatibility adapter, or protocol negotiation in this SDK.

## Repositories and dependencies

| Repository | Responsibility |
| --- | --- |
| [omni-bar](https://github.com/nilo-ggyt/omni-bar) | Host application, Launcher, static plugin assembly, foreground coordination, native integration and packaging |
| [omni-plugin-sdk](https://github.com/nilo-ggyt/omni-plugin-sdk) | This independent protocol crate and portable contract tests |
| [omni-md](https://github.com/nilo-ggyt/omni-md) | Native Markdown plugin, its private document/workspace core, and native editor tests |

Every future plugin owns its own implementation and repository. The SDK never
imports Host or concrete plugin modules. Host and all bundled plugins must resolve
**one SDK Cargo package identity**, including the same canonical Git URL and full
revision. Equal package versions or protocol numbers do not unify different Git,
path, or registry sources.

Initially, consumers use an immutable Git source. Replace `SDK_COMMIT` with the
reviewed full commit from this repository; it is a placeholder, not a release ref:

```toml
# Keep this exact source/revision identical in Host and every bundled plugin.
[workspace.dependencies]
omni-plugin-sdk = { git = "https://github.com/nilo-ggyt/omni-plugin-sdk.git", rev = "SDK_COMMIT", version = "=3.0.0" }
```

Member crates inherit that dependency with `omni-plugin-sdk.workspace = true`.
Keep a consumer lockfile and inspect `cargo metadata --locked --format-version 1`
to verify the SDK identity. Temporary local patches may support candidate work;
they do not establish that the committed release graph builds independently.
This task does not publish the SDK to crates.io (`publish = false`).

## Build and verify

Rust **1.99.0**, edition **2024**, is pinned in `rust-toolchain.toml`. The SDK itself
contains no AppKit dependency, so its contract tests can run on Linux or macOS.
Actual native plugin surfaces target Apple Silicon macOS, with macOS **14.0** as
the deployment floor. Building for that floor is not macOS 14 runtime evidence.

The 1.99.0 pin avoids the [trait-object vtable miscompilation in Rust 1.98.0](https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/),
fixed in 1.98.1. This matters to session and lifecycle trait objects even though the protocol does
not need newly stabilized language syntax. See the [1.99.0 release](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/).

Run these commands from a clean checkout:

```sh
# Check source layout and the selected compiler/safety diagnostics.
cargo +1.99.0 fmt --all -- --check
cargo +1.99.0 clippy --workspace --all-targets --locked -- -D warnings

# Verify public contracts and rustdoc examples against the committed dependency graph.
cargo +1.99.0 test --workspace --locked
cargo +1.99.0 test --workspace --doc --locked
cargo +1.99.0 doc --workspace --no-deps --locked
```

On a macOS development machine, also validate the product target:

```sh
# Build the same architecture consumed by the statically linked application.
MACOSX_DEPLOYMENT_TARGET=14.0 cargo +1.99.0 build --release --target aarch64-apple-darwin --locked
```

The [CI workflow](.github/workflows/ci.yml) uses Rust 1.99.0 on `ubuntu-24.04` and
`macos-26`, with the macOS target explicitly set to `aarch64-apple-darwin`.
These labels were checked against GitHub's [supported hosted runner list](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
on 2026-10-15. The workflow definition is not evidence that a remote run occurred.
Its macOS runner does not establish macOS 14 compatibility or Host UI acceptance.

## Public contract

All established public types remain exported from the crate root:

| Contract | Root items |
| --- | --- |
| Discoverability and inputs | `PLUGIN_PROTOCOL_VERSION`, `PluginDescriptor`, `PluginCommandDescriptor`, `ModifierFlags`, `PluginInvocation` |
| Static dispatch and resume | `PluginRegistration`, `PluginActivationHandler`, `PluginCommandTarget` |
| Host capabilities | `OmniBarHostContext` |
| Foreground ownership | `PluginActivation`, `PluginSession`, `PluginCloseIntent`, `PluginCloseScope`, `PluginCloseRequest`, `PluginHostLifecycle`, `PluginNativeView` |
| Failures | `PluginError` |

Private modules separate protocol metadata, registration, Host capabilities,
session ownership, and errors. Consumers do not depend on those implementation
module paths.

The Host reads only static descriptors while idle. `PluginRegistration::activate`
rejects a mismatched protocol or unknown command before calling plugin code. A
validated handler executes synchronously on the current thread and returns either
`Completed` or an owned `Session`. Query text, selected text, and selected paths
are borrowed for activation; a returned session cannot retain those borrows.

The Host context supplies workspace/application-data roots, explicit clipboard
operations, native-view mounting, and foreground lifecycle callbacks. This is a
same-process ownership boundary, not a sandbox or permission broker. Merely
obtaining a root path does not authorize eager scanning or resource creation.

## Native ownership and teardown

`PluginNativeView` borrows a plugin-owned `NSView`; copying it does not retain the
object. Constructing a non-null handle is unsafe: the caller guarantees a valid
view, native strong ownership throughout Host use, and AppKit main-thread access.
The handle is neither `Send` nor `Sync`. The Host must stop using and invalidate
its borrowed handle before the native object becomes unavailable.

A concrete plugin session owns its native/document resource graph. Save or flush
before requesting close; a failure preserves the usable surface and unsaved data.
After successful preparation, commit one explicit `PluginCloseRequest`, stop new
work and request native close. Every protocol-3 close entry carries both fields:

```rust
use omni_plugin_sdk::{PluginCloseIntent, PluginCloseRequest, PluginCloseScope};

// Declare the ordinary project-resource boundary and actual old-window release.
let request = PluginCloseRequest {
    intent: PluginCloseIntent::DismissAndResume,
    scope: PluginCloseScope::ForegroundTransition,
};
```

`PluginSession::request_close(request)`, `PluginHostLifecycle::begin_close(request)`
and `PluginHostLifecycle::finish_close(request)` use this same request type.
Protocol 3 changes the successful resource receipt from protocol 2 while retaining these
explicit request fields. The previous full-graph-nil meaning is not silently reused. There is
no implicit scope, default conversion, compatibility adapter or parallel protocol generation. The default session method explicitly refuses either
scope with `close_unsupported`; it does not claim lifecycle support or release.

| Close scope | Required release observation | Permitted completion |
| --- | --- | --- |
| `ForegroundTransition` | Canonical/session resources, buffers, files, mappings and tasks are released; explicitly owned roles pass actual deallocation or a concrete audited inert-shell receipt; borrowed framework editors are empty and disconnected; mounts invalidated and the old window actually deallocated. | The original ordinary successor, once, after session release. |
| `ApplicationTermination` | The same project-resource, inert-shell and borrowed-editor proofs pass; a validated closed, detached, empty framework window lease may remain until actual process exit. | Advance an already pending application quit, once; no successor or continued residency. |

The Host alone authorizes terminal scope after committing **actual pending quit**.
A plugin cannot select it because a close is slow, a window is invisible, an OS
reference remains, or a callback/timeout occurs. The window receipt requires
independent validation of closed, detached, empty and OS-only ownership facts;
the SDK's scalar request is no such proof. A permitted OS window lease lasts at
most until process exit and is explicitly **not deallocation**. It may retain no
canonical document, history, mapping, file, business buffer, live callback authority or
hidden reusable session. A concrete inert shell must have relinquished every project
payload field and allocation; weak document links or an empty visible string alone are
insufficient. Borrowed framework caches require public content/undo/delegate disconnection
and independent bounded-repeat evidence. Class-name waivers are forbidden. See the precise
[resource contract](RESOURCE_CONTRACT.md). No sleep, polling loop, force-exit or assumed queue
turn supplies release evidence.

An accepted request's `intent` is immutable. Identical repeats are idempotent and
do not save or close again. If actual quit overtakes an accepted ordinary close,
the Host may promote only `ForegroundTransition -> ApplicationTermination`, with
the **same intent**. The plugin first asks `begin_close(promoted_request)` to
validate promotion and updates its stored request only on success. Opposing
intent, terminal downgrade, unauthorized promotion or a stale transaction is
rejected without changing either participant's prior accepted request. An
accepted terminal request never later becomes an ordinary handoff.

`windowWillClose:`, a close request, the once-only teardown barrier, full native
deallocation and terminal empty-window receipt are distinct observations.
Finalization runs beyond relevant native callback stacks, disconnects native and
plugin callback ownership, releases payload and calls `finish_close` with the
**currently accepted request**. The Host validates its transaction identity and
exact request, rejects stale/pre-promotion or duplicate callbacks, invalidates
borrowed mounts and consumes the matching barrier once. It releases its session
after outstanding Rust borrows and callback frames return. Ordinary completion
can then allow the original successor; terminal completion can only continue
pending quit. Even an original `ExitToLauncher` request cannot create Launcher
from a terminal receipt. The same guarded policy applies to every statically
registered plugin.

`DismissAndResume` closes to zero business windows and remembers only a dormant
target for an explicit later activation. `ExitToLauncher` closes first and then
returns to Launcher. Idle plugin catalogs retain no session, heap cache, mapping,
file descriptor, timer, worker, or process. Host global-hotkey registration is separately
owned application state.

`Drop` supplies bounded, non-panicking cleanup of the session's owned payload. It
does not retry saves, show UI, wait for the OS, request another close or manufacture
release completion. Explicit methods retain fallible user work and error handling.

The SDK expresses these contracts; it does not implement the native state machine
or prove destruction. Tests here cover metadata, borrowed inputs, synchronous
dispatch, rejection ordering, error propagation, synthetic session ownership and
a portable guarded close-participant model. That model checks explicit pending
quit, same-intent promotion, stale/duplicate completion, payload ownership, empty
window predicates and scope-specific successor routing without AppKit objects.
Real `NSView` mounting, native weak-reference release, file/mapping closure,
callback reentrancy, IME/accessibility, physical footprint, energy, and DMG signing
remain Host/plugin integration responsibilities.

## Contributing and license

Read [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md) before changes.
Public API changes require explicit protocol and consumer-impact review. There
are no compatibility shims or simultaneous old/new protocols in one Host assembly.

MIT, Copyright (c) 2026 **nilo vance**. See [LICENSE](LICENSE). Dependency licensing
remains owned by the respective upstream packages.
