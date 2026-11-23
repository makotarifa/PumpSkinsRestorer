//! Plugin entry point and metadata.

use pumpkin_plugin_api::{permissions::HTTP_OUTBOUND, Context, Plugin, PluginMetadata, Result};
use tracing::info;

use crate::events;

pub struct PumpSkinsRestorer;

impl Plugin for PumpSkinsRestorer {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "offline_skins".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["makyg".into()],
            description: "PumpSkinsRestorer: applies Mojang skins with online_mode=false.".into(),
            dependencies: vec![],
            permissions: vec![HTTP_OUTBOUND.into()],
        }
    }

    fn on_load(&self, context: Context) -> Result<()> {
        events::register(&context)?;
        info!("PumpSkinsRestorer: plugin loaded");
        Ok(())
    }
}
