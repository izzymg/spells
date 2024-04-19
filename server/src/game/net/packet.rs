use std::fmt;
use std::time::Instant;
use strum_macros::FromRepr;
#[derive(FromRepr, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PacketCommand {
    CastSpell,
    Velocity,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct InvalidPacketError;

impl fmt::Display for InvalidPacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid packet")
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

pub(super) struct IncomingPacket {
    pub timestamp: Instant,
    pub command: PacketCommand,
    pub stamp: u8,
    pub payload: Vec<u8>,
}

impl IncomingPacket {
    pub(super) fn from_bytes(data: &[u8]) -> Result<IncomingPacket, InvalidPacketError> {
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
        Err(InvalidPacketError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_inc_packet() {
        let payload = 150000_u32.to_le_bytes().to_vec();
        let command = 0_u8;
        let stamp = 200_u8;

        let mut data: Vec<u8> = vec![command, stamp];
        data.append(&mut payload.clone());

        let packet = IncomingPacket::from_bytes(&data).unwrap();
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

        assert!(IncomingPacket::from_bytes(&data).is_err());
    }
}
