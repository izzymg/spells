use std::collections::HashMap;
use std::fmt::Display;
use std::io;

use super::{tcp_stream, Token};

const MAX_PAYLOAD_SIZE: u8 = 8 + 1; // inclusive of delimiter
const DELIMITER: u8 = 0x3b; // ;

pub struct BroadcastError {
    pub token: Token,
    pub error: io::Error,
}

impl Display for BroadcastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "broadcast failure on client ({}): {}",
            self.token.0, self.error
        )
    }
}

#[derive(Debug)]
pub enum ReceiveError {
    IoError(io::Error),
    MessageSize,
    BadDelimiter,
}

impl Display for ReceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReceiveError::IoError(err) => {
                write!(f, "io error: {}", err)
            }
            ReceiveError::MessageSize => {
                write!(f, "invalid message size")
            }
            ReceiveError::BadDelimiter => {
                write!(f, "bad delimiter, expected {:#x}", DELIMITER)
            }
        }
    }
}

impl From<io::Error> for ReceiveError {
    fn from(value: io::Error) -> Self {
        ReceiveError::IoError(value)
    }
}

fn read_stream_message(stream: &mut impl io::Read) -> Result<Vec<u8>, ReceiveError> {
    let mut header = [0_u8; 1];
    stream.read_exact(&mut header)?;
    let to_read = u8::from_le_bytes(header);
    if !(1..=MAX_PAYLOAD_SIZE).contains(&to_read) {
        return Err(ReceiveError::MessageSize);
    }
    let mut buf = vec![0_u8; to_read as usize];
    stream.read_exact(&mut buf)?;
    if *(buf.last().unwrap()) != DELIMITER {
        dbg!(buf);
        return Err(ReceiveError::BadDelimiter);
    }
    Ok(buf[0..buf.len() - 1].to_vec())
}

#[derive(Default)]
pub struct ConnectedClients {
    map: HashMap<Token, tcp_stream::ClientStream>,
}

impl ConnectedClients {
    pub fn add(&mut self, token: Token, stream: tcp_stream::ClientStream) {
        self.map.insert(token, stream);
    }

    pub fn remove(&mut self, token: Token) -> Option<tcp_stream::ClientStream> {
        self.map.remove(&token)
    }

    pub fn broadcast(&mut self, data: &[u8]) -> Vec<BroadcastError> {
        self.map
            .iter_mut()
            .map(|(token, client)| {
                client.write_prefixed(data).map_err(|error| BroadcastError {
                    error,
                    token: *token,
                })
            })
            .filter_map(|v| v.is_err().then(|| v.unwrap_err()))
            .collect()
    }

    /// returns `None` if `token` isn't a connected client
    pub fn try_receive(&mut self, token: Token) -> Result<Option<Vec<u8>>, ReceiveError> {
        let client = match self.map.get_mut(&token) {
            Some(client) => client,
            None => {
                return Ok(None);
            }
        };

        let message = read_stream_message(client)?;
        Ok(Some(message))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use io::Read;
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
        *message.last_mut().unwrap() = DELIMITER;
        dbg!(message);
        let response = read_stream_message(&mut FakeReader {
            payload: message.to_vec(),
            read: 0,
        })
        .unwrap();
        // everything but the header & the delimiter
        assert_eq!(response, (message[1..message.len() - 1]).to_vec());
    }
}
