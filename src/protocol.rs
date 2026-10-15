//! Immutable protocol metadata and borrowed activation inputs.

use std::path::Path;

/// 唯一接受的资源与生命周期协议；3 明确区分项目对象、受检空壳和框架借用缓存。
pub const PLUGIN_PROTOCOL_VERSION: u16 = 3;

/// One discoverable command declared in a static plugin descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginCommandDescriptor {
    /// Command identity, unique within its owning plugin.
    pub id: &'static str,
    /// Human-facing command title shown by the host.
    pub title: &'static str,
    /// Short task-oriented explanation for the host's result list.
    pub subtitle: &'static str,
    /// Additional immutable search terms owned by the plugin.
    pub keywords: &'static [&'static str],
}

/// Static discoverability metadata that requires no active plugin resources.
///
/// The Host composition validates identifier uniqueness across its registrations.
/// Constructing a descriptor does not instantiate the plugin or call its handler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginDescriptor {
    /// Globally unique identity of the plugin in the assembled catalog.
    pub plugin_id: &'static str,
    /// Required exact protocol, checked before activation dispatch.
    pub protocol_version: u16,
    /// Commands available without creating a plugin session.
    pub commands: &'static [PluginCommandDescriptor],
    /// Short discoverability code presented by the host.
    pub activation_code: &'static str,
    /// Small textual symbol available without loading native assets.
    pub symbol: &'static str,
}

impl PluginDescriptor {
    /// Find a declared command by exact identity, without normalizing or allocating.
    #[must_use]
    pub fn command(self, command_id: &str) -> Option<PluginCommandDescriptor> {
        // Borrow static entries and stop at the matching command; absence stays explicit.
        self.commands.iter().copied().find(|command| {
            // Match command identity rather than UI titles or activation-code heuristics.
            command.id == command_id
        })
    }

    /// Yield the existing static command search fields in their established order.
    ///
    /// The host owns matching/ranking. This iterator supplies metadata only and
    /// does not allocate normalized strings or inspect the filesystem.
    pub fn search_fields(
        self,
        command: PluginCommandDescriptor,
    ) -> impl Iterator<Item = &'static str> {
        // Stream the short code, visible labels, and keywords without retaining query state.
        [self.activation_code, command.title, command.subtitle]
            .into_iter()
            .chain(command.keywords.iter().copied())
    }
}

/// Modifier state captured by the host for one synchronous invocation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ModifierFlags {
    /// The Command modifier was active.
    pub command: bool,
    /// The Shift modifier was active.
    pub shift: bool,
    /// The Option modifier was active.
    pub option: bool,
    /// The Control modifier was active.
    pub control: bool,
}

/// Inputs borrowed from the caller for the duration of one synchronous activation.
///
/// A returned session must not retain references to these inputs. A plugin that
/// actually needs an input after activation must make that ownership explicit.
#[derive(Clone, Copy, Debug)]
pub struct PluginInvocation<'a> {
    /// Exact identity of the command selected from the plugin descriptor.
    pub command_id: &'a str,
    /// Current host query, borrowed without a normalized copy.
    pub query: &'a str,
    /// Optional selected text supplied by an explicit host invocation.
    pub selected_text: Option<&'a str>,
    /// Optional selected path supplied by an explicit host invocation.
    pub selected_path: Option<&'a Path>,
    /// Modifiers associated with the invocation rather than later global state.
    pub modifiers: ModifierFlags,
}
