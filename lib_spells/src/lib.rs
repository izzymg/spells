pub const SERVER_HEADER: &[u8] = "SPELLSERVER 0.1\n".as_bytes();

pub mod message_stream;
pub mod alignment;
pub mod net;
pub mod shared;
pub mod tcp_stream;
