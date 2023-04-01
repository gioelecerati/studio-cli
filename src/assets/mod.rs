use chrono::{DateTime, NaiveDateTime, Utc};
use colored::*;
use livepeer_rs::{
    playback::Playback,
    vod::{Task, Vod},
};

pub mod upload;

fn truncate_and_pad(s: &str, max_width: usize, min_width: usize) -> String {
    let truncated = if s.len() > max_width {
        s.chars().take(max_width).collect::<String>()
    } else {
        s.to_string()
    };
    format!("{:<width$}   ", truncated, width = min_width)
}

pub fn assets(client: &livepeer_rs::Livepeer) -> bool {
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&[
            "My Assets",
            "Get Assets by User ID",
            "Get Assets by CID",
            "Get Asset By ID or PlaybackID",
            "Upload Asset",
            "Test (Upload -> Task -> Playback -> Export to IPFS)",
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
                let assets_value = client.asset.list_paginated_assets(10000, 0, true);

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
                let cid = dialoguer::Input::<String>::new()
                    .with_prompt("Enter CID")
                    .interact()
                    .unwrap();

                let assets_value = client.asset.get_assets_by_cid(cid, client.user.info.admin);

                if let Ok(a) = assets_value {
                    asset_list = a;
                } else {
                    error!("Error getting assets: {:?}", assets_value);
                    e = Some(());
                }
            }

            if index == 3 {
                let asset_id = dialoguer::Input::<String>::new()
                    .with_prompt("Enter asset ID or Playback ID")
                    .interact()
                    .unwrap();
                let single_asset = client.asset.get_asset_by_id(String::from(asset_id.clone()));
                if let Ok(a) = single_asset {
                    asset = Some(a);
                } else {
                    let single_asset = client
                        .asset
                        .get_asset_by_playback_id(String::from(asset_id), client.user.info.admin);

                    if let Ok(a) = single_asset {
                        let a = &a.as_array().unwrap().clone()[0];
                        asset = Some(a.clone());
                    } else {
                        error!("Error getting asset: {:?}", single_asset);
                        e = Some(());
                    }
                }
            }

            if asset.is_some() {
                inspect_asset(asset, client);
                assets(client);
            }

            if index == 4 {
                // Trigger upload function
                upload::upload_asset(client).unwrap();
                assets(client);
                return false;
            }

            if index == 5 {
                let asset_test = test_asset_flow(client);
                assets(client);
            }

            if index == 6 {
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
                        let ids = list
                            .iter()
                            .map(|x| {
                                let created_at = x["createdAt"].as_str().unwrap_or("");

                                let created_at_formatted = if !created_at.is_empty() {
                                    let timestamp = created_at.parse::<i64>().unwrap();
                                    let naive_datetime =
                                        NaiveDateTime::from_timestamp(timestamp, 0);
                                    let datetime: DateTime<Utc> =
                                        DateTime::from_utc(naive_datetime, Utc);
                                    datetime.to_rfc3339()
                                } else {
                                    String::from("")
                                };

                                format!(
                                    "{id} - {name} - {phase} - {created_at} - {playback_id}",
                                    id = x["id"].as_str().unwrap(),
                                    name = truncate_and_pad(
                                        &format!("{}", x["name"].as_str().unwrap().cyan().bold()),
                                        40,
                                        40
                                    ),
                                    phase = truncate_and_pad(
                                        x["status"]["phase"].as_str().unwrap(),
                                        10,
                                        10
                                    )
                                    .white()
                                    .bold(),
                                    created_at = created_at_formatted.white().bold(),
                                    playback_id =
                                        x["playbackId"].as_str().unwrap_or("").white().bold(),
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
            "Export to IPFS",
            "< Back",
            "< Home",
        ])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(index) => {
            match index {
                0 => {
                    inspect_asset(Some(a.clone()), client);
                    assets(client);
                }

                1 => {
                    if let Ok(t) = task {
                        let pretty_task = serde_json::to_string_pretty(&t).unwrap();
                        println!("{}", pretty_task);
                        let single_task = t[0].clone();
                        crate::tasks::inspect_task(Some(single_task), client);
                    } else {
                        info!("Error getting task: {:?}", task);
                    }
                }

                3 => {
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

                4 => {
                    let export_result = client.asset.export_to_ipfs(
                        String::from(a["id"].as_str().unwrap()),
                        String::from("{}"),
                    );
                    if let Ok(e) = export_result {
                        let pretty_export_result = serde_json::to_string_pretty(&e).unwrap();
                        println!("{}", pretty_export_result);
                    } else {
                        error!("Error exporting to ipfs: {:?}", export_result);
                    }
                }

                5 => {
                    assets(client);
                }

                6 => {
                    crate::list_options(&client);
                    std::process::exit(0);
                }

                2 => {
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
                _ => {
                    error!("No selection made");
                }
            }
        }
        None => {
            error!("No selection made");
        }
    }
}

pub fn test_asset_flow(client: &livepeer_rs::Livepeer) -> bool {
    info!("Running asset flow test...");
    let current_folder_string = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let result = upload::do_upload(client, &current_folder_string, false);
    if !result.is_some() {
        println!("❌ - Error uploading asset");
        return false;
    }
    println!("✅ - Asset upload");
    println!("Polling task progress...");
    let res = result.unwrap();
    let asset_id = res.asset_id;
    let task_id = res.task_id;
    let playback_id = res.playback_id;

    let task_result = super::tasks::track_task_status(
        serde_json::from_str(&format!("{}{}{}", r#"{"id":""#, task_id, r#""}"#)).unwrap(),
        client,
    );
    if !task_result {
        println!("❌ - Task failed");
        return false;
    }
    println!("✅ - Task completed");
    let playback_info = client.playback.get_playback_info(&playback_id);
    if !playback_info.is_ok() {
        println!("❌ - Error getting playback info");
        return false;
    }
    println!("✅ - Got playback info");
    let export_result = client.asset.export_to_ipfs(asset_id, String::from("{}"));
    if !export_result.is_ok() {
        println!("❌ - Error exporting to ipfs");
        return false;
    }
    println!("✅ - Exported to ipfs");

    return true;
}
