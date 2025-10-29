mod message;
pub use message::*;

mod setup;
pub use setup::*;

mod utils;
pub use utils::*;

// https://docs.rs/renet/1.2.0/src/renet/channel/mod.rs.html
pub const NET_CHANNELS: [u8; 2] = [
    0, // DefaultChannel::Unreliable
    1, // DefaultChannel::ReliableUnordered
];
