//! 协议 3 的可移植权限模型：close scope、单调提升与收据路由；不模拟原生资源证明。
//!
//! The SDK supplies data/traits, not this state machine. These fake participants exercise an
//! implementation of the documented obligations; booleans never prove real AppKit deallocation.

use std::cell::Cell;
use std::rc::{Rc, Weak};

use omni_plugin_sdk::{
    PluginCloseIntent, PluginCloseRequest, PluginCloseScope, PluginError, PluginHostLifecycle,
    PluginSession,
};

/// Synthetic native observations separate payload release from a possible OS-only empty-window lease.
#[derive(Clone, Copy, Default)]
struct NativeFacts {
    payload_released: bool,
    explicit_objects_alive: bool,
    inert_payload_empty: bool,
    inert_authority_revoked: bool,
    borrowed_relations_cleared: bool,
    window_alive: bool,
    window_closed: bool,
    window_detached: bool,
    window_empty: bool,
    window_os_only: bool,
}

impl NativeFacts {
    /// Model the terminal receipt without inventing native object identities in a portable test.
    fn terminal_window() -> Self {
        // Every terminal predicate is independent; later regressions remove one at a time.
        Self {
            payload_released: true,
            explicit_objects_alive: false,
            inert_payload_empty: false,
            inert_authority_revoked: false,
            borrowed_relations_cleared: true,
            window_alive: true,
            window_closed: true,
            window_detached: true,
            window_empty: true,
            window_os_only: true,
        }
    }

    /// 所有显式角色已析构且借用关系已清；不对框架私有组合对象作销毁声明。
    fn all_released() -> Self {
        // With no live window, no terminal-only window predicate is needed or inferred.
        Self {
            payload_released: true,
            borrowed_relations_cleared: true,
            ..Self::default()
        }
    }
}

/// One fake Host owns authorization and completion policy, with only a weak canonical-payload probe.
struct ContractHost {
    created: Cell<bool>,
    pending_quit: Cell<bool>,
    accepted: Cell<Option<PluginCloseRequest>>,
    payload: Weak<()>,
    native: Cell<NativeFacts>,
    completed: Cell<bool>,
    ignored_completions: Cell<usize>,
    launcher_successors: Cell<usize>,
    terminal_completions: Cell<usize>,
}

impl PluginHostLifecycle for ContractHost {
    /// Admit one modeled foreground owner before any close policy is evaluated.
    fn commit_creation(&self) -> Result<(), PluginError> {
        if self.created.replace(true) {
            // A duplicate creation cannot silently acquire another instance of this transaction.
            return Err(PluginError::new("creation_conflict", "duplicate creation"));
        }
        // A fresh participant now owns one transaction, without any native resource in this model.
        Ok(())
    }

    /// Model Host authority and monotonic request admission without changing the SDK's trait behavior.
    fn begin_close(&self, request: PluginCloseRequest) -> Result<(), PluginError> {
        if !self.created.get() || self.completed.get() {
            // Missing or consumed ownership rejects every request without reviving the old session.
            return Err(PluginError::new("stale_close", "inactive transaction"));
        }
        if request.scope == PluginCloseScope::ApplicationTermination && !self.pending_quit.get() {
            // The plugin's requested enum alone cannot grant actual pending-quit authority.
            return Err(PluginError::new("quit_not_pending", "quit not authorized"));
        }
        if let Some(accepted) = self.accepted.get() {
            // Existing admission constrains both fields before either participant may replace its copy.
            if request.intent != accepted.intent {
                // Even a legitimate quit cannot replace the original dismissal/Launcher intent.
                return Err(PluginError::new(
                    "intent_conflict",
                    "original intent retained",
                ));
            }
            if accepted.scope == PluginCloseScope::ApplicationTermination
                && request.scope == PluginCloseScope::ForegroundTransition
            {
                // Terminal authority is monotonic and cannot later permit an ordinary successor.
                return Err(PluginError::new(
                    "scope_downgrade",
                    "terminal scope retained",
                ));
            }
            // Equal requests are idempotent; the only different admitted scope is authorized promotion.
        }
        // Publish only after every guard passes; rejection leaves the prior accepted value intact.
        self.accepted.set(Some(request));
        Ok(())
    }

    /// Route one independently validated modeled receipt using its exact accepted request.
    fn finish_close(&self, request: PluginCloseRequest) {
        if self.completed.get() || self.accepted.get() != Some(request) {
            // A pre-promotion or duplicate callback cannot consume the current transaction's completion.
            self.ignored_completions
                .set(self.ignored_completions.get() + 1);
            return;
        }
        let facts = self.native.get();
        let explicit_roles_complete = !facts.explicit_objects_alive
            || (facts.inert_payload_empty && facts.inert_authority_revoked);
        if self.payload.upgrade().is_some()
            || !facts.payload_released
            || !explicit_roles_complete
            || !facts.borrowed_relations_cleared
        {
            // 两个 scope 都拒绝业务数据、未受检的活对象或未清除的借用权限；空字符串单项不足。
            self.ignored_completions
                .set(self.ignored_completions.get() + 1);
            return;
        }
        let scope_complete = match request.scope {
            PluginCloseScope::ForegroundTransition => {
                // 普通交接除共同资源收据外，还要求旧窗口实际析构，不能借用 terminal 例外。
                !facts.window_alive
            }
            PluginCloseScope::ApplicationTermination => {
                // A remaining window is admissible only with the complete terminal receipt and actual quit.
                self.pending_quit.get()
                    && (!facts.window_alive
                        || (facts.window_closed
                            && facts.window_detached
                            && facts.window_empty
                            && facts.window_os_only))
            }
        };
        if !scope_complete {
            // Missing release evidence keeps the close pending; no elapsed delay or retry repairs it.
            self.ignored_completions
                .set(self.ignored_completions.get() + 1);
            return;
        }
        // Consume once before routing the resulting transaction, preserving scope in both outcomes.
        self.completed.set(true);
        match request.scope {
            PluginCloseScope::ForegroundTransition => {
                if request.intent == PluginCloseIntent::ExitToLauncher && !self.pending_quit.get() {
                    // Launcher is a permitted ordinary successor only when application quit is absent.
                    self.launcher_successors
                        .set(self.launcher_successors.get() + 1);
                }
                // Dismissal stays hidden; an already pending quit suppresses every ordinary successor.
            }
            PluginCloseScope::ApplicationTermination => {
                // Terminal receipt advances only process termination, even for the original Launcher intent.
                self.terminal_completions
                    .set(self.terminal_completions.get() + 1);
            }
        }
    }
}

/// A fake session owns one actual Rust payload until explicit release or bounded Drop.
struct ContractSession {
    host: Rc<ContractHost>,
    payload: Option<Rc<()>>,
    accepted: Option<PluginCloseRequest>,
    reject_save: bool,
    saves: usize,
    native_closes: usize,
    drops: Rc<Cell<usize>>,
}

impl ContractSession {
    /// Acquire isolated test participants without native handles or real user data.
    fn new() -> Self {
        // One strong payload belongs to the session; the Host can observe its release without keeping it alive.
        let payload = Rc::new(());
        let host = Rc::new(ContractHost {
            created: Cell::new(false),
            pending_quit: Cell::new(false),
            accepted: Cell::new(None),
            payload: Rc::downgrade(&payload),
            native: Cell::new(NativeFacts::default()),
            completed: Cell::new(false),
            ignored_completions: Cell::new(0),
            launcher_successors: Cell::new(0),
            terminal_completions: Cell::new(0),
        });
        // Creation precedes every close admission; this model owns no hidden second lifecycle instance.
        host.commit_creation().unwrap();
        Self {
            host,
            payload: Some(payload),
            accepted: None,
            reject_save: false,
            saves: 0,
            native_closes: 0,
            drops: Rc::new(Cell::new(0)),
        }
    }

    /// End canonical ownership explicitly before asking the Host to accept a release receipt.
    fn release_payload(&mut self) {
        // Dropping the sole strong Rc makes the fake Host's weak probe independently observable.
        drop(self.payload.take());
    }
}

impl PluginSession for ContractSession {
    /// Model one initial preflight plus idempotent/authorized changes accepted by the Host owner.
    fn request_close(&mut self, request: PluginCloseRequest) -> Result<(), PluginError> {
        if self.accepted.is_some() {
            // A repeat or promotion asks the Host first and never saves/closes a second time.
            self.host.begin_close(request)?;
            self.accepted = Some(request);
            return Ok(());
        }
        // Initial fallible user work precedes lifecycle admission and native close.
        self.saves += 1;
        if self.reject_save {
            // Save failure retains the canonical payload and never publishes a destructive transition.
            return Err(PluginError::new("save_failed", "synthetic save refusal"));
        }
        self.host.begin_close(request)?;
        // Only successful admission replaces local identity and issues exactly one native close request.
        self.accepted = Some(request);
        self.native_closes += 1;
        Ok(())
    }
}

impl Drop for ContractSession {
    /// Release the synthetic owner without participating in any fallible close operation.
    fn drop(&mut self) {
        // Drop performs finite scalar/resource cleanup only, without save, callbacks, IO or completion claims.
        self.release_payload();
        self.drops.set(self.drops.get() + 1);
    }
}

/// Construct both policy fields explicitly; this helper grants no Host transaction authority.
fn request(intent: PluginCloseIntent, scope: PluginCloseScope) -> PluginCloseRequest {
    // Both fields remain caller-selected and explicit; constructing this value commits no lifecycle state.
    PluginCloseRequest { intent, scope }
}

/// The initial terminal request is rejected until actual pending quit, retaining all live user resources.
#[test]
fn initial_terminal_scope_requires_host_authority() {
    // Start with a live ordinary participant and no Host-owned pending-quit transaction.
    let mut session = ContractSession::new();
    let terminal = request(
        PluginCloseIntent::ExitToLauncher,
        PluginCloseScope::ApplicationTermination,
    );
    assert_eq!(
        session.request_close(terminal).unwrap_err().code(),
        "quit_not_pending"
    );
    assert_eq!(session.host.accepted.get(), None);
    assert_eq!(session.accepted, None);
    assert!(session.host.payload.upgrade().is_some());
    assert_eq!(session.native_closes, 0);

    // The Host explicitly enters pending quit before repeating the failed initial preflight.
    session.host.pending_quit.set(true);
    session.request_close(terminal).unwrap();
    assert_eq!(session.host.accepted.get(), Some(terminal));
    assert_eq!(session.accepted, Some(terminal));
    assert_eq!(session.saves, 2);
    assert_eq!(session.native_closes, 1);
}

/// Promotion preserves both possible original intents and never repeats successful save/native close.
#[test]
fn accepted_scope_promotes_only_monotonically_without_changing_intent() {
    for intent in [
        PluginCloseIntent::DismissAndResume,
        PluginCloseIntent::ExitToLauncher,
    ] {
        // Each original intent owns an independent transaction so no state leaks between participants.
        let mut session = ContractSession::new();
        let foreground = request(intent, PluginCloseScope::ForegroundTransition);
        let terminal = request(intent, PluginCloseScope::ApplicationTermination);
        session.request_close(foreground).unwrap();
        session.request_close(foreground).unwrap();
        assert!(session.request_close(terminal).is_err());
        assert_eq!(session.accepted, Some(foreground));
        assert_eq!(session.host.accepted.get(), Some(foreground));

        // Host-authorized promotion is idempotent and updates both participant copies only after validation.
        session.host.pending_quit.set(true);
        session.request_close(terminal).unwrap();
        session.request_close(terminal).unwrap();
        assert_eq!(session.accepted, Some(terminal));
        assert_eq!(session.host.accepted.get(), Some(terminal));
        assert_eq!(
            session.request_close(foreground).unwrap_err().code(),
            "scope_downgrade"
        );
        let opposite = match intent {
            PluginCloseIntent::DismissAndResume => {
                // Opposing Launcher intent must not overwrite an accepted dismissal during quit.
                PluginCloseIntent::ExitToLauncher
            }
            PluginCloseIntent::ExitToLauncher => {
                // Opposing dismissal likewise cannot erase an accepted Launcher intent.
                PluginCloseIntent::DismissAndResume
            }
        };
        assert_eq!(
            session
                .request_close(request(opposite, terminal.scope))
                .unwrap_err()
                .code(),
            "intent_conflict"
        );
        assert_eq!(session.accepted, Some(terminal));
        assert_eq!(session.saves, 1);
        assert_eq!(session.native_closes, 1);
    }
}

/// An empty OS-leased window is insufficient for ordinary handoff even after all payload is released.
#[test]
fn foreground_completion_requires_old_window_deallocation() {
    // Ordinary Launcher handoff cannot consume a receipt that would only be valid for process termination.
    let mut session = ContractSession::new();
    let foreground = request(
        PluginCloseIntent::ExitToLauncher,
        PluginCloseScope::ForegroundTransition,
    );
    session.request_close(foreground).unwrap();
    session.release_payload();
    session.host.native.set(NativeFacts::terminal_window());
    session.host.finish_close(foreground);
    assert!(!session.host.completed.get());
    assert_eq!(session.host.launcher_successors.get(), 0);

    // 旧窗口的实际释放是独立条件，不能把切换 scope 当成普通交接的退路。
    session.host.native.set(NativeFacts::all_released());
    session.host.finish_close(foreground);
    assert!(session.host.completed.get());
    assert_eq!(session.host.launcher_successors.get(), 1);
    assert_eq!(session.host.terminal_completions.get(), 0);
}

/// 空壳业务数据、回调权限和共享编辑器关系逐项必要，窗口 nil 不能代替这些收据。
#[test]
fn foreground_requires_complete_owned_and_borrowed_receipts_with_an_inert_shell() {
    let mut session = ContractSession::new();
    let foreground = request(
        PluginCloseIntent::ExitToLauncher,
        PluginCloseScope::ForegroundTransition,
    );
    session.request_close(foreground).unwrap();
    session.release_payload();
    let valid = NativeFacts {
        explicit_objects_alive: true,
        inert_payload_empty: true,
        inert_authority_revoked: true,
        ..NativeFacts::all_released()
    };
    for invalid in [
        NativeFacts {
            inert_payload_empty: false,
            ..valid
        },
        NativeFacts {
            inert_authority_revoked: false,
            ..valid
        },
        NativeFacts {
            borrowed_relations_cleared: false,
            ..valid
        },
    ] {
        // 每次仅撤掉一个必要事实；保留原请求且不生成后继，不能用低内存或时间代替。
        session.host.native.set(invalid);
        session.host.finish_close(foreground);
        assert!(!session.host.completed.get());
        assert_eq!(session.host.launcher_successors.get(), 0);
    }
    // 这里只验证具备完整空壳收据时的路由规则，实际字段与原生对象由消费者另行验收。
    session.host.native.set(valid);
    session.host.finish_close(foreground);
    assert!(session.host.completed.get());
    assert_eq!(session.host.launcher_successors.get(), 1);
}

/// Every terminal receipt predicate matters, and no receipt may retain actual canonical payload.
#[test]
fn terminal_completion_requires_released_payload_and_a_valid_window_receipt() {
    // Real pending-quit authority changes receipt routing, while leaving payload-release obligations intact.
    let mut session = ContractSession::new();
    session.host.pending_quit.set(true);
    let terminal = request(
        PluginCloseIntent::ExitToLauncher,
        PluginCloseScope::ApplicationTermination,
    );
    session.request_close(terminal).unwrap();
    let valid = NativeFacts::terminal_window();
    session.host.native.set(valid);
    session.host.finish_close(terminal);
    assert!(!session.host.completed.get());
    session.release_payload();

    for invalid in [
        NativeFacts {
            payload_released: false,
            ..valid
        },
        NativeFacts {
            explicit_objects_alive: true,
            inert_payload_empty: false,
            inert_authority_revoked: true,
            ..valid
        },
        NativeFacts {
            explicit_objects_alive: true,
            inert_payload_empty: true,
            inert_authority_revoked: false,
            ..valid
        },
        NativeFacts {
            borrowed_relations_cleared: false,
            ..valid
        },
        NativeFacts {
            window_closed: false,
            ..valid
        },
        NativeFacts {
            window_detached: false,
            ..valid
        },
        NativeFacts {
            window_empty: false,
            ..valid
        },
        NativeFacts {
            window_os_only: false,
            ..valid
        },
    ] {
        // Missing one independent terminal condition keeps the same request pending without a successor.
        session.host.native.set(invalid);
        session.host.finish_close(terminal);
        assert!(!session.host.completed.get());
        assert_eq!(session.host.terminal_completions.get(), 0);
        assert_eq!(session.host.launcher_successors.get(), 0);
    }
    // Complete terminal receipt permits only the already authorized quit, never the remembered Launcher action.
    session.host.native.set(valid);
    session.host.finish_close(terminal);
    assert!(session.host.completed.get());
    assert_eq!(session.host.terminal_completions.get(), 1);
    assert_eq!(session.host.launcher_successors.get(), 0);
}

/// terminal 可消费完整的惰性空壳与借用关系收据，但绝不能按原 intent 创建 Launcher。
#[test]
fn terminal_completion_accepts_audited_inert_roles_without_an_ordinary_successor() {
    // Host 先授予真实 pending quit；显式的 Launcher intent 使错误后继路由可被观察。
    let mut session = ContractSession::new();
    session.host.pending_quit.set(true);
    let terminal = request(
        PluginCloseIntent::ExitToLauncher,
        PluginCloseScope::ApplicationTermination,
    );
    session.request_close(terminal).unwrap();

    // 真实 Rust payload 必须先结束所有权；其弱引用 nil 独立于模型中的空壳事实。
    session.release_payload();
    assert!(session.host.payload.upgrade().is_none());
    session.host.native.set(NativeFacts {
        explicit_objects_alive: true,
        inert_payload_empty: true,
        inert_authority_revoked: true,
        ..NativeFacts::terminal_window()
    });

    // 模型只验证已具备全部收据时的权限路由，不用 bool 声称实际 AppKit 对象已清空。
    session.host.finish_close(terminal);
    assert_eq!(session.host.accepted.get(), Some(terminal));
    assert!(session.host.completed.get());
    assert_eq!(session.host.terminal_completions.get(), 1);
    assert_eq!(session.host.launcher_successors.get(), 0);

    // 重复回执不能消费第二次终止权限，也不能恢复被抑制的普通后继。
    session.host.finish_close(terminal);
    assert_eq!(session.host.ignored_completions.get(), 1);
    assert_eq!(session.host.terminal_completions.get(), 1);
    assert_eq!(session.host.launcher_successors.get(), 0);
}

/// Completion preserves the promoted scope and consumes once despite stale/duplicate callbacks.
#[test]
fn stale_and_duplicate_completion_cannot_turn_terminal_receipt_into_a_successor() {
    // Accept ordinary intent first so the later terminal close has a genuine stale predecessor callback.
    let mut session = ContractSession::new();
    let foreground = request(
        PluginCloseIntent::ExitToLauncher,
        PluginCloseScope::ForegroundTransition,
    );
    let terminal = request(foreground.intent, PluginCloseScope::ApplicationTermination);
    session.request_close(foreground).unwrap();
    session.host.pending_quit.set(true);
    session.request_close(terminal).unwrap();
    session.release_payload();
    session.host.native.set(NativeFacts::terminal_window());

    // The old foreground callback cannot consume the promoted transaction even if its original intent matches.
    session.host.finish_close(foreground);
    assert!(!session.host.completed.get());
    session.host.finish_close(terminal);
    session.host.finish_close(terminal);
    session.host.finish_close(foreground);
    assert_eq!(session.host.ignored_completions.get(), 3);
    assert_eq!(session.host.terminal_completions.get(), 1);
    assert_eq!(session.host.launcher_successors.get(), 0);
    assert_eq!(
        session.host.begin_close(terminal).unwrap_err().code(),
        "stale_close"
    );

    // Host session release occurs after the callback, with canonical payload already gone and bounded Drop only.
    let drops = session.drops.clone();
    assert_eq!(drops.get(), 0);
    assert!(session.host.payload.upgrade().is_none());
    drop(session);
    assert_eq!(drops.get(), 1);
}

/// An already deallocated native window also satisfies terminal scope without a surviving-window receipt.
#[test]
fn terminal_scope_accepts_complete_release_without_reclassifying_it_as_ordinary() {
    // Full native release can satisfy pending quit directly, without requiring an artificial surviving window.
    let mut session = ContractSession::new();
    session.host.pending_quit.set(true);
    let terminal = request(
        PluginCloseIntent::ExitToLauncher,
        PluginCloseScope::ApplicationTermination,
    );
    session.request_close(terminal).unwrap();
    session.release_payload();
    session.host.native.set(NativeFacts::all_released());
    session.host.finish_close(terminal);
    assert_eq!(session.host.terminal_completions.get(), 1);
    assert_eq!(session.host.launcher_successors.get(), 0);
}

/// Failed save retains usable payload before admission; eventual Drop does not retry or claim completion.
#[test]
fn failed_save_preserves_ownership_and_drop_is_not_a_second_close() {
    // A user-work refusal occurs before Host admission and cannot abandon the canonical resource owner.
    let mut session = ContractSession::new();
    session.reject_save = true;
    let foreground = request(
        PluginCloseIntent::DismissAndResume,
        PluginCloseScope::ForegroundTransition,
    );
    assert_eq!(
        session.request_close(foreground).unwrap_err().code(),
        "save_failed"
    );
    assert!(session.host.payload.upgrade().is_some());
    assert_eq!(session.host.accepted.get(), None);
    assert_eq!(session.native_closes, 0);
    let host = session.host.clone();
    let drops = session.drops.clone();

    // Explicit test destruction releases only owned Rust resources and cannot replace successful save preflight.
    drop(session);
    assert_eq!(drops.get(), 1);
    assert!(host.payload.upgrade().is_none());
    assert!(!host.completed.get());
    assert_eq!(host.terminal_completions.get(), 0);
    assert_eq!(host.launcher_successors.get(), 0);
}
