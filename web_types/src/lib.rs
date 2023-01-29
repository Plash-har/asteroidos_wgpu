use std::time::{Instant, Duration};

use serde::{Serialize, Deserialize};

use game_logic::Player;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// The big container this one gets serialized
/// From client to server
pub enum UpMsgBox {
    NewConnection,
    KeepAlive {
        #[serde(with = "serde_millis")]
        time: Instant,
    },
    PlayerUpdate {
        player: Player,
        #[serde(with = "serde_millis")]
        time: Instant,
    },
    Disconect,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// The big container this one gets serialized
/// From server to client
pub enum DownMsgBox {
    ConnectionAcknowleged {
        key: u64,
        your_id: usize,
    },
    ServerClosing,
    KeepAlive {
        #[serde(with = "serde_millis")]
        time: Instant,
    },
    GameUpdate(GameUpdate),
    // If the server doesn't recognise the player
    // like if he was disconnected from the server but the client still sends packages
    Unrecognised,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GameUpdate {
    PlayerUpdate {
        id: usize,
        player: Player,
    },
    AsteroidChunkGen {
        pos: (i64, i64),
        #[serde(with = "serde_millis")]
        time: Instant,
    },
    NewPlayer {
        id: usize,
        player: Player,
    },
    PlayerDisconnect {
        id: usize,
    },
}

/// The max time for a client to not respond if more we disconnect to client
pub const TIMEOUT: Duration = Duration::from_secs(1);