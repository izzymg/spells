use std::sync::mpsc;

// wraps a send channel
#[derive(bevy::ecs::system::Resource)]
pub struct ClientStreamSender(mpsc::Sender<Vec<u8>>);

impl ClientStreamSender {

    pub fn new(tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self(tx)
    }

    // Returns false if sending is now impossible (very bad)
    pub fn send_data(&mut self, data: Vec<u8>) -> bool {
        !self.0.send(data).is_err()
    }
}
