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
- 按最新用户 AGENTS，新增/修改的源码注释、rustdoc、架构与生命周期说明使用简洁准确中文；标识符、错误码、协议字段与标准 `SAFETY` / `# Safety` 标记保留。公开集成文档仍提供英文。
- 逐一解释逻辑阶段、条件分支、match、循环、回调、失败出口和资源转换；长函数不能只写一条概要。注释解释约束、不变量与后果，不机械翻译语法；不批量改写第三方或既有注释。
- Main-thread native ownership is not made safe by `Rc` or TLS alone. Never add
  unsafe `Send`/`Sync` to opaque native handles or fabricate an `NSView` in a pure
  test. Portable contract tests do not prove native release or performance.
- Run the applicable pinned fmt/Clippy/tests/doc checks, inspect dependency/source
  identity, and report commands and actual observations. Keep unexecuted remote
  CI and untested macOS versions explicit.
- Retain MIT attribution to nilo vance and applicable upstream notices. Do not
  include secrets, generated build products, local overrides, or private journals.
  Do not commit, push, or publish without authorization for the concrete action.
