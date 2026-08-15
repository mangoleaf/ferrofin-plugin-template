# Ferrofin plugin template

A ready-to-build skeleton for a **Ferrofin** WASM plugin. Clone it, write your
plugin's logic in `src/lib.rs`, run `./build.sh`, and drop the resulting
`.wasm` into your server. Everything else — the toolchain, the target, the
contract bindings, the build wiring — is already set up.

> **What is a Ferrofin plugin?** A sandboxed WebAssembly component the server
> loads at startup from `{data_dir}/plugins/`. It runs *inside* the server but
> with **no filesystem and no network of its own** — it can only do the small,
> explicit set of things the host grants it (log, read its config, make
> host-mediated HTTP requests, query the library, write media segments,
> contribute metadata). That list is the whole security boundary. See
> [The sandbox](#the-sandbox-what-you-can-and-cant-do) below.

---

## Quick start

You need [`rustup`](https://rustup.rs) installed. That's the only prerequisite —
the pinned toolchain and the `wasm32-wasip2` target install themselves on your
first build (see `rust-toolchain.toml`).

```sh
# 1. Clone this template (rename the directory to your plugin).
git clone <this-repo> my-ferrofin-plugin && cd my-ferrofin-plugin

# 2. Give your plugin a unique identity.
uuidgen        # copy the result…
#   …and paste it into [package.metadata.ferrofin] guid in Cargo.toml (also
#   set name/description there — descriptor() reads them at build time).
#   Rename the crate in [package] name too.

# 3. Write your plugin: edit src/lib.rs (it compiles and runs as-is to start).

# 4. Build.
./build.sh

# 5. Install: copy the artifact into your server and restart it.
cp dist/*.wasm {ferrofin_data_dir}/plugins/
#   …restart Ferrofin, then enable the plugin in Dashboard → Plugins.
```

That's the whole loop. From here you only ever touch `src/lib.rs` (and
`Cargo.toml` if you add dependencies).

---

## Releasing & distribution

When you push this repo to GitHub, the included workflows (`.github/workflows/`,
all actions pinned by commit SHA) give you the full Jellyfin-style distribution
flow: users add your repository URL to Ferrofin **once**, then install and
update your plugin from the dashboard — the server downloads, verifies, and
stages it; a restart activates it.

**Releasing** is one step: bump `version` in `Cargo.toml` (semver
`MAJOR.MINOR.PATCH`) and merge to `main`. The release workflow then:

1. builds the `.wasm` and publishes a `vX.Y.Z` GitHub Release with it attached;
2. regenerates `manifest.json` from **all** releases — for each version:
   the asset's download URL, its MD5 (`checksum`, the Jellyfin-standard field),
   its SHA-256 (`sha256`, Ferrofin's stronger extension — preferred by the
   server), and the `targetAbi` read from the vendored WIT contract;
3. publishes the manifest to the `gh-pages` branch.

Identity (guid/name/description) comes from `[package.metadata.ferrofin]` in
`Cargo.toml` — the same values `descriptor()` compiles in via `build.rs`, so
the id your plugin reports at runtime can never drift from its catalog entry
(Ferrofin verifies this at install time and refuses mismatches).

**One-time setup** after pushing to GitHub: enable Pages
(Settings → Pages → *Deploy from a branch* → `gh-pages`). Your repository URL
is then:

```
https://<owner>.github.io/<repo>/manifest.json
```

**How users install** (the primary flow — no shell access needed):

1. Ferrofin dashboard → Plugins → Repositories → add the URL above.
2. Catalog → your plugin → Install. The server downloads it over HTTPS,
   verifies the checksum, validates it is a real `ferrofin:plugin` component
   with the right ABI and identity, and stages it.
3. Restart Ferrofin (the dashboard shows the pending-restart flag). Done —
   enable/configure it under Plugins.

**Manual install** (dev workflow / air-gapped servers): build locally and drop
the artifact in yourself:

```sh
./build.sh
cp dist/*.wasm {ferrofin_data_dir}/plugins/   # then restart Ferrofin
```

## What you implement

Your plugin is the `Guest` trait in `src/lib.rs`. Every method is optional in
spirit — return the empty/`None` default for anything you don't need:

| Method | When it's called | Return when unused |
|---|---|---|
| `descriptor()` | once at load — your id, name, version | *(required)* |
| `default_config()` | first install — the seed config JSON | `"{}"` |
| `config_pages()` | at load — your dashboard settings page(s) | `vec![]` *(Ferrofin then shows a generic JSON editor for your config)* |
| `web_transforms()` | at load — patches to served jellyfin-web files (client-side hooks; **JS injection into every user's browser** — most plugins: none) | `vec![]` |
| `handle_request(req)` | per request to `/Plugins/{guid}/web/*` — your plugin's own HTTP API (**anonymous-reachable**; gate on `req.is_admin` etc.) | `404` |
| `tasks()` | at load — the dashboard tasks you offer | `vec![]` |
| `run_task(id)` | when a task runs (on demand or scheduled) | — |
| `on_event(name, json)` | on each server event while enabled | *(do nothing)* |
| `metadata_lookup(item, ids)` | per item during a library scan | `Ok(None)` |

## What you can call

The host grants your plugin exactly these — nothing else exists in the sandbox.
Full signatures and docs are in [`wit/ferrofin-plugin.wit`](wit/ferrofin-plugin.wit),
and there's a copy-paste cheat-sheet at the bottom of `src/lib.rs`.

- **`log(level, message)`** — write to the server log.
- **`get_config()`** — your persisted config JSON (the admin edits it in the
  dashboard; `{}` until they save something).
- **`http_fetch(request)`** — outbound HTTP the host performs for you (the only
  network access). Public destinations only by default; the admin can allowlist
  your plugin for private/LAN hosts.
- **`query_items(query)`** — read-only library queries (≤1000 rows per call).
- **`write_media_segments(item_id, segments)`** — persist Intro/Outro/Recap/…
  segments, scoped to your plugin.
- **`get_state(key)` / `set_state(key, value)`** — a small per-plugin key/value
  store (per-user settings, cursors; the admin never sees it).
- **`next_up(user_id, limit)`** — the user's next-episodes queue.

## The sandbox: what you can and can't do

This is the point of the WASM model, so be clear on it:

**Your plugin can never**, no matter what you write: read or write **any file**
(media, the server database, host keys — none of it), open its **own** network
connections, run host code or spawn processes, read server memory, or exceed
its memory/CPU-time limits. A crash or timeout is contained and, if repeated,
the plugin is sidelined — it can't take the server down.

**Your plugin can**, through the host functions above: read the library
**catalog** (titles, ids, paths — never file contents) and make **host-mediated
HTTP** requests. So a plugin *can* legitimately reach out to a metadata API, and
a malicious one could send your catalog somewhere — install plugins you have
reason to trust. Private/loopback/LAN addresses are refused by default; a
specific plugin gets them via the server's `FERROFIN_WASM_PRIVATE_HTTP_ALLOW`.

Practical consequences for you: **don't add crates that touch files or sockets**
— they won't link or won't work. Do your I/O through `http_fetch`. Keep handlers
quick (there's a per-call deadline). Hold state in `static`s if you must, but
treat the config JSON and the library as your only durable storage.

## Configuration & your settings page

`default_config()` returns the JSON seeded on first install; you read the
current value with `host::get_config()` (a JSON string — parse it however you
like, e.g. add `serde_json` to `Cargo.toml`). A plugin that needs no config
just returns `"{}"`.

The admin edits that JSON through **your settings page**: `config_pages()`
returns raw HTML in the standard jellyfin-web plugin-page shape (see the
worked example in `src/lib.rs` — a `data-role="page"` root plus an inline
script that loads the config with `ApiClient.getPluginConfiguration` and saves
it with `ApiClient.updatePluginConfiguration`). The dashboard shows a
**Settings** button on your plugin automatically. Ship no page and Ferrofin
synthesizes a generic JSON editor instead, so your plugin is configurable
either way.

**Trust note:** your page's HTML/JS runs in the *admin's browser* with the
admin's session — it is dashboard code, not sandboxed plugin code. (This is
exactly how Jellyfin plugin pages work.)

## Other languages

The build here is Rust (via `wit-bindgen`), which is the smoothest path. But the
contract is a language-neutral **WASM component** interface — any toolchain that
targets the WASM component model against `wit/ferrofin-plugin.wit` produces a
loadable plugin. If you go that route, this repo is still useful as the contract
+ the install instructions; swap out the Rust build for yours.

## Contract version

The vendored `wit/ferrofin-plugin.wit` is `ferrofin:plugin@0.3.0`. **It is 0.x
and unstable** — a minor bump may require rebuilding, and the server refuses to
load a component built against a different version (it names both versions in
the error). To target a newer server, replace `wit/ferrofin-plugin.wit` with
that version from the Ferrofin repo and rebuild.

## Licensing

- The **scaffolding** in this template (`src/lib.rs`, `build.sh`, config files)
  is provided under the MIT license (see [`LICENSE`](LICENSE)) — it's a starting
  point; your plugin's own code is yours to license as you choose (the
  `Cargo.toml` `license` field is a placeholder — set it).
- **`wit/ferrofin-plugin.wit`** is Ferrofin's interface definition, vendored
  here for building, and is **GPL-3.0-only** (Ferrofin is GPL-3.0-only). If you
  distribute your plugin, understand you're building against a GPL interface;
  when in doubt, consult the Ferrofin project about plugin licensing.
