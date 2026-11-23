# PumpSkinsRestorer

A WASM plugin for Pumpkin that restores Mojang skins on servers running with `online_mode = false`. Pumpkin cannot verify Java accounts in offline mode, so it hands every player the default skin. PumpSkinsRestorer looks the player's name up in Mojang's public APIs during login and queues the signed skin texture before Pumpkin builds the player-info packet.

## Compatibility

- Server: Pumpkin `0.2.2+26.3-26.60` (Java protocol 777 / Bedrock 1.26.60).
- API: WIT bindings `pumpkin:plugin@0.1.0` (WIT v0.1), WASI `p2`.
- Crate: `pumpkin-plugin-api 0.2.1`, `waki 0.5.1`, `serde_json 1`.
- Permission: `http.outbound` (declared as `HTTP_OUTBOUND` in `PluginMetadata`).
- Plugin name: `offline_skins`; the installed file is `plugins/PumpSkinsRestorer.wasm`.

## How it works

1. Pumpkin fires `PlayerLoginEvent` after authentication and **before** the player-info packet is assembled. PumpSkinsRestorer registers a handler for it at `EventPriority::Normal`.
2. The handler reads the joining player's name and resolves it with two Mojang requests:
   - `https://api.mojang.com/users/profiles/minecraft/{name}` for the Mojang UUID;
   - `https://sessionserver.mojang.com/session/minecraft/profile/{uuid}?unsigned=false` for the profile properties.
3. The `textures` property's `value` and `signature` are parsed into a `PlayerSkin` and passed to `player.set_skin(&skin)`.
4. Because the login event still precedes the player-info packet, Pumpkin sends that profile to the client as part of the normal join. There is **no `respawn()`, no second join, and no teleport**: the old respawn workaround caused `Wrong teleport id` errors during join, so it was removed deliberately.

If the name has no Mojang profile, the response contains no `textures` property, or either request fails, the handler logs a warning and leaves the default skin in place. Login is never aborted, and the server never receives a half-applied profile.

The two requests each use a five-second connect timeout. Both the HTTP status and the JSON body are checked; a non-200 response or malformed JSON is an error, not a silent empty skin.

## Code structure

```text
src/
  lib.rs      // module declarations + register_plugin!
  plugin.rs   // Plugin impl: metadata, HTTP permission, on_load handler registration
  events.rs   // LoginHandler: name -> fetch_skin -> set_skin
  mojang.rs   // waki HTTP client, Mojang URLs, two-step fetch_skin
  skin.rs     // pure parse_skin(&Value) -> Option<PlayerSkin> + unit tests
```

`skin.rs` contains no HTTP or Pumpkin state: it only interprets a Mojang profile document, so it is covered by host-side unit tests with `cargo test --lib`. Coding conventions and the review checklist are in [`SKILLS.md`](SKILLS.md).

## Building

```sh
rustup toolchain install stable
rustup target add wasm32-wasip2
cd PumpSkinsRestorer

cargo test --lib
cargo check --target wasm32-wasip2
cargo build --release --target wasm32-wasip2
```

The release artifact is `target/wasm32-wasip2/release/pump_skins_restorer.wasm`. The crate builds with `opt-level = "s"`, LTO, one codegen unit, and stripped symbols.

`cargo check --target wasm32-wasip2` is part of verification because the Pumpkin API, the generated WIT bindings, and `waki`'s WASI HTTP client only compile for the WASM target. `cargo test --lib` covers the pure parsing layer on the host.

## Installation

1. Build the plugin (or download the CI artifact).
2. Copy the `.wasm` into the server's `plugins/` directory and name it `PumpSkinsRestorer.wasm`:

   ```sh
   cp target/wasm32-wasip2/release/pump_skins_restorer.wasm /path/to/server/plugins/PumpSkinsRestorer.wasm
   ```

3. Make sure outbound HTTP is allowed for the plugin. If your `pumpkin.toml` restricts plugin permissions, add:

   ```toml
   [plugins.overrides.offline_skins]
   enabled = true
   allowed_permissions = ["http.outbound"]
   ```

4. Start Pumpkin and join with a name that exists in Mojang's database. The server log shows `PumpSkinsRestorer: skin queued for <name>`.

The plugin has no configuration file and stores no data.

## Cracked accounts

Offline mode has no proof that a client owns the name it sends, so skins are resolved **by username**, not by the offline UUID Pumpkin assigned.

- A cracked name that does not exist in Mojang's database keeps the default skin.
- A cracked client using someone else's premium name receives that player's skin. This is inherent to name-based lookup in offline mode, not something the plugin can verify away.
- Bedrock players are not in the Java profile API and keep the default skin.
- If a player changes their Mojang skin, they must reconnect; the skin is only fetched during login.

## Limitations

- Requires the `http.outbound` permission and working access to `api.mojang.com` and `sessionserver.mojang.com`.
- No cache: every login performs two Mojang requests. Very large servers or frequent reconnects can hit Mojang rate limits, and a slow lookup adds latency (up to two five-second connect timeouts).
- No retries or fallback skins beyond the server default.
- The signature is forwarded exactly as Mojang returns it; the plugin does not sign textures itself.
- Only the Java profile API is used. Bedrock skins and custom/offline skin files are out of scope.

## CI

`.github/workflows/ci.yml` runs on pushes to `main`, pull requests, and manual dispatch: host unit tests, WASM check, release build, `pump_skins_restorer.wasm.sha256` generation, and an `actions/upload-artifact@v4` upload of the `.wasm` and its checksum. `.github/workflows/release.yml` publishes both files to a GitHub Release on `v*` tags.

## Credits

- [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) — server and `pumpkin-plugin-api`.
- [waki](https://github.com/rust-wasi/waki) — WASI HTTP client.
- Mojang's public profile and session APIs for the signed texture data.
- [EssentialsPumpkinX](../dev/essentials-pumpkinx) — project layout, documentation style, and CI conventions used here.

## License

GPL-3.0. See [`LICENSE-GPL`](LICENSE-GPL).
