//! Data models + enums.

pub mod conversation;
pub mod enums;
pub mod label;
pub mod message;

pub use conversation::{Conversation, ConversationLabel};
pub use enums::MimeType;
pub use label::Label;
pub use message::{Attachment, Message, MessageMetadata, Recipient};
