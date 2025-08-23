use std::{collections::HashMap, net::Ipv4Addr};

use clap::Parser;
use framework::DebugMode;

use crate::managers::saves::SavesManager;
mod assets;
mod framework;
mod game;
mod managers;
mod math_utils;
mod objects;
mod systems;

fn main() {
    let args = Args::parse();
    println!("run args:\n{:#?}\n---\n\n", &args);

    if let Some(save_name) = &args.new_save_name {
        println!("New save name is {}", save_name);

        match SavesManager::default().new_save(&save_name, &HashMap::new()) {
            Ok(_) => println!("Successfully created a new save file!"),
            Err(err) => println!("Failed to create a new save file!\nErr: {}", err),
        }
        framework::start_game_without_render(args.clone());
        return;
    }

    if let Some(_) = &args.load_save {
        println!("Runnning game as server on port 7777");

        managers::networking::new_server(7777, 10).unwrap();
        framework::start_game_without_render(args.clone());
    }

    let ip;
    if let Some(args_ip) = args.ip {
        ip = args_ip;
    } else {
        ip = Ipv4Addr::new(127, 0, 0, 1);
    }

    let client_id = args.client_id;
    println!("Connecting to {}:7777, client ID is {}", ip, client_id);
    managers::networking::new_client(std::net::IpAddr::V4(ip), 7777, client_id).unwrap();

    let debug = args.debug.clone();

    if debug {
        println!("debug");
        framework::start_game_with_render(args, DebugMode::Full);
    } else {
        framework::start_game_with_render(args, DebugMode::None);
    }
}

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(long)]
    pub load_save: Option<String>,
    #[arg(long)]
    pub debug: bool,
    #[arg(long)]
    pub new_save_name: Option<String>,
    #[arg(long = "connect")]
    pub ip: Option<Ipv4Addr>,
    #[arg(long = "clientid")]
    #[clap(default_value_t)]
    pub client_id: u64,
}
