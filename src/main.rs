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
    println!("run args:\n{:#?}\n---\n\n", &args);

    if args.contains(&"server".into()) {
        println!("runnning game as server");
        managers::networking::new_server(7777, 10).unwrap();
        framework::start_game_without_render();
    }

    match get_current_networking_mode() {
        managers::networking::NetworkingMode::Server(_) => (),
        managers::networking::NetworkingMode::Client(_) => (),
        managers::networking::NetworkingMode::Disconnected(_) => {
            println!("creating a client");
            managers::networking::new_client(
                std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                7777,
            )
            .unwrap();
        }
    }

    if args.contains(&"debug".into()) {
        println!("debug");
        framework::start_game_with_render(DebugMode::Full);
    } else {
        framework::start_game_with_render(DebugMode::None);
    }
}
