use crate::game::net::server::Token;
use bevy::prelude::*;
use std::fmt;
use std::io;
use std::time::Instant;
use strum_macros::FromRepr;

const MAX_PAYLOAD_SIZE: u8 = 8 + 1; // inclusive of delimiter
const PACKET_DELIMITER: u8 = 0x3b; // ;

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
    pub fn to_3d(&self) -> Vec3 {
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

/// Attempts to read a valid size header out of `stream` and returns the size of the rest of the
/// packet including its delimiter. Errors `MessageSize` if the header is invalid, or io errors.
pub(super) fn read_packet_header(stream: &mut impl io::Read) -> Result<usize, InvalidPacketError> {
    let mut header = [0_u8; 1];
    stream.read_exact(&mut header)?;
    let to_read = u8::from_le_bytes(header);
    if !(1..=MAX_PAYLOAD_SIZE).contains(&to_read) {
        return Err(InvalidPacketError::MessageSize(to_read.into()));
    }
    Ok(to_read.into())
}

/// Attempt to read a packet from the stream into `buf`, returning the contents up to and not including the
/// delimiter. Errors `BadDelimiter` or io errors. Will panic if buffer is empty.
pub(super) fn read_packet_contents<'a>(
    stream: &mut impl io::Read,
    buf: &'a mut [u8],
) -> Result<&'a [u8], InvalidPacketError> {
    stream.read_exact(buf)?;
    let delimiter = *(buf.last().unwrap());
    if delimiter != PACKET_DELIMITER {
        return Err(InvalidPacketError::BadDelimiter(delimiter));
    }
    Ok(&buf[0..buf.len() - 1])
}

#[derive(FromRepr, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub(super) enum PacketCommand {
    Move,
}

#[derive(Debug)]
pub enum InvalidPacketError {
    IoError(io::Error),
    MessageSize(usize),
    BadDelimiter(u8),
    InvalidPayload,
    BadMoveDirection,
}

impl From<io::Error> for InvalidPacketError {
    fn from(value: io::Error) -> Self {
        InvalidPacketError::IoError(value)
    }
}

impl fmt::Display for InvalidPacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidPacketError::IoError(err) => {
                write!(f, "io error: {}", err)
            }
            InvalidPacketError::MessageSize(size) => {
                write!(f, "invalid message size {}", size)
            }
            InvalidPacketError::BadDelimiter(dl) => {
                write!(
                    f,
                    "bad delimiter {:#x}, expected {:#x}",
                    dl, PACKET_DELIMITER
                )
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_create_inc_packet() {
        let payload = 150000_u32.to_le_bytes().to_vec();
        let command = 0_u8;
        let stamp = 200_u8;

        let mut data: Vec<u8> = vec![command, stamp];
        data.append(&mut payload.clone());

        let packet: IncomingPacket = (&data[..]).try_into().unwrap();
        assert_eq!(packet.command, PacketCommand::from_repr(command).unwrap());
        assert_eq!(packet.stamp, stamp);
        assert_eq!(packet.payload, payload);
        println!("{:?}\n{:?}", payload, packet.payload);
    }

    #[test]
    fn test_packet_bad_data() {
        let payload = 0_u32.to_le_bytes().to_vec();
        let command = u8::MAX.to_le_bytes().to_vec();

        let mut data: Vec<u8> = vec![];
        data.append(&mut command.clone());
        data.append(&mut payload.clone());

        assert!(TryInto::<IncomingPacket>::try_into(&data[..]).is_err());
    }

    struct FakeReader {
        payload: Vec<u8>,
        read: usize,
    }

    impl Read for FakeReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let mut i = 0;
            while i < buf.len() {
                buf[i] = self.payload[self.read];
                println!("write: {} ({})", buf[i], i);
                self.read += 1;
                i += 1;
            }
            Ok(i)
        }
    }

    #[test]
    fn test_valid_read() {
        let mut message = [0_u8; MAX_PAYLOAD_SIZE as usize + 1]; //+1 for the header
        for p in 1..(message.len() - 1) {
            message[p] = 10_u8;
        }
        // correct header
        *message.first_mut().unwrap() = (message.len() - 1) as u8;
        // correct delimiter
        *message.last_mut().unwrap() = PACKET_DELIMITER;
        let mut reader = FakeReader {
            payload: message.to_vec(),
            read: 0,
        };
        let size = read_packet_header(&mut reader).unwrap();
        let mut buf = vec![0_u8; size];
        let response = read_packet_contents(&mut reader, &mut buf).unwrap();
        // everything but the header & the delimiter
        assert_eq!(response, (message[1..message.len() - 1]).to_vec());
    }

    #[test]
    fn test_bad_header() {
        let mut message = [0_u8; 1000];
        message[0] = u8::MAX;
        let res = read_packet_header(&mut FakeReader {
            payload: message.to_vec(),
            read: 0,
        });
        assert!(res.is_err());
    }
}
