#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(warnings)]

#[macro_use]
extern crate log;

use colored::*;
use console::Term;

pub mod accesscontrol;
pub mod assets;
pub mod auth;
pub mod live;
pub mod playback;
pub mod tasks;
pub mod users;
pub mod ai;

use env_logger::{filter::Filter, fmt::Color, Builder, Logger};

fn main() {
    env_logger::init_from_env(env_logger::Env::default().filter_or("LIVEPEER_STUDIO_LOG", "warn"));
    println!(
        "{}",
        r#"
            __            ___                  ___
      _____/ /___  ______/ (_)___        _____/ (_)
     / ___/ __/ / / / __  / / __ \______/ ___/ / /
    (__  ) /_/ /_/ / /_/ / / /_/ /_____/ /__/ / /
   /____/\__/\__,_/\__,_/_/\____/      \___/_/_/
        v0.1.1
    "#
        .green()
    );
    init();
}

pub fn init() {
    let items = vec!["Prod", "Stg", "Dev", "Box"];
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&items)
        .with_prompt(
            "Welcome to Livepeer Studio - Please select an environment you want to interact with",
        )
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap();

    let lenv = match selection {
        Some(index) => match index {
            0 => "prod",
            1 => "stg",
            2 => "dev",
            3 => "box",
            _ => "stg",
        },
        None => "stg",
    };

    let api_key = auth::load_api_keys_from_disk(&String::from(lenv)).unwrap_or_else(|| {
        error!("Unable to load API Key, retry");
        crate::init();
        std::process::exit(0);
    });

    // Initialize livepeer client
    let _lvpr_env = match lenv {
        "prod" => livepeer_rs::LivepeerEnv::Prod,
        "stg" => livepeer_rs::LivepeerEnv::Stg,
        "dev" => livepeer_rs::LivepeerEnv::Dev,
        "box" => livepeer_rs::LivepeerEnv::Box,
        _ => livepeer_rs::LivepeerEnv::Stg,
    };

    info!("Initializing livepeer client on env {}", lenv);

    let lvpr_client = livepeer_rs::Livepeer::new(Some(api_key), Some(_lvpr_env)).unwrap();

    // select functionality {assets, streams, users}
    list_options(&lvpr_client);

    init();
}

fn list_options(lvpr_client: &livepeer_rs::Livepeer) {
    let options = ["Users", "Streams", "Assets", "Tasks", "Playback", "AI", "<- Back"];
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&options)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap();

    match selection {
        Some(index) => match index {
            0 => { users::users(&lvpr_client); }
            1 => { live::streams(&lvpr_client); }
            2 => { assets::assets(&lvpr_client); }
            3 => { tasks::tasks(&lvpr_client); }
            4 => playback::playbacks(&lvpr_client),
            5 => ai::generate(&lvpr_client),
            6 => {
                crate::init();
                std::process::exit(0);
            }
            _ => info!("No selection made"),
        },
        None => info!("No selection made"),
    }
}
