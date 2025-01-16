use std::{net::Ipv4Addr, time::Duration};

use clap::Parser;
use framework::DebugMode;
use rand::Rng;
mod assets;
mod framework;
mod game;
mod managers;
mod math_utils;
mod objects;
mod systems;

fn main() {
    std::thread::sleep(Duration::from_millis(250));

    let mut args = Args::parse();
    println!("run args:\n{:#?}\n---\n\n", &args);

    if let Some(save_name) = &args.new_save_name {
        println!("New save name is {}", save_name);

        let seed: u32;
        match &args.new_save_seed {
            Some(arg_seed) => {
                println!("New save seed is {}", arg_seed);
                seed = *arg_seed;
            }
            None => {
                println!("New save seed isn't specified. Setting a random one.");
                seed = rand::thread_rng().gen_range(1..u32::MAX);
            }
        }
        args.new_save_seed = Some(seed);

        //set_global_system_value("WorldGeneratorSeed", vec![SystemValue::UInt(seed)]);
        //register_save_value("WorldGeneratorSeed");

        /*match new_save(&save_name) {
            Ok(_) => println!("Successfully created a new save file!"),
            Err(err) => println!("Failed to create a new save file!\nErr: {}", err),
        }*/
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
    pub new_save_seed: Option<u32>,
    #[arg(long)]
    pub new_save_name: Option<String>,
    #[arg(long = "connect")]
    pub ip: Option<Ipv4Addr>,
    #[arg(long = "clientid")]
    #[clap(default_value_t)]
    pub client_id: u64,
}
