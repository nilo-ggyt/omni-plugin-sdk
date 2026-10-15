//! Public API and observable contract tests without AppKit or real user resources.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use omni_plugin_sdk::{
    ModifierFlags, OmniBarHostContext, PLUGIN_PROTOCOL_VERSION, PluginActivation,
    PluginActivationHandler, PluginCloseIntent, PluginCloseRequest, PluginCloseScope,
    PluginCommandDescriptor, PluginCommandTarget, PluginDescriptor, PluginError,
    PluginHostLifecycle, PluginInvocation, PluginNativeView, PluginRegistration, PluginSession,
};

/// Real multi-command metadata prevents a single-command fixture masking lookup errors.
const COMMANDS: &[PluginCommandDescriptor] = &[
    PluginCommandDescriptor {
        id: "open",
        title: "Example",
        subtitle: "Open a foreground session",
        keywords: &["sample", "open"],
    },
    PluginCommandDescriptor {
        id: "inspect",
        title: "Inspect",
        subtitle: "Complete an inline action",
        keywords: &["inspect"],
    },
];

/// Every test registration uses a static descriptor, as real Host assembly does.
const DESCRIPTOR: PluginDescriptor = PluginDescriptor {
    plugin_id: "com.omnibar.example",
    protocol_version: PLUGIN_PROTOCOL_VERSION,
    commands: COMMANDS,
    activation_code: "ex",
    symbol: "E",
};

/// Both older and newer descriptors must be rejected rather than negotiated.
const OLDER_DESCRIPTOR: PluginDescriptor = PluginDescriptor {
    protocol_version: PLUGIN_PROTOCOL_VERSION - 1,
    ..DESCRIPTOR
};

/// A future contract is just as incompatible as an obsolete contract.
const NEWER_DESCRIPTOR: PluginDescriptor = PluginDescriptor {
    protocol_version: PLUGIN_PROTOCOL_VERSION + 1,
    ..DESCRIPTOR
};

/// Synthetic lifecycle observations prove trait wiring, not native deallocation.
#[derive(Default)]
struct FakeLifecycle {
    events: RefCell<Vec<&'static str>>,
    close_request: Cell<Option<PluginCloseRequest>>,
}

impl PluginHostLifecycle for FakeLifecycle {
    /// Observe inline creation without allocating native resources.
    fn commit_creation(&self) -> Result<(), PluginError> {
        // Recording before the handler returns makes synchronous ordering observable.
        self.events.borrow_mut().push("created");
        Ok(())
    }

    /// Record both explicit lifecycle fields while the fake session still exists.
    fn begin_close(&self, request: PluginCloseRequest) -> Result<(), PluginError> {
        // These ordinary wiring tests forward the request; guarded policy cases have a separate model.
        self.close_request.set(Some(request));
        self.events.borrow_mut().push("closing");
        Ok(())
    }

    /// Model the Host's notification after an independently observed teardown barrier.
    fn finish_close(&self, request: PluginCloseRequest) {
        // This is a synthetic test event; real AppKit barrier/release tests belong to Host.
        self.close_request.set(Some(request));
        self.events.borrow_mut().push("finished");
    }
}

/// An isolated host keeps activation effects visible without accessing user data.
#[derive(Default)]
struct FakeHost {
    clipboard: Option<String>,
    write_attempts: usize,
    last_write_thread: Option<std::thread::ThreadId>,
    reject_writes: bool,
    mounted_view: Option<PluginNativeView>,
    lifecycle: Rc<FakeLifecycle>,
}

impl OmniBarHostContext for FakeHost {
    /// Supply an inert path; this fake never creates or scans it.
    fn workspace_root(&self) -> &Path {
        // Path retrieval remains a borrowed capability rather than eager filesystem work.
        Path::new("/synthetic/workspace")
    }

    /// Keep application-data access independent from workspace authority.
    fn application_data_root(&self) -> &Path {
        // No directory is created merely because an activation can ask for its location.
        Path::new("/synthetic/application-data")
    }

    /// Store a borrowed handle without pretending to own a native strong reference.
    fn mount_native_view(&mut self, view: PluginNativeView) -> Result<(), PluginError> {
        // This capability is part of the public API; portable tests never fabricate NSViews.
        self.mounted_view = Some(view);
        Ok(())
    }

    /// Return only the fixture text owned by this fake host.
    fn read_clipboard_text(&mut self) -> Result<Option<String>, PluginError> {
        // Cloning synthetic text models the explicit owned return value, not an SDK cache.
        Ok(self.clipboard.clone())
    }

    /// Expose handler execution and failure propagation through an observable host effect.
    fn write_clipboard_text(&mut self, text: &str) -> Result<(), PluginError> {
        // Count attempts before rejection so tests can prove whether dispatch occurred at all.
        self.write_attempts += 1;
        self.last_write_thread = Some(std::thread::current().id());
        if self.reject_writes {
            // Refuse without changing the previous clipboard value, preserving failure state.
            return Err(PluginError::new("clipboard_unavailable", "fixture refusal"));
        }

        // Successful explicit writes replace only this test's private synthetic value.
        self.clipboard = Some(text.to_owned());
        Ok(())
    }

    /// Share one lifecycle owner with the activated session.
    fn foreground_lifecycle(&self) -> Rc<dyn PluginHostLifecycle> {
        // Unsizing the Rc preserves the same owner and permits reference-release assertions.
        self.lifecycle.clone()
    }
}

/// A session shares only its lifecycle fixture; no native object is fabricated.
struct OwnedSession {
    lifecycle: Rc<dyn PluginHostLifecycle>,
}

impl PluginSession for OwnedSession {
    /// Forward the explicit close request while retaining ownership until Host release.
    fn request_close(&mut self, request: PluginCloseRequest) -> Result<(), PluginError> {
        // A close request is not Drop: the Host still owns this session after the callback.
        self.lifecycle.begin_close(request)
    }
}

/// The default trait methods represent a valid session without native close support.
struct ViewlessSession;

impl PluginSession for ViewlessSession {}

/// Build borrowed fixture inputs without introducing shared global test state.
fn invocation(command_id: &str) -> PluginInvocation<'_> {
    // Static optional inputs make the command borrow the only variable test input.
    PluginInvocation {
        command_id,
        query: "synthetic query",
        selected_text: None,
        selected_path: None,
        modifiers: ModifierFlags::default(),
    }
}

/// Complete one host effect inline and propagate its stable failure unchanged.
fn complete_inline(
    host: &mut dyn OmniBarHostContext,
    invocation: &PluginInvocation<'_>,
) -> Result<PluginActivation, PluginError> {
    // Explicit host refusal stops activation without changing its original error contract.
    host.write_clipboard_text(invocation.query)?;

    // No session ownership is created for this command's completed inline work.
    Ok(PluginActivation::Completed)
}

/// Activate a synthetic session and share its Host-provided lifecycle owner.
fn open_session(
    host: &mut dyn OmniBarHostContext,
    _invocation: &PluginInvocation<'_>,
) -> Result<PluginActivation, PluginError> {
    // Observe creation during activation; a rejected callback would propagate before transfer.
    let lifecycle = host.foreground_lifecycle();
    lifecycle.commit_creation()?;

    // The returned Box is the only session owner and keeps one lifecycle reference alive.
    Ok(PluginActivation::Session(Box::new(OwnedSession {
        lifecycle,
    })))
}

/// Metadata can expose multiple commands without invoking plugin code.
#[test]
fn descriptor_lookup_and_search_fields_use_declared_static_metadata() {
    // Verify both declared identities, including the second entry missed by old fixtures.
    assert_eq!(DESCRIPTOR.command("open"), Some(COMMANDS[0]));
    assert_eq!(DESCRIPTOR.command("inspect"), Some(COMMANDS[1]));
    assert_eq!(DESCRIPTOR.command("missing"), None);

    // Search ordering preserves code, visible labels, and plugin-supplied keywords.
    let fields = DESCRIPTOR.search_fields(COMMANDS[0]).collect::<Vec<_>>();
    assert_eq!(
        fields,
        [
            "ex",
            "Example",
            "Open a foreground session",
            "sample",
            "open"
        ]
    );
}

/// Existing root imports, handler aliases, const registration, and dormant targets remain usable.
#[test]
fn public_api_supports_static_registration_and_resume_targets() {
    // A function pointer and const constructor exercise the original integration shape.
    const HANDLER: PluginActivationHandler = complete_inline;
    const REGISTRATION: PluginRegistration = PluginRegistration::new(&DESCRIPTOR, HANDLER);
    let target = PluginCommandTarget {
        registration: REGISTRATION,
        command_id: "inspect",
    };

    // Copying a target preserves only metadata; it does not create an active session.
    assert_eq!(target.command_id, "inspect");
    assert!(std::ptr::eq(REGISTRATION.descriptor(), &DESCRIPTOR));
    assert_eq!(target.registration, REGISTRATION);
}

/// Host mutation has completed before successful activation returns to its caller.
#[test]
fn completed_activation_dispatches_once_on_the_current_thread() {
    // Start with isolated state so a handler call is observable without global counters.
    let mut host = FakeHost::default();
    let registration = PluginRegistration::new(&DESCRIPTOR, complete_inline);
    let caller_thread = std::thread::current().id();

    // The synchronous result and immediately visible write jointly prove inline dispatch.
    let result = registration.activate(&mut host, &invocation("inspect"));
    assert!(matches!(result, Ok(PluginActivation::Completed)));
    assert_eq!(host.write_attempts, 1);
    assert_eq!(host.last_write_thread, Some(caller_thread));
    assert_eq!(host.clipboard.as_deref(), Some("synthetic query"));
    assert!(host.mounted_view.is_none());
}

/// An undeclared command cannot enter a handler or mutate Host capabilities.
#[test]
fn unknown_command_is_rejected_before_handler_side_effects() {
    // Preserve a known previous value to detect accidental dispatch as well as error classification.
    let mut host = FakeHost {
        clipboard: Some("previous value".to_owned()),
        ..FakeHost::default()
    };
    let registration = PluginRegistration::new(&DESCRIPTOR, complete_inline);

    // Unknown identity fails before the handler, leaving all observable Host state unchanged.
    let error = registration
        .activate(&mut host, &invocation("missing"))
        .expect_err("unknown command must be rejected");
    assert_eq!(error.code(), "command_not_found");
    assert_eq!(error.to_string(), "找不到插件命令");
    assert_eq!(host.write_attempts, 0);
    assert_eq!(host.clipboard.as_deref(), Some("previous value"));
}

/// Neither an older nor a newer protocol may execute its handler.
#[test]
fn exact_protocol_rejection_precedes_command_validation_and_dispatch() {
    // The package intentionally exposes one breaking contract rather than accepting protocol 1 alongside it.
    assert_eq!(PLUGIN_PROTOCOL_VERSION, 2);
    assert_eq!(env!("CARGO_PKG_VERSION"), "2.0.0");
    // Both directions exercise exact equality; each iteration starts with an untouched Host.
    for descriptor in [&OLDER_DESCRIPTOR, &NEWER_DESCRIPTOR] {
        let mut host = FakeHost::default();
        let registration = PluginRegistration::new(descriptor, complete_inline);

        // A missing command also proves version rejection is the first observable guard.
        let error = registration
            .activate(&mut host, &invocation("missing"))
            .expect_err("incompatible protocol must be rejected");
        assert_eq!(error.code(), "protocol_version_mismatch");
        assert_eq!(error.to_string(), "插件协议版本与宿主不一致");
        assert_eq!(host.write_attempts, 0);
        assert!(host.clipboard.is_none());
    }
}

/// Handler errors keep their stable code/message and preserve failed-operation state.
#[test]
fn handler_failure_is_propagated_without_reclassification() {
    // The fake refuses explicit clipboard writes while retaining a previous synthetic value.
    let mut host = FakeHost {
        clipboard: Some("previous value".to_owned()),
        reject_writes: true,
        ..FakeHost::default()
    };
    let registration = PluginRegistration::new(&DESCRIPTOR, complete_inline);

    // Unlike validation failure, this failure occurs after exactly one handler-side attempt.
    let error = registration
        .activate(&mut host, &invocation("inspect"))
        .expect_err("handler failure must reach the caller");
    assert_eq!(error.code(), "clipboard_unavailable");
    assert_eq!(error.to_string(), "fixture refusal");
    assert_eq!(host.write_attempts, 1);
    assert_eq!(host.clipboard.as_deref(), Some("previous value"));
}

/// The Host-owned session survives a close request and releases its Rc only when dropped.
#[test]
fn session_transfer_and_both_close_intents_preserve_owner_release_order() {
    // Exercise both successors without inventing native window/deallocation evidence.
    for intent in [
        PluginCloseIntent::DismissAndResume,
        PluginCloseIntent::ExitToLauncher,
    ] {
        // Both ordinary successors require the same complete native release scope.
        let request = PluginCloseRequest {
            intent,
            scope: PluginCloseScope::ForegroundTransition,
        };
        let mut host = FakeHost::default();
        let registration = PluginRegistration::new(&DESCRIPTOR, open_session);
        let activation = registration
            .activate(&mut host, &invocation("open"))
            .expect("synthetic session activates");
        let mut session = match activation {
            // This handler must transfer its explicit foreground owner to the caller.
            PluginActivation::Session(session) => session,
            // A completed result would lose the resource owner this test requires.
            PluginActivation::Completed => panic!("session handler completed without an owner"),
        };

        // Host and session share the callback owner; a request alone cannot release it.
        assert_eq!(&*host.lifecycle.events.borrow(), &["created"]);
        assert_eq!(Rc::strong_count(&host.lifecycle), 2);
        assert!(session.native_view().is_none());
        session
            .request_close(request)
            .expect("close intent accepted");
        assert_eq!(host.lifecycle.close_request.get(), Some(request));
        assert_eq!(Rc::strong_count(&host.lifecycle), 2);

        // The fake Host signals a synthetic barrier before dropping its sole session Box.
        host.lifecycle.finish_close(request);
        assert_eq!(
            &*host.lifecycle.events.borrow(),
            &["created", "closing", "finished"]
        );
        drop(session);
        assert_eq!(Rc::strong_count(&host.lifecycle), 1);
    }
}

/// Default session methods preserve their established viewless/unsupported behavior.
#[test]
fn default_session_remains_viewless_and_refuses_unimplemented_close() {
    // A default session must not manufacture a view or silently pretend a close succeeded.
    let mut session = ViewlessSession;
    assert!(session.native_view().is_none());
    for scope in [
        PluginCloseScope::ForegroundTransition,
        PluginCloseScope::ApplicationTermination,
    ] {
        // Explicit refusal applies equally to both scopes; neither invents unsupported completion.
        let error = session
            .request_close(PluginCloseRequest {
                intent: PluginCloseIntent::DismissAndResume,
                scope,
            })
            .expect_err("default close is unsupported");
        assert_eq!(error.code(), "close_unsupported");
        assert_eq!(error.to_string(), "插件 session 不支持关闭请求");
    }
}

/// The invocation retains caller-owned text/path borrows rather than copying their content.
#[test]
fn invocation_preserves_borrowed_inputs_and_modifiers() {
    // Owned fixture values outlive the invocation and do not access any real filesystem path.
    let query = String::from("borrowed query");
    let selected = String::from("selected text");
    let path = PathBuf::from("synthetic/note.md");
    let invocation = PluginInvocation {
        command_id: "open",
        query: &query,
        selected_text: Some(&selected),
        selected_path: Some(&path),
        modifiers: ModifierFlags {
            command: true,
            shift: false,
            option: true,
            control: false,
        },
    };

    // Pointer identity verifies borrowing; value checks retain the exact invocation semantics.
    assert_eq!(invocation.query.as_ptr(), query.as_ptr());
    assert!(std::ptr::eq(
        invocation
            .selected_text
            .expect("selected fixture exists")
            .as_ptr(),
        selected.as_ptr()
    ));
    assert!(std::ptr::eq(
        invocation.selected_path.expect("path fixture exists"),
        path.as_path()
    ));
    assert!(invocation.modifiers.command);
    assert!(invocation.modifiers.option);
    assert!(!invocation.modifiers.shift);
    assert!(!invocation.modifiers.control);
}

/// A null view is rejected without constructing a fake native object.
#[test]
fn null_native_view_is_rejected_without_dereferencing() {
    // SAFETY: the constructor explicitly accepts null and returns None without native access.
    let handle = unsafe { PluginNativeView::from_raw(std::ptr::null_mut()) };
    assert!(handle.is_none());
}

/// Existing registration equality is descriptor-based, not function-pointer based.
#[test]
fn registration_equality_and_debug_keep_handler_and_session_state_opaque() {
    // Distinct handlers sharing metadata preserve the original equality semantics.
    let inline = PluginRegistration::new(&DESCRIPTOR, complete_inline);
    let session = PluginRegistration::new(&DESCRIPTOR, open_session);
    assert_eq!(inline, session);
    assert!(format!("{inline:?}").contains(DESCRIPTOR.plugin_id));

    // Outcome formatting does not require Debug on a plugin's private resource graph.
    assert_eq!(format!("{:?}", PluginActivation::Completed), "Completed");
    assert_eq!(
        format!("{:?}", PluginActivation::Session(Box::new(ViewlessSession))),
        "Session(..)"
    );
}

/// Host path and clipboard capabilities stay independent from an active native surface.
#[test]
fn host_contract_can_be_implemented_without_private_host_modules() {
    // A consumer implements only SDK traits; the SDK does not pull in a Host/plugin crate.
    let mut host = FakeHost::default();
    assert_eq!(host.workspace_root(), Path::new("/synthetic/workspace"));
    assert_eq!(
        host.application_data_root(),
        Path::new("/synthetic/application-data")
    );
    assert_eq!(host.read_clipboard_text(), Ok(None));

    // Explicit writes/reads use fixture ownership and leave no mounted native borrow.
    host.write_clipboard_text("synthetic value")
        .expect("write fixture");
    assert_eq!(
        host.read_clipboard_text(),
        Ok(Some("synthetic value".to_owned()))
    );
    assert!(host.mounted_view.is_none());
}
