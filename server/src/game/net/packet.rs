use std::fmt;
use std::io;
use std::time::Instant;
use strum_macros::FromRepr;

const MAX_PAYLOAD_SIZE: u8 = 8 + 1; // inclusive of delimiter
const PACKET_DELIMITER: u8 = 0x3b; // ;

/// Attempts to read a valid size header out of `stream` and returns the size of the rest of the
/// packet including its delimiter. Errors `MessageSize` if the header is invalid, or io errors.
pub fn read_packet_header(stream: &mut impl io::Read) -> Result<usize, InvalidPacketError> {
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
pub fn read_packet_contents<'a>(
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
pub enum PacketCommand {
    CastSpell,
    Move,
}

#[derive(Debug)]
pub enum InvalidPacketError {
    IoError(io::Error),
    MessageSize(usize),
    BadDelimiter(u8),
    InvalidPayload,
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

// Remove and parse a client stamp from the start of`data`
fn pull_stamp_from_data(data: &[u8]) -> Option<(u8, &[u8])> {
    if let Some((stamp_bytes, rest)) = data.split_first() {
        Some((u8::from_le_bytes([*stamp_bytes]), rest))
    } else {
        None
    }
}

#[derive(Debug)]
pub struct IncomingPacket {
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
