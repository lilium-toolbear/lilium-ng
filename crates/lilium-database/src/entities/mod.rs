pub mod dzmm_account;
pub mod event_processor_offsets;
pub mod messages;
pub mod outgoing_commands;
pub mod room_members;
pub mod rooms;
pub mod users;
pub mod websocket_connections;
pub mod websocket_events;

pub mod prelude {
    pub use super::dzmm_account::Entity as DzmmAccount;
    pub use super::event_processor_offsets::Entity as EventProcessorOffsets;
    pub use super::messages::Entity as Messages;
    pub use super::outgoing_commands::Entity as OutgoingCommands;
    pub use super::room_members::Entity as RoomMembers;
    pub use super::rooms::Entity as Rooms;
    pub use super::users::Entity as Users;
    pub use super::websocket_connections::Entity as WebsocketConnections;
    pub use super::websocket_events::Entity as WebsocketEvents;
}
