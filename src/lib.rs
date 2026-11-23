//! OfflineSkins: applies Mojang skin textures when Pumpkin runs with
//! `online_mode = false`.
//!
//! Offline logins carry no profile textures, so the plugin fetches the signed
//! textures from Mojang and hands them to Pumpkin during the login event.

mod events;
mod mojang;
mod plugin;
mod skin;

use plugin::OfflineSkins;

pumpkin_plugin_api::register_plugin!(OfflineSkins);
