use crate::game::net::server::Token;
use bevy::prelude::*;
use std::fmt;
use std::time::Instant;
use strum_macros::FromRepr;

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
            PacketCommand::Move => PacketData::Movement(MovementDirection::try_from(payload)?),
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
    Movement(MovementDirection),
    Noop,
}

/// Movement states including no movement, going clockwise from forward.
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromRepr, Hash)]
#[repr(u8)]
pub enum MovementDirection {
    Still = 0,
    Forward,
    Right,
    Backward,
    Left,
}

impl MovementDirection {
    /// Convert a movement direction to a direction in -z forward y up 3D space.
    pub fn to_3d(self) -> Vec3 {
        match &self {
            MovementDirection::Still => Vec3::ZERO,
            MovementDirection::Forward => Vec3::NEG_Z,
            MovementDirection::Right => Vec3::X,
            MovementDirection::Backward => Vec3::Z,
            MovementDirection::Left => Vec3::NEG_X,
        }
    }
}

impl TryFrom<&[u8]> for MovementDirection {
    type Error = InvalidPacketError;
    /// Produce a movement direction from a payload.
    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.len() != 1 {
            return Err(InvalidPacketError::BadMoveDirection);
        }
        if let Some(dir) = MovementDirection::from_repr(u8::from_le_bytes([payload[0]])) {
            Ok(dir)
        } else {
            Err(InvalidPacketError::BadMoveDirection)
        }
    }
}

#[derive(FromRepr, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub(super) enum PacketCommand {
    Move,
}

#[derive(Debug)]
pub enum InvalidPacketError {
    InvalidPayload,
    BadMoveDirection,
}

impl fmt::Display for InvalidPacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidPacketError::InvalidPayload => {
                write!(f, "bad payload formatting")
            }
            InvalidPacketError::BadMoveDirection => {
                write!(f, "bad movement direction")
            }
        }
    }
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
