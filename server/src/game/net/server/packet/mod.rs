use crate::game::net::server::Token;
use std::fmt;
use std::time::Instant;
use strum_macros::FromRepr;

#[derive(Debug)]
pub enum InvalidPacketError {
    InvalidPayload,
    ParseError(lib_spells::net::ParseError),
}

impl fmt::Display for InvalidPacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidPacketError::InvalidPayload => {
                write!(f, "bad payload formatting")
            }
            InvalidPacketError::ParseError(err) => {
                write!(f, "packet parse error: {}", err)
            }
        }
    }
}

impl From<lib_spells::net::ParseError> for InvalidPacketError {
    fn from(value: lib_spells::net::ParseError) -> Self {
        Self::ParseError(value)
    }
}

/// Higher level packet of input from a client
#[derive(Debug, Copy, Clone)]
pub struct Packet {
    pub token: Token,
    pub timestamp: Instant,
    pub data: PacketData,
}

impl Packet {
    pub(super) fn from_incoming(
        token: Token,
        incoming: IncomingPacket,
    ) -> Result<Packet, InvalidPacketError> {
        let payload = &incoming.payload[..];
        let data = match incoming.command {
            PacketCommand::Move => PacketData::Movement(lib_spells::net::MovementDirection::try_from(payload)?),
        };
        Ok(Packet {
            timestamp: incoming.timestamp,
            token,
            data,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PacketData {
    Movement(lib_spells::net::MovementDirection),
    Noop,
}

#[derive(FromRepr, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub(super) enum PacketCommand {
    Move,
}

// Remove and parse a `PacketCommand` from the start of `data`, returns the rest of the data
fn pull_command_from_data(data: &[u8]) -> Option<(PacketCommand, &[u8])> {
    if let Some((cmd_bytes, rest)) = data.split_first() {
        let command = u8::from_le_bytes([*cmd_bytes]);
        Some((PacketCommand::from_repr(command)?, rest))
    } else {
        None
    }
}

// Remove and parse a client stamp from the start of `data`
fn pull_stamp_from_data(data: &[u8]) -> Option<(u8, &[u8])> {
    if let Some((stamp_bytes, rest)) = data.split_first() {
        Some((u8::from_le_bytes([*stamp_bytes]), rest))
    } else {
        None
    }
}

/// Lower level incoming payload
#[derive(Debug)]
pub(super) struct IncomingPacket {
    pub timestamp: Instant,
    pub command: PacketCommand,
    pub stamp: u8,
    pub payload: Vec<u8>,
}

impl TryFrom<&[u8]> for IncomingPacket {
    type Error = InvalidPacketError;
    fn try_from(data: &[u8]) -> Result<IncomingPacket, InvalidPacketError> {
        if let Some((command, rest)) = pull_command_from_data(data) {
            if let Some((stamp, rest)) = pull_stamp_from_data(rest) {
                return Ok(IncomingPacket {
                    timestamp: Instant::now(),
                    command,
                    stamp,
                    payload: rest.to_vec(),
                });
            }
        }
        Err(InvalidPacketError::InvalidPayload)
    }
}
