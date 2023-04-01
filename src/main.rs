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
    let items = vec!["Prod", "Stg", "Dev"];
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
            _ => "stg",
        },
        None => "stg",
    };

    let selected_api_key = auth::load_api_keys_from_disk(&String::from(lenv));

    let mut api_key = String::new();

    if selected_api_key.is_some() {
        api_key = selected_api_key.unwrap();
    } else {
        error!("Unable to load API Key, retry");
        crate::init();
        std::process::exit(0);
    }

    // Initialize livepeer client
    let mut _lvpr_env = livepeer_rs::LivepeerEnv::Stg;

    match lenv {
        "prod" => _lvpr_env = livepeer_rs::LivepeerEnv::Prod,
        "stg" => _lvpr_env = livepeer_rs::LivepeerEnv::Stg,
        "dev" => _lvpr_env = livepeer_rs::LivepeerEnv::Dev,
        _ => _lvpr_env = livepeer_rs::LivepeerEnv::Stg,
    }

    info!("Initalizing livepeer client on env {}", lenv);

    let lvpr_client =
        livepeer_rs::Livepeer::new(Some(String::from(api_key)), Some(_lvpr_env)).unwrap();

    // select functionality {assets, streams, users}

    list_options(&lvpr_client);

    init();
}

fn list_options(lvpr_client: &livepeer_rs::Livepeer) {
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&["Users", "Streams", "Assets", "Tasks", "Playback", "<- Back"])
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap();

    match selection {
        Some(index) => match index {
            0 => users::users(&lvpr_client),
            1 => {
                live::streams(&lvpr_client);
            }
            2 => {
                assets::assets(&lvpr_client);
            }
            3 => tasks::tasks(&lvpr_client),
            4 => playback::playbacks(&lvpr_client),
            5 => {
                crate::init();
                std::process::exit(0);
            }
            _ => {
                info!("No selection made");
            }
        },
        None => {
            info!("No selection made");
        }
    }
}
