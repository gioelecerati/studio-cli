use rand::thread_rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::*;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct ApiKeyInfo {
    email: String,
    admin: String,
    api_key: String,
}

// Load api keys from folder $HOME/.studio/$ENV
pub fn load_api_keys_from_disk(env: &String) -> Option<String> {
    let home = dirs::home_dir().unwrap();
    let studio_path = home.join(".studio");
    let env_path = studio_path.join(env);

    if !studio_path.exists() {
        warn!("No .studio folder found {}, creating", studio_path.display());
        std::fs::create_dir(&studio_path).unwrap();
    } else {
        info!("Found .studio folder {}", studio_path.display());
    }

    if !env_path.exists() {
        warn!("No Api Keys found {}, creating", env_path.display());
        std::fs::create_dir(&env_path).unwrap();
        return Some(ask_create_api_key(env, &env_path));
    }

    let api_keys: Vec<_> = std::fs::read_dir(&env_path).unwrap().collect();
    if api_keys.is_empty() {
        warn!("No Api Keys found {}", env_path.display());
        return Some(ask_create_api_key(env, &env_path));
    }

    let mut items = vec![String::from("+ Add a new API KEY")];
    let mut file_map = HashMap::new();

    for entry in api_keys {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let contents = std::fs::read_to_string(&path).unwrap();
            let api_key_info: ApiKeyInfo = serde_json::from_str(&contents).unwrap();
            let redacted_key = format!("{}*****", &api_key_info.api_key[..5]);
            let display_string = format!("{} - {} - {}", api_key_info.email, api_key_info.admin, redacted_key);
            items.push(display_string.clone());
            file_map.insert(display_string, path.file_name().unwrap().to_str().unwrap().to_string());
        }
    }

    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&items)
        .with_prompt(format!("Please select an api key for the env {}", env))
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(0) => Some(ask_create_api_key(env, &env_path)),
        Some(index) => {
            let selected_item = items[index].clone();
            let filename = file_map.get(&selected_item).unwrap();
            let contents = std::fs::read_to_string(env_path.join(filename)).unwrap();
            let api_key_info: ApiKeyInfo = serde_json::from_str(&contents).unwrap();
            info!("Selected api key {}", api_key_info.api_key);
            Some(api_key_info.api_key)
        }
        None => {
            warn!("No api key selected");
            Some(ask_create_api_key(env, &env_path))
        }
    }
}

pub fn ask_create_api_key(env: &String, path: &PathBuf) -> String {
    let api_key = dialoguer::Input::new()
        .with_prompt(format!("Please enter your api key for the env {}", env))
        .interact_text()
        .unwrap();

    if let Some(userinfo) = is_valid_api_key(&api_key, env) {
        save_api_key_to_disk(env, &api_key, &userinfo);
    } else {
        crate::init();
        std::process::exit(0);
    }
    api_key
}

pub fn is_valid_api_key(api_key: &String, env: &str) -> Option<livepeer_rs::user::UserInfo> {
    let lvpr_env = match env {
        "prod" => livepeer_rs::LivepeerEnv::Prod,
        _ => livepeer_rs::LivepeerEnv::Stg,
    };

    match livepeer_rs::Livepeer::new(Some(api_key.clone()), Some(lvpr_env)) {
        Ok(cl) => {
            info!("Api key is valid");
            Some(cl.user.info)
        }
        Err(e) => {
            error!("Api key is not valid {}", e);
            None
        }
    }
}

pub fn save_api_key_to_disk(env: &String, api_key: &String, userinfo: &livepeer_rs::user::UserInfo) {
    let home = dirs::home_dir().unwrap();
    let path = home.join(".studio").join(env);

    let api_key_info = ApiKeyInfo {
        email: userinfo.email.clone(),
        admin: if userinfo.admin { "Admin".to_string() } else { "User".to_string() },
        api_key: api_key.clone(),
    };

    let filecontent = serde_json::to_string(&api_key_info).unwrap();
    let filename = format!("{:x}", Sha256::digest(filecontent.as_bytes()));

    let mut file = std::fs::File::create(path.join(&filename)).unwrap();
    file.write_all(filecontent.as_bytes()).unwrap();

    info!("Api key saved to disk {}", path.join(&filename).display());
}

pub fn save_string_to_disk(env: &String, filename: &String, content: &String) {
    let home = dirs::home_dir().unwrap();
    let path = home.join(".studio").join(env);

    if !path.exists() {
        std::fs::create_dir(&path).unwrap();
    }

    let save_path = path.join(filename);
    let mut file = std::fs::File::create(save_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    info!("File saved to disk {}", path.join(filename).display());
}

pub fn get_string_from_disk(env: &String, filename: &String) -> Option<String> {
    let home = dirs::home_dir().unwrap();
    let path = home.join(".studio").join(env);

    let mut file = std::fs::File::open(path.join(filename)).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    info!("File loaded from disk {}", path.join(filename).display());

    Some(contents)
}
