use bevy_math::prelude::*;
use std::fmt::{self, Display};
use std::mem::size_of;
use std::time::Duration;
use strum_macros::FromRepr;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Packet {
    pub timestamp: Duration,
    pub seq: u8,
    pub command_type: PacketType,
    pub command_data: PacketData,
}

impl Packet {
    fn concat_with_header(&self, payload: &[u8]) -> Vec<u8> {
        let timestamp_bytes = (self.timestamp.as_millis() as u64).to_le_bytes();
        [
            &timestamp_bytes[..],
            &[self.seq],
            &[self.command_type as u8],
            payload,
        ]
        .concat()
    }
    pub fn serialize(&self) -> Vec<u8> {
        match self.command_data {
            PacketData::Noop => self.concat_with_header(&[0]),
            PacketData::Movement(dir) => self.concat_with_header(&[dir.0]),
        }
    }

    pub fn deserialize(payload: &[u8]) -> Result<Self, InvalidPacketError> {
        // timestamp + seq + command
        let expect_bytes = size_of::<u64>() + (size_of::<u8>() * 2);
        if payload.len() < expect_bytes {
            return Err(InvalidPacketError::ParseError);
        }
        let (timestamp, rest) = payload.split_at(size_of::<u64>());
        let (seq, rest) = rest.split_at(size_of::<u8>());
        let (cmd, rest) = rest.split_at(size_of::<u8>());
        let command_type = PacketType::from_byte(cmd[0])?;
        let command_data = PacketData::parse(command_type, rest)?;

        Ok(Self {
            timestamp: Duration::from_millis(u64::from_le_bytes(timestamp.try_into().unwrap())),
            seq: seq[0],
            command_type,
            command_data,
        })
    }
}

#[derive(FromRepr, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PacketType {
    Move,
}

impl PacketType {
    /// Return the corresponding packet type given a single byte.
    pub fn from_byte(byte: u8) -> Result<Self, InvalidPacketError> {
        match PacketType::from_repr(byte) {
            Some(pt) => Ok(pt),
            None => Err(InvalidPacketError::InvalidPacketType(byte)),
        }
    }
}

#[derive(Debug)]
pub enum InvalidPacketError {
    InvalidPacketType(u8),
    ParseError,
}

impl Display for InvalidPacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidPacketError::InvalidPacketType(t) => {
                write!(f, "invalid packet type {}", t)
            }
            InvalidPacketError::ParseError => {
                write!(f, "packet parse error")
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PacketData {
    Movement(MovementDirection),
    Noop,
}

impl PacketData {
    /// Parse `payload` as `PacketData` for the associated `PacketType`
    fn parse(packet_type: PacketType, payload: &[u8]) -> Result<Self, InvalidPacketError> {
        match packet_type {
            PacketType::Move => Ok(PacketData::Movement(MovementDirection::try_from(payload)?)),
        }
    }
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MovementDirection(pub u8);
pub const MOVE_NONE: u8 = 0b00000000;
pub const MOVE_LEFT: u8 = 0b00000001;
pub const MOVE_RIGHT: u8 = 0b00000010;
pub const MOVE_UP: u8 = 0b00000100;
pub const MOVE_DOWN: u8 = 0b00001000;
pub const MOVE_FORWARD: u8 = 0b00010000;
pub const MOVE_BACKWARD: u8 = 0b00100000;

impl TryFrom<&[u8]> for MovementDirection {
    type Error = InvalidPacketError;
    /// Produce a movement direction from a payload.
    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.len() != 1 {
            return Err(InvalidPacketError::ParseError);
        }
        Ok(MovementDirection(u8::from_le_bytes([payload[0]])))
    }
}

impl From<MovementDirection> for Vec3 {
    fn from(value: MovementDirection) -> Vec3 {
        let mut vec = Vec3::ZERO;
        let dir = value.0;
        if dir & MOVE_LEFT > 0 {
            vec.x += -1.;
        }
        if dir & MOVE_RIGHT > 0 {
            vec.x += 1.;
        }
        if dir & MOVE_UP > 0 {
            vec.y += 1.;
        }
        if dir & MOVE_DOWN > 0 {
            vec.y += -1.;
        }
        if dir & MOVE_FORWARD > 0 {
            vec.z += -1.;
        }
        if dir & MOVE_BACKWARD > 0 {
            vec.z += 1.
        }
        vec
    }
}

impl From<Vec3> for MovementDirection {
    fn from(vec: Vec3) -> Self {
        if vec == Vec3::ZERO {
            return MovementDirection(MOVE_NONE);
        }
        let mut movement = 0_u8;
        if vec.x < 0.0 {
            movement |= MOVE_LEFT;
        } else if vec.x > 0.0 {
            movement |= MOVE_RIGHT;
        }

        if vec.y < 0.0 {
            movement |= MOVE_DOWN;
        } else if vec.y > 0.0 {
            movement |= MOVE_UP;
        }

        if vec.z < 0.0 {
            movement |= MOVE_FORWARD;
        } else if vec.z > 0.0 {
            movement |= MOVE_BACKWARD;
        }

        Self(movement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let packet = Packet {
            timestamp: Duration::from_millis(100),
            seq: 15,
            command_type: PacketType::Move,
            command_data: PacketData::Movement(MovementDirection(MOVE_BACKWARD)),
        };

        let serialized = packet.serialize();
        let deserialized = Packet::deserialize(&serialized).unwrap();
        assert_eq!(packet, deserialized);
        assert!(Packet::deserialize(&[0, 0, 2, 4, 0]).is_err());
    }

    #[test]
    fn test_dir_to_vec() {
        let dir = MovementDirection(MOVE_RIGHT | MOVE_UP | MOVE_DOWN | MOVE_FORWARD);
        let expect = Vec3::new(1.0, 0.0, -1.0);
        assert_eq!(Vec3::from(dir), expect);
        let dir = MovementDirection(MOVE_NONE);
        assert_eq!(Vec3::from(dir), Vec3::ZERO);
    }

    #[test]
    fn test_vec_to_dir() {
        let vec = Vec3::new(1.0, 0.0, -1.0);
        assert!(MovementDirection::from(vec).0 & MOVE_RIGHT > 0);
    }
}
