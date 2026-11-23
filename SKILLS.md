# Coding skills and conventions

This document describes how PumpSkinsRestorer is organised and the conventions to follow when changing it. The plugin is a Rust `cdylib`/`rlib` compiled to `wasm32-wasip2` for Pumpkin 0.2.2. Its one job is narrow: resolve a Mojang skin during login and apply it without respawning the player.

## Project structure

`src/lib.rs` stays thin: it only declares the modules and invokes `pumpkin_plugin_api::register_plugin!(PumpSkinsRestorer)`. Everything else lives in one module with one responsibility.

- **`src/plugin.rs`** — the `Plugin` implementation: `metadata()` (name `offline_skins`, version from `CARGO_PKG_VERSION`, author, description, dependencies, and the `HTTP_OUTBOUND` permission) and `on_load()`, which registers the login handler at `EventPriority::Normal`.
- **`src/events.rs`** — `LoginHandler` and its `impl EventHandler<PlayerLoginEvent>`. It reads the player's name, calls `fetch_skin`, and applies the result with `player.set_skin(&skin)`. A missing skin or an HTTP error logs a warning and returns the event unchanged; login must never fail because Mojang was unreachable.
- **`src/mojang.rs`** — the only code that performs I/O: a `waki::Client` GET helper with a five-second connect timeout, a status-code check, JSON decoding, and `fetch_skin`, which performs the two-step Mojang lookup (name → UUID, UUID → signed textures). It converts the result into a `PlayerSkin`.
- **`src/skin.rs`** — the pure parser `parse_skin(&Value) -> Option<PlayerSkin>`. It finds the `textures` property in a Mojang profile document and extracts `value`/`signature`. It has no HTTP, filesystem, or Pumpkin state; its `#[cfg(test)]` unit tests run on the host with `cargo test --lib`.

Rules for changing the plugin:

1. Put network calls in `mojang.rs` and JSON interpretation in `skin.rs`. The event handler only orchestrates; it does not parse JSON.
2. Keep `lib.rs` at module declarations plus the registration macro. Do not grow it back into a single-file plugin.
3. Preserve the login-only strategy. Do not add `respawn()`, delayed ticks, teleports, or a second join attempt to refresh a skin: those caused `Wrong teleport id` errors and are not needed because the login event runs before the player-info packet.
4. Keep the declared `HTTP_OUTBOUND` permission when touching `metadata()`. Without it the WASI host denies outbound requests.
5. On any failure, keep the default skin and log the concrete reason. Never fabricate a skin and never abort login.
6. Update `README.md` when behaviour, requirements, installation, or limitations change.
7. Verify with `cargo test --lib`, `cargo check --target wasm32-wasip2`, and `cargo build --release --target wasm32-wasip2` before installing a build.

## Design patterns

- **Event handler** — a single `EventHandler<PlayerLoginEvent>` registered in `on_load`. Pumpkin calls it during the join sequence; the handler returns the event unchanged after applying the skin. There is no background loop, no tick hook, and no polling.
- **Pure core, imperative shell** — `skin.rs` is a deterministic function over a JSON document; `mojang.rs` owns the HTTP edge; `events.rs` connects the two. This keeps the parsing rules fully testable on the host without a server or network.
- **Two-step Mojang lookup** — username → Mojang UUID → signed profile. The UUID lookup is required because the profile endpoint does not accept a username. Both responses are validated before use.
- **Explicit, non-fatal failure** — `Result<Option<PlayerSkin>, String>` distinguishes "no skin exists" from "the lookup failed", and both paths degrade to the default skin with a warning.
- **Patterns NOT to use** — a respawn/teleport workaround; a global mutable cache or state without a clear owner; retries without bounds or backoff; panics/`unwrap()` on network or JSON data; fake placeholder skins; a second HTTP client or a second copy of the Mojang URLs; moving parsing into the event handler.

## Human-readable code first

- **Descriptive names** — modules, functions, and variables say what they do (`fetch_skin`, `parse_skin`, `get_json`, `LoginHandler`). Abbreviations are limited to established terms (HTTP, JSON, UUID, WIT, WASI, WASM).
- **Comments only for the *why*** — the login-ordering comment explains why `set_skin` is enough and why no respawn is used. Do not restate what the code already says.
- **Small functions** — one job per function: request/response handling, JSON parsing, orchestration. Split anything that mixes I/O with parsing.
- **Explicit errors** — return `Result` with concrete messages that include the failing operation (`HTTP: …`, `HTTP 404`, `body: …`, `JSON: …`, `response has no id`). No silent fallback and no fake success.
- **Pure tests** — cover `parse_skin` with table-driven cases: valid textures property, missing property, missing `value`, optional signature, and malformed shapes. Do not require network access in tests.
- **Review checklist** — before merging, confirm the plugin still listens to `PlayerLoginEvent`; `set_skin` is called before player-info and there is no `respawn()`/teleport; `HTTP_OUTBOUND` remains declared; parsing stays pure and tested; failures keep the default skin and log a warning; `README.md` matches the code; and all three verification commands pass.
