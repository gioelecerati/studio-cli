use colored::*;
use livepeer_rs::{
    playback::Playback,
    vod::{Task, Vod},
};

pub mod upload;

pub fn assets(client: &livepeer_rs::Livepeer) -> bool {
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&[
            "My Assets",
            "Get Assets by User ID",
            "Get Asset By ID",
            "Upload Asset",
            "< Back",
        ])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(index) => {
            let mut asset_list = serde_json::Value::Null;
            let mut asset: Option<serde_json::Value> = None;
            let mut e: Option<_> = None;

            if index == 0 {
                let assets_value = client.asset.list_assets();

                if let Ok(a) = assets_value {
                    asset_list = a;
                } else {
                    error!("Error getting assets: {:?}", assets_value);
                    e = Some(());
                }
            }

            if index == 1 {
                let user_id = dialoguer::Input::<String>::new()
                    .with_prompt("Enter user ID")
                    .interact()
                    .unwrap();

                let assets_value = client.asset.get_assets_by_user_id(user_id);

                if let Ok(a) = assets_value {
                    asset_list = a;
                } else {
                    error!("Error getting assets: {:?}", assets_value);
                    e = Some(());
                }
            }

            if index == 2 {
                let asset_id = dialoguer::Input::<String>::new()
                    .with_prompt("Enter asset ID")
                    .interact()
                    .unwrap();
                let single_asset = client.asset.get_asset_by_id(String::from(asset_id));
                if let Ok(a) = single_asset {
                    asset = Some(a);
                } else {
                    error!("Error getting asset: {:?}", single_asset);
                    e = Some(());
                }
            }

            if asset.is_some() {
                inspect_asset(asset, client);
                assets(client);
            }

            if index == 3 {
                // Trigger upload function
                upload::upload_asset(client).unwrap();
                assets(client);
                return false;
            }

            if index == 4 {
                crate::list_options(&client);
                std::process::exit(0);
            }

            let selection =
                dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .items(&["Count", "List", "< Back"])
                    .default(0)
                    .interact_on_opt(&crate::Term::stderr())
                    .unwrap();

            match selection {
                Some(index) => {
                    let list = asset_list.as_array().unwrap();
                    if index == 0 {
                        // length of list
                        let count = list.len();
                        info!("Assets found: {}", count);
                        assets(client);
                    }

                    if list.len() == 0 {
                        warn!("No assets found");
                        crate::init();
                        std::process::exit(0);
                    }

                    if index == 1 {
                        // selection from list by id
                        let ids = list
                            .iter()
                            .map(|x| {
                                format!(
                                    "{} - {} - {} - {}",
                                    x["id"].as_str().unwrap(),
                                    x["name"].as_str().unwrap().cyan().bold(),
                                    x["status"]["phase"].as_str().unwrap(),
                                    x["status"]["updatedAt"],
                                )
                            })
                            .collect::<Vec<String>>();

                        let selection = dialoguer::Select::with_theme(
                            &dialoguer::theme::ColorfulTheme::default(),
                        )
                        .items(&ids)
                        .default(0)
                        .interact_on_opt(&crate::Term::stderr())
                        .unwrap();

                        match selection {
                            Some(index) => {
                                let id = list[index]["id"].as_str().unwrap();
                                let asset_value = client.asset.get_asset_by_id(String::from(id));

                                if let Ok(a) = asset_value {
                                    asset = Some(a);
                                    inspect_asset(asset, client);
                                } else {
                                    error!("Error getting asset: {:?}", asset_value);
                                    e = Some(());
                                }
                            }
                            None => {
                                error!("No selection made");
                            }
                        }
                    }
                }
                None => {
                    error!("No selection made");
                }
            }

            if e.is_some() {
                crate::init();
                std::process::exit(0);
            }
        }
        None => {
            warn!("No selection made");
        }
    }
    assets(client);
    return false;
}

pub fn inspect_asset(asset: Option<serde_json::Value>, client: &livepeer_rs::Livepeer) {
    let a = asset.unwrap();
    let task = client
        .task
        .get_task_by_output_asset_id(String::from(a["id"].as_str().unwrap()));

    let pretty_asset = serde_json::to_string_pretty(&a).unwrap();
    println!("{}", pretty_asset);

    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&[
            "Retrieve again",
            "Get originating task",
            "Playback Asset",
            "Get playback info",
            "< Back",
            "< Home",
        ])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(index) => {
            if index == 0 {
                inspect_asset(Some(a.clone()), client);
                assets(client);
            }

            if index == 1 {
                if let Ok(t) = task {
                    let pretty_task = serde_json::to_string_pretty(&t).unwrap();
                    println!("{}", pretty_task);
                    let single_task = t[0].clone();
                    crate::tasks::inspect_task(Some(single_task), client);
                } else {
                    info!("Error getting task: {:?}", task);
                }
            }

            if index == 3 {
                let playback_info = client
                    .playback
                    .get_playback_info(&String::from(a["playbackId"].as_str().unwrap()));
                if let Ok(p) = playback_info {
                    let pretty_playback_info = serde_json::to_string_pretty(&p).unwrap();
                    println!("{}", pretty_playback_info);
                    crate::playback::playback(p, &client);
                } else {
                    error!("Error getting playback info: {:?}", playback_info);
                }
            }

            if index == 4 {
                assets(client);
            }

            if index == 5 {
                crate::list_options(&client);
                std::process::exit(0);
            }

            if index == 2 {
                // run command ffplay with playbackURL
                let playback_url = a["playbackUrl"].as_str();

                let ffplay_path = crate::live::get_ffplay_path();

                if ffplay_path.is_err() {
                    error!("ffplay not found");
                    assets(client);
                }

                let ffplay = ffplay_path.unwrap();

                match playback_url {
                    Some(url) => {
                        info!("Playback URL: {}", url);
                        info!("Playing asset...");
                        info!("Wait for ffplay to load...");
                        let output = std::process::Command::new(ffplay)
                            .arg(url)
                            .output()
                            .expect("failed to execute process");
                    }
                    None => {
                        error!("No playback URL found");
                        assets(client);
                    }
                }

                assets(client);
            }
        }
        None => {
            error!("No selection made");
        }
    }
}
