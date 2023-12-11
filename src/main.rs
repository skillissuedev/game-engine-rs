use std::{env, net::Ipv4Addr};

use framework::DebugMode;

use crate::managers::networking::get_current_networking_mode;

mod assets;
mod framework;
mod game;
mod managers;
mod math_utils;
mod objects;
mod systems;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("run args:");
    dbg!(&args);
    println!("---\n");

    if let Some(arg1) = args.get(1) {
        if arg1 == "server" {
            println!("runnning game as server");
            managers::networking::new_server(9999, 10).unwrap();
        }
    }
    match get_current_networking_mode() {
        managers::networking::NetworkingMode::Server(_) => (),
        managers::networking::NetworkingMode::Client(_) => (),
        managers::networking::NetworkingMode::Disconnected(_) => {
            println!("creating a client");
            managers::networking::new_client(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999).unwrap();
        },
    }


    #[cfg(debug_assertions)]
    framework::start_game(DebugMode::ShowFps);

    #[cfg(not(debug_assertions))]
    framework::start_game(DebugMode::ShowFps);
}
