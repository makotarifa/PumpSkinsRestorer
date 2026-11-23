//! Login handler: the only point where a skin can be applied reliably.
//!
//! `PlayerLoginEvent` fires before Pumpkin builds the player-info packet, so
//! `Player::set_skin` is already part of the joining client's first view.
//! Do not respawn the player afterwards to refresh the skin: during join an
//! extra teleport makes the client abort with "Wrong teleport id".

use pumpkin_plugin_api::{
    events::{EventData, EventHandler, EventPriority, PlayerLoginEvent},
    Context, Result, Server,
};
use tracing::{info, warn};

use crate::mojang::fetch_skin;

pub struct LoginHandler;

impl EventHandler<PlayerLoginEvent> for LoginHandler {
    fn handle(
        &self,
        _server: Server,
        event: EventData<PlayerLoginEvent>,
    ) -> EventData<PlayerLoginEvent> {
        let name = event.player.get_name();
        let skin = match fetch_skin(&name) {
            Ok(Some(skin)) => skin,
            Ok(None) => {
                warn!("PumpSkinsRestorer: {name} has no Mojang skin; keeping the default");
                return event;
            }
            Err(error) => {
                warn!("PumpSkinsRestorer: could not fetch the skin for {name}: {error}");
                return event;
            }
        };

        event.player.set_skin(&skin);
        info!("PumpSkinsRestorer: skin queued for {name}");
        event
    }
}

pub fn register(context: &Context) -> Result<()> {
    context.register_event_handler(LoginHandler, EventPriority::Normal, false)?;
    Ok(())
}
