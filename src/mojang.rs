//! Mojang profile lookups over HTTPS.
//!
//! Offline mode supplies no textures, so the signed `textures` property is
//! fetched directly from Mojang rather than trusting the login name.

use std::time::Duration;

use pumpkin_plugin_api::player::PlayerSkin;
use serde_json::Value;
use waki::Client;

use crate::skin::parse_skin;

const PROFILE_BY_NAME: &str = "https://api.mojang.com/users/profiles/minecraft";
const PROFILE_BY_UUID: &str = "https://sessionserver.mojang.com/session/minecraft/profile";
const TIMEOUT: Duration = Duration::from_secs(5);

pub fn fetch_skin(name: &str) -> Result<Option<PlayerSkin>, String> {
    let lookup = get_json(&format!("{PROFILE_BY_NAME}/{name}"))?;
    let uuid = lookup["id"].as_str().ok_or("response has no id")?;

    // `unsigned=false` is required: Pumpkin rejects textures without the
    // Mojang signature.
    let profile = get_json(&format!("{PROFILE_BY_UUID}/{uuid}?unsigned=false"))?;
    Ok(parse_skin(&profile))
}

fn get_json(url: &str) -> Result<Value, String> {
    let response = Client::new()
        .get(url)
        .connect_timeout(TIMEOUT)
        .send()
        .map_err(|error| format!("HTTP: {error}"))?;
    if response.status_code() != 200 {
        return Err(format!("HTTP {}", response.status_code()));
    }
    let body = response.body().map_err(|error| format!("body: {error}"))?;
    serde_json::from_slice(&body).map_err(|error| format!("JSON: {error}"))
}
