//! Your Ferrofin plugin.
//!
//! Everything the server can ask your plugin to do is one of the `Guest`
//! methods below; everything your plugin can do to the outside world is a
//! `host::*` call. Both come from `wit/ferrofin-plugin.wit` — read it, it is
//! short and it is the whole contract.
//!
//! The skeleton below compiles and runs as-is (it registers one "hello" task
//! that writes a log line). Replace the bodies with your plugin's behavior.

wit_bindgen::generate!({
    // The vendored contract in this repo. Don't change this path.
    path: "wit",
    world: "plugin",
});

// The host functions you may call. This is the ENTIRE set — no filesystem,
// no sockets, nothing else exists inside the sandbox.
use ferrofin::plugin::host;
use ferrofin::plugin::types::LogLevel;
// `generate!` hoists the record types the world uses (PluginDescriptor,
// TaskDescriptor, ItemSummary, MetadataResult, …) to the crate root, so they
// need no `use`. The rest (HttpRequest, ItemQuery, MediaSegment) live under
// `ferrofin::plugin::types` — import them where you use them.

/// Your plugin. Hold any in-memory state on this type's statics (a plugin
/// instance lives for the whole server run). Nothing here is persisted;
/// durable state belongs in the host — your config JSON, or the library.
struct Plugin;

impl Guest for Plugin {
    /// Your plugin's identity. Called once at load.
    fn descriptor() -> PluginDescriptor {
        PluginDescriptor {
            // ⚠️ TODO: generate your OWN id and paste it here. Run:  uuidgen
            // Two plugins with the same id collide — the host loads only one.
            id: "00000000-0000-0000-0000-000000000000".to_owned(),
            name: "My Ferrofin Plugin".to_owned(),
            // Reuses the crate version from Cargo.toml — bump it there.
            version: env!("CARGO_PKG_VERSION").to_owned(),
            description: "Describe what your plugin does, in one line.".to_owned(),
        }
    }

    /// The configuration JSON written on first install. The admin edits it in
    /// the dashboard; you read it back with `host::get_config()`. Return `{}`
    /// if your plugin needs no configuration.
    fn default_config() -> String {
        r#"{}"#.to_owned()
    }

    /// The background tasks your plugin offers. They appear in the dashboard's
    /// "Scheduled Tasks" page and can be run on demand or on a schedule.
    /// Return an empty vec if your plugin has no tasks.
    fn tasks() -> Vec<TaskDescriptor> {
        vec![TaskDescriptor {
            id: "hello".to_owned(),
            name: "Say hello".to_owned(),
            description: "Writes a greeting to the server log.".to_owned(),
            category: "My Plugin".to_owned(),
        }]
    }

    /// Runs one of the tasks you advertised in `tasks()`. Return `Err(..)` to
    /// have the run recorded as failed (the string is logged).
    fn run_task(task_id: String) -> Result<(), String> {
        match task_id.as_str() {
            "hello" => {
                host::log(LogLevel::Info, "Hello from my Ferrofin plugin!");
                Ok(())
            }
            other => Err(format!("unknown task `{other}`")),
        }
    }

    /// Called for each server event while your plugin is enabled
    /// (`LibraryChanged`, `PlaybackStart`, `PlaybackStopped`, `SessionStarted`,
    /// `TaskCompleted`, …). `event_json` is the event's payload. Most plugins
    /// ignore most events — keep this fast; a slow handler makes the host drop
    /// events for you. Do nothing here if you don't need events.
    fn on_event(_event_name: String, _event_json: String) {}

    /// Offers metadata for one library item during a scan, AFTER the built-in
    /// providers ran. Return `Ok(None)` unless your plugin is a metadata
    /// source — that is the common case, and the default here. Results are
    /// applied supplement-only: you can fill fields the item still lacks, never
    /// overwrite the built-in providers or the user's edits.
    fn metadata_lookup(
        _item: ItemSummary,
        _provider_ids: Vec<(String, String)>,
    ) -> Result<Option<MetadataResult>, String> {
        Ok(None)
    }
}

// ── Capability cheat-sheet ────────────────────────────────────────────────
// Everything your plugin can do to the world, with a one-line example. Delete
// what you don't use. Full signatures + docs are in wit/ferrofin-plugin.wit.
//
//   Log:
//     host::log(LogLevel::Info, "message");
//
//   Read your config (the JSON the admin saved; `{}` until they do):
//     let cfg: String = host::get_config();
//
//   Query the library (read-only; max 1000 rows per call):
//     use ferrofin::plugin::types::ItemQuery;
//     let movies = host::query_items(&ItemQuery {
//         kinds: vec!["Movie".to_owned()],
//         parent_id: None,
//         search_term: None,
//         limit: Some(50),
//     })?; // -> Vec<ItemSummary>
//
//   Outbound HTTP (the ONLY network access; public hosts only by default —
//   the admin can allowlist your plugin id for private/LAN hosts):
//     use ferrofin::plugin::types::HttpRequest;
//     let resp = host::http_fetch(&HttpRequest {
//         method: "GET".to_owned(),
//         url: "https://api.example.com/thing".to_owned(),
//         headers: vec![],
//         body: None,
//     })?; // -> HttpResponse { status, headers, body }
//
//   Write media segments (Intro/Outro/Recap/Preview/Commercial). Scoped to
//   your plugin — you can never touch another provider's or a user's segments.
//   An empty list erases yours for that item. Ticks are 100ns units.
//     use ferrofin::plugin::types::MediaSegment;
//     host::write_media_segments(&item_id, &[MediaSegment {
//         segment_type: "Intro".to_owned(),
//         start_ticks: 0,
//         end_ticks: 30 * 10_000_000, // 30 seconds
//     }])?;
// ──────────────────────────────────────────────────────────────────────────

// Wires `Plugin` up as the component's exports. Keep this as the last line.
export!(Plugin);
