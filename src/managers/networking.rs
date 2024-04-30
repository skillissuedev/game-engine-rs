use crate::objects::Transform;
use machineid_rs::{HWIDComponent, IdBuilder};
use once_cell::sync::Lazy;
use renet::{
    transport::{
        ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
        ServerConfig,
    },
    ConnectionConfig, DefaultChannel, DisconnectReason, RenetClient, RenetServer, ServerEvent,
};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    num::ParseIntError,
    time::{Duration, SystemTime},
};

use super::{debugger, systems::get_system_mut_with_id};

static mut MAX_BYTES_PER_TICK: u64 = 100 * 1024 * 1024;
static mut CURRENT_NETWORKING_MODE: NetworkingMode = NetworkingMode::Disconnected(None);
static mut CURRENT_NETWORK_EVENTS: Vec<NetworkEvent> = vec![];
static mut CLIENT_ID: Lazy<u64> = Lazy::new(generate_client_id);

#[derive(Debug)]
pub struct ServerHandle {
    server: RenetServer,
    transport: NetcodeServerTransport,
}

#[derive(Debug)]
pub struct ClientHandle {
    client: RenetClient,
    transport: NetcodeClientTransport,
    status: ClientStatus,
}

#[derive(Debug)]
pub enum MessageReliability {
    Reliable,
    Unreliable,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageReceiver {
    Everybody,
    EverybodyExcept(u64),
    OneClient(u64),
}

#[derive(Debug)]
pub enum NetworkingMode {
    Server(ServerHandle),
    Client(ClientHandle),
    Disconnected(Option<DisconnectReason>),
}

#[derive(Debug)]
pub enum ClientStatus {
    Connecting,
    Connected,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub receiver: MessageReceiver,
    pub system_id: String,
    pub message_id: String,
    pub message: MessageContents,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageContents {
    SyncObject(SyncObjectMessage),
    Custom(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncObjectMessage {
    pub object_name: String,
    pub transform: Transform,
}

#[derive(Debug)]
pub enum NetworkError {
    MessageSerializeErr(serde_bare::error::Error),
    NotDisconnected,
    IsDisconnected,
    WrongClientStatus,
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    ClientConnected(u64),
    ClientDisconnected(u64, String),
    ConnectedSuccessfully,
    Disconnected(Option<DisconnectReason>),
}

fn set_current_networking_mode(mode: NetworkingMode) {
    unsafe {
        CURRENT_NETWORKING_MODE = mode;
    }
}

pub fn get_current_networking_mode() -> &'static NetworkingMode {
    unsafe { &CURRENT_NETWORKING_MODE }
}

pub fn new_server(port: u16, max_players: usize) -> Result<(), NetworkError> {
    match get_current_networking_mode() {
        NetworkingMode::Disconnected(_) => (),
        _ => {
            debugger::error(&format!(
                "new_server call error!\nCURRENT_NETWORK_MODE is not Disconnected"
            ));
            return Err(NetworkError::NotDisconnected);
        }
    }

    let server_address: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let cfg = ConnectionConfig::default();

    let server = RenetServer::new(cfg);
    let socket: UdpSocket = UdpSocket::bind(server_address).unwrap();

    let server_config = ServerConfig {
        max_clients: max_players,
        protocol_id: 0,
        public_addr: server_address,
        authentication: ServerAuthentication::Unsecure,
    };
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let transport = NetcodeServerTransport::new(current_time, server_config, socket).unwrap();

    let handle = ServerHandle { server, transport };
    println!("creating server");
    set_current_networking_mode(NetworkingMode::Server(handle));
    Ok(())
}

pub fn new_client(ip_address: IpAddr, port: u16) -> Result<(), NetworkError> {
    match get_current_networking_mode() {
        NetworkingMode::Disconnected(_) => (),
        _ => {
            debugger::error(&format!(
                "new_client call error!\nCURRENT_NETWORK_MODE is not Disconnected"
            ));
            return Err(NetworkError::NotDisconnected);
        }
    }

    let cfg = ConnectionConfig::default();

    let client = RenetClient::new(cfg);

    let server_addr: SocketAddr = SocketAddr::new(ip_address, port);
    let socket: UdpSocket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let auth = ClientAuthentication::Unsecure {
        protocol_id: 0,
        client_id: unsafe { *CLIENT_ID },
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, auth, socket).unwrap();

    let status = ClientStatus::Connecting;
    let handle = ClientHandle {
        client,
        transport,
        status,
    };
    set_current_networking_mode(NetworkingMode::Client(handle));
    Ok(())
}

impl ServerHandle {
    pub fn send_message(
        &mut self,
        message_reliability: MessageReliability,
        message: Message,
    ) -> Result<(), NetworkError> {
        let renet_message_reliability = match message_reliability {
            MessageReliability::Reliable => DefaultChannel::ReliableOrdered,
            MessageReliability::Unreliable => DefaultChannel::Unreliable,
        };

        let message_bytes_vec = match serde_bare::to_vec(&message) {
            Ok(vec) => vec,
            Err(err) => {
                debugger::error(&format!("got an error when calling send_message in ServerHandle\nfailed to serialize message to bytes vec\nerr: {}", err));
                return Err(NetworkError::MessageSerializeErr(err));
            }
        };

        match message.receiver {
            MessageReceiver::Everybody => {
                self.server
                    .broadcast_message(renet_message_reliability, message_bytes_vec);
            }
            MessageReceiver::EverybodyExcept(client_id) => self.server.broadcast_message_except(
                client_id,
                renet_message_reliability,
                message_bytes_vec,
            ),
            MessageReceiver::OneClient(client_id) => {
                self.server
                    .send_message(client_id, renet_message_reliability, message_bytes_vec)
            }
        };

        Ok(())
    }

    pub fn update(&mut self, delta_time: Duration) {
        self.server.update(delta_time);
        match self.transport.update(delta_time, &mut self.server) {
            Ok(_) => (),
            Err(err) => debugger::warn(&format!(
                "failed to update server transport\nerror: {}",
                err
            )),
        }

        while let Some(ev) = self.server.get_event() {
            match ev {
                ServerEvent::ClientConnected { client_id } => {
                    println!("client connected! client_id: {}", client_id);
                    set_network_event(NetworkEvent::ClientConnected(client_id));
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    println!("client disconnected! client_id: {}", client_id);
                    set_network_event(NetworkEvent::ClientDisconnected(
                        client_id,
                        reason.to_string(),
                    ));
                }
            }
        }

        for client_id in self.server.clients_id() {
            while let Some(message_bytes) = self
                .server
                .receive_message(client_id, DefaultChannel::ReliableOrdered)
            {
                send_message_to_system(message_bytes.into());
            }

            while let Some(message_bytes) = self
                .server
                .receive_message(client_id, DefaultChannel::Unreliable)
            {
                send_message_to_system(message_bytes.into());
            }
        }

        self.transport.send_packets(&mut self.server);
    }
}

impl ClientHandle {
    pub fn send_message(
        &mut self,
        message_reliability: MessageReliability,
        message: Message,
    ) -> Result<(), NetworkError> {
        match self.status {
            ClientStatus::Connected => (),
            _ => {
                debugger::error(&format!(
                    "failed to send message\nclient status is not Connected\nstatus: {:?}",
                    self.status
                ));
                return Err(NetworkError::WrongClientStatus);
            }
        }

        let renet_message_reliability = match message_reliability {
            MessageReliability::Reliable => DefaultChannel::ReliableOrdered,
            MessageReliability::Unreliable => DefaultChannel::Unreliable,
        };

        let message_bytes_vec = match serde_bare::to_vec(&message) {
            Ok(vec) => vec,
            Err(err) => {
                debugger::error(&format!("got an error when calling send_message in ServerHandle\nfailed to serialize message to bytes vec\nerr: {}", err));
                return Err(NetworkError::MessageSerializeErr(err));
            }
        };

        self.client
            .send_message(renet_message_reliability, message_bytes_vec);
        Ok(())
    }

    pub fn update(&mut self, delta_time: Duration) {
        self.set_client_status();
        match get_current_networking_mode() {
            NetworkingMode::Disconnected(_) => return,
            _ => (),
        }

        self.client.update(delta_time);
        match self.transport.update(delta_time, &mut self.client) {
            Ok(_) => (),
            Err(err) => debugger::warn(&format!("failed to update client transport\nerr: {}", err)),
        }

        while let Some(message_bytes) = self.client.receive_message(DefaultChannel::ReliableOrdered)
        {
            send_message_to_system(message_bytes.into());
        }

        while let Some(message_bytes) = self.client.receive_message(DefaultChannel::Unreliable) {
            send_message_to_system(message_bytes.into());
        }

        match self.transport.send_packets(&mut self.client) {
            Ok(_) => (),
            Err(err) => debugger::warn(&format!("failed to send packets\nerr: {}", err)),
        };
    }

    fn set_client_status(&mut self) {
        if self.transport.is_connected() {
            if let ClientStatus::Connecting = self.status {
                set_network_event(NetworkEvent::ConnectedSuccessfully);
                println!("Connected successfully!");
            }
            self.status = ClientStatus::Connected;
        } else if self.transport.is_connecting() {
            self.status = ClientStatus::Connecting
        } else if let Some(reason) = self.client.disconnect_reason() {
            set_current_networking_mode(NetworkingMode::Disconnected(Some(reason)));
            println!("disconnected!\nreason: {}", reason);
        } else if self.client.is_disconnected() {
            set_current_networking_mode(NetworkingMode::Disconnected(None));
            println!("disconnected!\nreason is None");
        }
    }
}

pub fn update(delta_time: Duration) {
    unsafe {
        CURRENT_NETWORK_EVENTS.clear();

        match &mut CURRENT_NETWORKING_MODE {
            NetworkingMode::Server(server) => server.update(delta_time),
            NetworkingMode::Client(client) => client.update(delta_time),
            NetworkingMode::Disconnected(_) => (),
        }
    }
}

pub fn send_message(reliability: MessageReliability, message: Message) -> Result<(), NetworkError> {
    unsafe {
        match &mut CURRENT_NETWORKING_MODE {
            NetworkingMode::Server(server) => server.send_message(reliability, message),
            NetworkingMode::Client(client) => client.send_message(reliability, message),
            NetworkingMode::Disconnected(_) => {
                debugger::error(&format!("can't send a message while being disconnected!"));
                return Err(NetworkError::IsDisconnected);
            }
        }
    }
}

pub fn send_message_to_system(message_bytes: Vec<u8>) {
    let message: Message = match serde_bare::from_slice(&message_bytes) {
        Ok(message) => message,
        Err(err) => {
            debugger::error(&format!("networking manager error\ngot an error in update(client)!\nfailed to deserialize message\nerr: {}", err));
            return;
        }
    };

    match get_system_mut_with_id(&message.system_id.clone()) {
        Some(system) => system.reg_message(message),
        None => {
            debugger::error(&format!("networking manager error\ngot an error in update(client)!\nfailed to get system {}", &message.system_id));
            ()
        }
    }
}

fn set_network_event(event: NetworkEvent) {
    unsafe {
        CURRENT_NETWORK_EVENTS.push(event);
    }
}

pub fn get_network_events() -> Vec<NetworkEvent> {
    unsafe {
        CURRENT_NETWORK_EVENTS.clone()
    }
}

fn generate_client_id() -> u64 {
    let mut id_in_binary = "".to_string();

    let mut id_builer = IdBuilder::new(machineid_rs::Encryption::MD5);
    id_builer.add_component(HWIDComponent::CPUID);
    id_builer.add_component(HWIDComponent::OSName);
    id_builer.add_component(HWIDComponent::SystemID);
    id_builer.add_component(HWIDComponent::DriveSerial);

    match id_builer.build("really great key") {
        Ok(id_str) => {
            for character in id_str.clone().into_bytes() {
                if id_in_binary.len() < 18 {
                    id_in_binary += &format!("{}", character);
                } else {
                    break;
                }
            }

            let id_u64_result: Result<u64, ParseIntError> = id_in_binary.parse();
            println!("CLIENT_ID: {:?}", id_u64_result);

            match id_u64_result {
                Ok(id) => return id,
                Err(err) => {
                    debugger::crash(&format!("Failed to generate CLIENT_ID! Error: {}", err));
                    unreachable!()
                }
            }
        }
        Err(err) => {
            debugger::crash(&format!("Failed to generate CLIENT_ID! Error: {}", err));
            unreachable!();
        }
    }
}

pub fn is_server() -> bool {
    match get_current_networking_mode() {
        NetworkingMode::Server(_) => true,
        NetworkingMode::Client(_) => false,
        NetworkingMode::Disconnected(_) => false,
    }
}

pub fn is_client() -> bool {
    match get_current_networking_mode() {
        NetworkingMode::Server(_) => false,
        NetworkingMode::Client(_) => true,
        NetworkingMode::Disconnected(_) => false,
    }
}

pub fn disconnect() {
    unsafe {
        match &mut CURRENT_NETWORKING_MODE {
            NetworkingMode::Server(_) => {
                debugger::error("failed to disconnect!\ncurrent networking mode is Server");
            }
            NetworkingMode::Client(client) => {
                client.client.disconnect();
            }
            NetworkingMode::Disconnected(_) => {
                debugger::error("failed to disconnect!\ncurrent networking mode is Disconnected");
            }
        }
    }
}
