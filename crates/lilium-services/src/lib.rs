pub mod account;
pub mod event;
pub mod media;
pub mod message;
pub mod notification;
pub mod outgoing_command;
pub mod room_member;
pub mod user;
pub mod websocket_connection;

pub type Result<T> = std::result::Result<T, lilium_common::LiliumError>;
