use std::fmt;
use std::time::Instant;

pub const CMD_VELOCITY: u8 = 0;
pub const CMD_CAST_SPELL: u8 = 1;

#[derive(Debug, Clone, Copy)]
pub(super) struct InvalidPacketError;

impl fmt::Display for InvalidPacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid packet")
    }
}

pub(super) struct IncomingPacket {
    timestamp: Instant,
    command: u8,
    payload: Vec<u8>,
}

impl IncomingPacket {
    pub(super) fn from_bytes(data: Vec<u8>) -> Result<IncomingPacket, InvalidPacketError> {
        if let Some(cmd_bytes) = data.first() {
            let command = u8::from_le_bytes([*cmd_bytes]);
            let rest = &data[1..data.len()];
            Ok(IncomingPacket {
                timestamp: Instant::now(),
                command,
                payload: rest.to_vec(),
            })
        } else {
            Err(InvalidPacketError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_inc_packet() {
        let payload = 150000_u32.to_le_bytes().to_vec();
        let command = CMD_CAST_SPELL;

        let mut data = vec![command];
        data.append(&mut payload.clone()); // payload gets consumed by append

        let packet = IncomingPacket::from_bytes(data).unwrap();
        assert_eq!(packet.command, command);
        assert_eq!(packet.payload, payload);
        println!("{:?}\n{:?}", payload, packet.payload);
    }
}
