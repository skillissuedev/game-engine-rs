use std::{env, net::Ipv4Addr};

use clap::Parser;
use framework::DebugMode;
use rand::Rng;

use crate::{framework::set_global_system_value, managers::{networking::get_current_networking_mode, saves::{load_save, new_save, register_save_value}, systems::SystemValue}};

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

    if let Some(save_name) = args.new_save_name {
        println!("New save name is {}", save_name);

        let seed: u32;
        match args.new_save_seed {
            Some(arg_seed) => {
                println!("New save seed is {}", arg_seed);
                seed = arg_seed;
            },
            None => {
                println!("New save seed isn't specified. Setting to a rangdom one.");
                seed = rand::thread_rng().gen_range(1..u32::MAX);
            },
        }

        set_global_system_value("WorldGeneratorSeed", vec![SystemValue::UInt(seed)]);
        register_save_value("WorldGeneratorSeed");

        match new_save(&save_name) {
            Ok(_) => println!("Successfully created a new save file!"),
            Err(err) => println!("Failed to create a new save file!\nErr: {}", err),
        }
        return;
    }

    if let Some(save) = args.load_save {
        println!("Runnning game as server on port 7777");

        if let Err(_) = load_save(&save) {
            println!("Failed to load the save file and start the server!");
            return;
        }

        managers::networking::new_server(7777, 10).unwrap();
        framework::start_game_without_render();
    }

    let ip;
    if let Some(args_ip) = args.ip {
        ip = args_ip;
    } else {
        ip = Ipv4Addr::new(127, 0, 0, 1);
    }
    println!("Connecting to {}:7777", ip);
    managers::networking::new_client(
        std::net::IpAddr::V4(ip),
        7777,
    ).unwrap();

    match get_current_networking_mode() {
        managers::networking::NetworkingMode::Server(_) => (),
        managers::networking::NetworkingMode::Client(_) => (),
        managers::networking::NetworkingMode::Disconnected(_) => {
        }
    }

    if args.debug {
        println!("debug");
        framework::start_game_with_render(DebugMode::Full);
    } else {
        framework::start_game_with_render(DebugMode::None);
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    pub load_save: Option<String>,
    #[arg(long)]
    pub debug: bool,
    #[arg(long)]
    pub new_save_seed: Option<u32>,
    #[arg(long)]
    pub new_save_name: Option<String>,
    #[arg(long="connect")]
    pub ip: Option<Ipv4Addr>
}
