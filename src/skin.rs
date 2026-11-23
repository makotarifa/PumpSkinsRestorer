//! Pure conversion from a Mojang profile response to [`PlayerSkin`].

use pumpkin_plugin_api::player::PlayerSkin;
use serde_json::Value;

/// Extracts the signed `textures` property from a session-server profile.
///
/// `None` means the account has no skin of its own (or the response is
/// malformed); the caller keeps the offline default in that case.
pub fn parse_skin(profile: &Value) -> Option<PlayerSkin> {
    let textures = profile["properties"]
        .as_array()?
        .iter()
        .find(|property| property["name"] == "textures")?;
    let value = textures["value"].as_str()?;

    Some(PlayerSkin {
        value: value.to_owned(),
        signature: textures["signature"].as_str().map(str::to_owned),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_skin;
    use serde_json::json;

    #[test]
    fn extracts_texture_value_and_signature() {
        let profile = json!({
            "properties": [
                { "name": "textures", "value": "base64-texture", "signature": "signed" }
            ]
        });

        let skin = parse_skin(&profile).expect("textures property is present");
        assert_eq!(skin.value, "base64-texture");
        assert_eq!(skin.signature.as_deref(), Some("signed"));
    }

    #[test]
    fn ignores_properties_without_textures() {
        let profile = json!({
            "properties": [
                { "name": "other", "value": "irrelevant" }
            ]
        });

        assert!(parse_skin(&profile).is_none());
    }

    #[test]
    fn ignores_profile_without_properties() {
        let profile = json!({ "id": "069a79f444e94726a5befca90e38aaf5" });

        assert!(parse_skin(&profile).is_none());
    }
}
