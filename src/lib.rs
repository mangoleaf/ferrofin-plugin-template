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
    /// Your plugin's identity. Called once at load. All values come from
    /// `[package.metadata.ferrofin]` in Cargo.toml (via build.rs) — edit them
    /// THERE, so the manifest generator and the runtime can never disagree.
    fn descriptor() -> PluginDescriptor {
        PluginDescriptor {
            id: env!("FERROFIN_PLUGIN_GUID").to_owned(),
            name: env!("FERROFIN_PLUGIN_NAME").to_owned(),
            // The crate version from Cargo.toml — bump it there to release.
            version: env!("CARGO_PKG_VERSION").to_owned(),
            description: env!("FERROFIN_PLUGIN_DESCRIPTION").to_owned(),
        }
    }

    /// The configuration JSON written on first install. The admin edits it in
    /// the dashboard (through your settings page below); you read it back
    /// with `host::get_config()`. Return `{}` if your plugin needs no
    /// configuration.
    fn default_config() -> String {
        r#"{"Greeting":"Hello from my plugin"}"#.to_owned()
    }

    /// Your dashboard settings page(s). This worked example edits the
    /// `Greeting` value from `default_config()` — the full round trip the
    /// dashboard uses: load with `ApiClient.getPluginConfiguration`, save
    /// with `ApiClient.updatePluginConfiguration` (which lands in
    /// `host::get_config()` on your side).
    ///
    /// The shape is the standard jellyfin-web plugin page: a
    /// `data-role="page"` root plus an inline script that may use the
    /// dashboard globals (`ApiClient`, `Dashboard`). Return `vec![]` to ship
    /// no page — Ferrofin then shows a generic JSON editor for your config
    /// instead, so your plugin stays configurable either way.
    fn config_pages() -> Vec<ConfigPage> {
        // Your plugin id, single-sourced from Cargo.toml like descriptor().
        let id = env!("FERROFIN_PLUGIN_GUID");
        let html = format!(
            r#"<div id="myPluginConfig" data-role="page" class="page type-interior pluginConfigurationPage">
  <div data-role="content"><div class="content-primary">
    <form class="myPluginForm">
      <h1>{name}</h1>
      <div class="inputContainer">
        <label class="inputLabel" for="myPluginGreeting">Greeting</label>
        <input is="emby-input" id="myPluginGreeting" type="text" />
        <div class="fieldDescription">Logged by the "Say hello" task.</div>
      </div>
      <button is="emby-button" type="submit" class="raised button-submit block"><span>Save</span></button>
    </form>
  </div></div>
  <script type="text/javascript">
  (function () {{
    var pluginId = '{id}';
    var page = document.querySelector('#myPluginConfig');
    page.addEventListener('pageshow', function () {{
      Dashboard.showLoadingMsg();
      ApiClient.getPluginConfiguration(pluginId).then(function (config) {{
        page.querySelector('#myPluginGreeting').value = config.Greeting || '';
        Dashboard.hideLoadingMsg();
      }}).catch(Dashboard.processErrorResponse);
    }});
    page.querySelector('.myPluginForm').addEventListener('submit', function (e) {{
      e.preventDefault();
      Dashboard.showLoadingMsg();
      ApiClient.getPluginConfiguration(pluginId).then(function (config) {{
        config.Greeting = page.querySelector('#myPluginGreeting').value;
        ApiClient.updatePluginConfiguration(pluginId, config).then(
          Dashboard.processPluginConfigurationUpdateResult
        ).catch(Dashboard.processErrorResponse);
      }}).catch(Dashboard.processErrorResponse);
      return false;
    }});
  }})();
  </script>
</div>
"#,
            name = env!("FERROFIN_PLUGIN_NAME"),
        );
        vec![ConfigPage {
            // Unique server-wide; the dashboard fetches your page by this.
            name: "my-plugin-config".to_owned(),
            content: html.into_bytes(),
            enable_in_main_menu: false,
        }]
    }

    /// Your plugin's public-egress allowlist — single-sourced from the
    /// `egress = [...]` array in Cargo.toml (visible to anyone auditing
    /// your repo) and ENFORCED by the server: http_fetch to any host not
    /// listed is refused before DNS. Empty = no internet access.
    fn declared_egress() -> Vec<String> {
        env!("FERROFIN_PLUGIN_EGRESS")
            .split(',')
            .filter(|e| !e.is_empty())
            .map(str::to_owned)
            .collect()
    }

    /// Your identity as a NAMED metadata provider (shown in the
    /// dashboard's library fetcher lists). `None` unless your plugin
    /// contributes metadata via `metadata_lookup`.
    fn provider_info() -> Option<ProviderDescriptor> {
        None
    }

    /// The item kinds your plugin ANALYZES ("Movie", "Episode", "Audio",
    /// ...). Return them and the server offers each new matching item to
    /// `scan_media` (once per item, host-tracked) — pull decoded data with
    /// `host::media_info` / `host::extract_audio` / `host::extract_frames`
    /// (the host decodes, your plugin analyzes) and persist results via
    /// `write_media_segments` / `set_state`. Empty = not an analyzer.
    fn scan_targets() -> Vec<String> {
        Vec::new()
    }

    /// Analyzes one offered library item. Never called unless
    /// `scan_targets()` is non-empty. Errors are logged and count toward
    /// your plugin's failure breaker — they never fail the library scan.
    fn scan_media(_item: ItemSummary) -> Result<(), String> {
        Ok(())
    }

    /// Web-file transformations: literal search/replace patches the server
    /// applies to served jellyfin-web files while your plugin is enabled —
    /// how a plugin injects client-side hooks (script tags, function
    /// wrappers). TRUST NOTE: this is JS injection into EVERY user's
    /// browser; most plugins should return an empty vec.
    fn web_transforms() -> Vec<WebTransform> {
        Vec::new()
    }

    /// Your plugin's own URL space: the server routes
    /// `ANY /Plugins/{your-guid}/web/*` here. Reachable WITHOUT
    /// authentication (assets load via plain <script src> tags) — the
    /// caller's resolved identity is in `request.user_id` /
    /// `request.is_admin` / `request.is_authenticated`; gate sensitive
    /// paths yourself. A plugin that serves no routes returns 404.
    fn handle_request(_request: PluginRequest) -> PluginResponse {
        PluginResponse {
            status: 404,
            headers: vec![("content-type".to_owned(), "application/json".to_owned())],
            body: br#"{"error":"this plugin serves no routes"}"#.to_vec(),
        }
    }

    /// The background tasks your plugin offers. They appear in the dashboard's
    /// "Scheduled Tasks" page and can be run on demand or on a schedule.
    /// Return an empty vec if your plugin has no tasks.
    fn tasks() -> Vec<TaskDescriptor> {
        vec![TaskDescriptor {
            id: "hello".to_owned(),
            name: "Say hello".to_owned(),
            description: "Writes a greeting to the server log.".to_owned(),
            // The dashboard groups tasks under this header — the plugin's
            // name (from Cargo.toml) is the right default; override it only
            // if you want your tasks grouped under something else.
            category: env!("FERROFIN_PLUGIN_NAME").to_owned(),
        }]
    }

    /// Runs one of the tasks you advertised in `tasks()`. Return `Err(..)` to
    /// have the run recorded as failed (the string is logged).
    fn run_task(task_id: String) -> Result<(), String> {
        match task_id.as_str() {
            "hello" => {
                // Read the greeting the admin saved on your settings page.
                // (Crude string scan to keep the skeleton dependency-free —
                // add serde_json to Cargo.toml for real config parsing.)
                let config = host::get_config();
                let greeting = config
                    .split_once("\"Greeting\":\"")
                    .and_then(|(_, rest)| rest.split_once('"'))
                    .map_or("Hello from my Ferrofin plugin!", |(v, _)| v);
                host::log(LogLevel::Info, greeting);
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
    fn remote_images(_item: ItemSummary) -> Result<Vec<ImageCandidate>, String> {
        // Not an artwork provider.
        Ok(vec![])
    }

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
//   Per-plugin key/value state (per-user settings, cursors — NOT config;
//   the admin never sees it; caps: 256 B key, 1 MiB value, 8 MiB total):
//     host::set_state("cursor", Some(b"42"))?;      // None deletes
//     let v: Option<Vec<u8>> = host::get_state("cursor");
//
//   The user's next episodes to watch (Jellyfin's NextUp):
//     let queue = host::next_up(&user_id, 16)?;     // -> Vec<ItemSummary>
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
