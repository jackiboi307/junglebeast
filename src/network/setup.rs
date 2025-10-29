use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
    sync::mpsc::TryRecvError,
};

use renet::{
    ClientId, ConnectionConfig, DefaultChannel, RenetClient, RenetServer, ServerEvent
};
use renet_netcode::{
    ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
    ServerAuthentication, ServerConfig, NETCODE_USER_DATA_BYTES,
};

const PROTOCOL_ID: u64 = 7;

pub fn create_client(addr: String) -> (RenetClient, NetcodeClientTransport) {
    let addr = addr.parse().unwrap();
    let connection_config = ConnectionConfig::default();
    let client = RenetClient::new(connection_config);

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        server_addr: addr,
        client_id,
        user_data: None,
        protocol_id: PROTOCOL_ID,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    (client, transport)
}

pub fn create_server(addr: String) -> (RenetServer, NetcodeServerTransport) {
    let addr = addr.parse().unwrap();
    let connection_config = ConnectionConfig::default();
    let server: RenetServer = RenetServer::new(connection_config);

    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let socket: UdpSocket = UdpSocket::bind(addr).unwrap();
    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

    (server, transport)
}
