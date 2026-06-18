// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 services/
// Module declarations for lilium-services. Each module corresponds to a Python service.

pub mod account;
pub mod event;
pub mod explore;
pub mod explore_content;
pub mod history;
pub mod media;
pub mod message;
pub mod notification;
pub mod outgoing_command;
pub mod room;
pub mod room_member;
pub mod sync;
pub mod user;
pub mod websocket_connection;

pub type Result<T> = std::result::Result<T, lilium_common::LiliumError>;
