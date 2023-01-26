use pwbox::{sodium::Sodium, ErasedPwBox, Eraser, Suite};
use rand::thread_rng;

// Load api keys from folder $HOME/.studio/$ENV
pub fn load_api_keys_from_disk(env: &String) -> Option<String> {
    let mut selected_api_key = None;
    // Does the folder exist?
    let home = dirs::home_dir().unwrap();

    let p = home.join(".studio");

    if p.exists() {
        info!("Found .studio folder {}", p.display());
    } else {
        warn!("No .studio folder found {}, creating", p.display());
        // create folder
        std::fs::create_dir(p.clone()).unwrap();
    }

    let path = p.join(env);

    if !path.exists() {
        warn!("No Api Keys found {}", path.display());
        // create folder
        std::fs::create_dir(path.clone()).unwrap();
        selected_api_key = Some(ask_create_api_key(env, &path))
    }

    // Load api keys from folder
    let mut api_keys = std::fs::read_dir(path.clone()).unwrap();
    let mut api_keys2 = std::fs::read_dir(path.clone()).unwrap();

    // if no files are found
    if api_keys.count() == 0 {
        warn!("No Api Keys found {}", path.display());
        selected_api_key = Some(ask_create_api_key(env, &path))
    } else {
        // ask the user to select an api key
        let mut items = Vec::new();
        items.push(String::from("+ Add a new API KEY"));
        for entry in api_keys2 {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                items.push(path.file_name().unwrap().to_str().unwrap().to_string());
            }
        }

        let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .items(&items)
            .with_prompt(format!("Please select an api key for the env {}", env))
            .default(0)
            .interact_on_opt(&crate::Term::stderr())
            .unwrap();

        match selection {
            Some(index) => {
                if index == 0 {
                    ask_create_api_key(env, &path);
                } else {
                    let api_key = items[index].clone();

                    // read file

                    let mut file = std::fs::File::open(path.join(&api_key)).unwrap();
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();

                    info!("Selected api key {}", contents);
                    selected_api_key = Some(contents);
                }
            }
            None => {
                warn!("No api key selected");
                selected_api_key = Some(ask_create_api_key(env, &path));
            }
        }
    }

    selected_api_key
}

pub fn ask_create_api_key(env: &String, path: &std::path::PathBuf) -> String {
    // ask the user to input a api key
    let api_key = dialoguer::Input::new()
        .with_prompt(format!("Please enter your api key for the env {}", env))
        .interact_text()
        .unwrap();

    let userinfo = is_valid_api_key(&api_key, &env);

    if userinfo.is_some() {
        // save the api key to disk

        let ui = userinfo.unwrap();
        save_api_key_to_disk(env, &api_key, &ui);
    } else {
        crate::init();
        std::process::exit(0);
    }
    return api_key;
}

pub fn is_valid_api_key(api_key: &String, env: &str) -> Option<livepeer_rs::user::UserInfo> {
    // check if the api key is valid
    let mut _lvpr_env = livepeer_rs::LivepeerEnv::Prod;

    match env {
        "prod" => _lvpr_env = livepeer_rs::LivepeerEnv::Prod,
        "stg" => _lvpr_env = livepeer_rs::LivepeerEnv::Stg,
        _ => _lvpr_env = livepeer_rs::LivepeerEnv::Stg,
    }
    let lvpr_client = livepeer_rs::Livepeer::new(Some(String::from(api_key)), Some(_lvpr_env));

    match lvpr_client {
        Ok(cl) => {
            info!("Api key is valid");

            let userinfo = cl.user.info;

            return Some(userinfo);
        }
        Err(e) => {
            error!("Api key is not valid {}", e);

            return None;
        }
    }
}

use std::io::*;

pub fn save_api_key_to_disk(
    env: &String,
    api_key: &String,
    userinfo: &livepeer_rs::user::UserInfo,
) {
    //
    let home = dirs::home_dir().unwrap();
    let p = home.join(".studio");
    let path = p.join(env);

    let email = &userinfo.email;
    let isAdmin = userinfo.admin;

    let mut admin = String::new();

    if isAdmin {
        admin = String::from("Admin");
    } else {
        admin = String::from("User");
    }

    let filename = &format!("{} - {} - {} ", email, admin, api_key);
    let filecontent = api_key;

    let mut file = std::fs::File::create(path.join(filename)).unwrap();
    file.write_all(filecontent.as_bytes()).unwrap();

    info!("Api key saved to disk {}", path.join(filename).display());
}
