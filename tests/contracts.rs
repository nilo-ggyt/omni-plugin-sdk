//! Portable semantic checks for public SDK behavior; real native lifecycle tests belong to its consumers.

use omni_plugin_sdk::{
    PLUGIN_PROTOCOL_VERSION, PluginCloseIntent, PluginCloseRequest, PluginCloseScope,
    PluginCommandDescriptor, PluginDescriptor, PluginNativeView, PluginSession,
};

/// Multiple commands expose accidental first-command dispatch and identity normalization.
const COMMANDS: &[PluginCommandDescriptor] = &[
    PluginCommandDescriptor {
        id: "open",
        title: "打开笔记",
        subtitle: "打开前台笔记会话",
        keywords: &["note", "open"],
    },
    PluginCommandDescriptor {
        id: "inspect",
        title: "检查笔记",
        subtitle: "执行当前命令",
        keywords: &["inspect"],
    },
];

/// These static values feed the actual SDK lookup, not a replacement Host or matching implementation.
const DESCRIPTOR: PluginDescriptor = PluginDescriptor {
    plugin_id: "example.notes",
    protocol_version: PLUGIN_PROTOCOL_VERSION,
    commands: COMMANDS,
    activation_code: "note",
    symbol: "N",
};

#[test]
fn declared_command_identity_is_exact_and_distinct_from_search_text() {
    // Search-facing titles and short codes cannot become activation identities;
    // the selected second command must resolve through its own declared id.
    let command = DESCRIPTOR.command("inspect").expect("declared command");
    assert_eq!(command.id, "inspect");
    for undeclared in ["missing", "Inspect", "inspect ", "检查笔记", "note"] {
        assert!(DESCRIPTOR.command(undeclared).is_none());
    }

    // Use the SDK's actual metadata stream as the Host would: a command's own
    // labels/keywords are searchable without borrowing a different command's fields.
    assert!(
        DESCRIPTOR
            .search_fields(command)
            .any(|field| field == "inspect")
    );
    assert!(
        DESCRIPTOR
            .search_fields(command)
            .any(|field| field == "检查笔记")
    );
    assert!(
        !DESCRIPTOR
            .search_fields(command)
            .any(|field| field == "打开笔记")
    );
}

/// A resource-free plugin can use the real default methods without supplying a fake Host.
struct ViewlessSession;

impl PluginSession for ViewlessSession {}

#[test]
fn an_unimplemented_close_never_claims_either_release_scope() {
    // No native owner or lifecycle implementation exists here. Calling the SDK
    // default must explicitly refuse every request instead of claiming cleanup.
    let mut session = ViewlessSession;
    for intent in [
        PluginCloseIntent::DismissAndResume,
        PluginCloseIntent::ExitToLauncher,
    ] {
        for scope in [
            PluginCloseScope::ForegroundTransition,
            PluginCloseScope::ApplicationTermination,
        ] {
            // Both policy dimensions are supplied through the public method;
            // repeated rejection must not turn a viewless session into a close receipt.
            let error = session
                .request_close(PluginCloseRequest { intent, scope })
                .expect_err("close requires an actual implementation");
            assert_eq!(error.code(), "close_unsupported");
            assert!(session.native_view().is_none());
        }
    }
}

#[test]
fn a_null_native_borrow_is_rejected_before_any_native_access() {
    // A null native boundary value is permitted by this unsafe constructor's
    // public contract; no fabricated NSView or ownership receipt is involved.
    // SAFETY: null is explicitly accepted and must return None without dereferencing it.
    let view = unsafe { PluginNativeView::from_raw(std::ptr::null_mut()) };
    assert!(view.is_none());
}
