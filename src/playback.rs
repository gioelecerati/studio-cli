use colored::*;
use livepeer_rs::playback::Playback;

pub fn playback(playback_info: serde_json::Value, client: &livepeer_rs::Livepeer) {
    let sources = playback_info["meta"]["source"].as_array().unwrap();
    let mut urls_hrns: Vec<(String, String)> = sources.iter()
        .map(|source| (
            source["url"].as_str().unwrap().to_string(),
            source["hrn"].as_str().unwrap().to_string()
        ))
        .collect();

    let mut strings_to_select: Vec<String> = vec![String::from("< Back")];
    strings_to_select.extend(urls_hrns.iter().map(|(url, hrn)| format!("{} - {}", hrn, url)));

    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select a URL to play")
        .items(&strings_to_select)
        .default(0)
        .interact()
        .unwrap();

    if selection == 0 {
        crate::init();
    } else {
        let playback_url = &urls_hrns[selection - 1].0;
        match crate::live::get_ffplay_path() {
            Ok(ffplay) => {
                info!("Playback URL: {}", playback_url);
                info!("Playing asset...");
                info!("Wait for ffplay to load...");
                std::process::Command::new(ffplay)
                    .arg(playback_url)
                    .output()
                    .expect("failed to execute process");
                crate::assets::assets(client);
            }
            Err(_) => {
                error!("ffplay not found");
                playback(playback_info.clone(), client);
            }
        }
    }
}

pub fn playbacks(client: &livepeer_rs::Livepeer) {
    let playback_id = dialoguer::Input::<String>::new()
        .with_prompt("Enter Playback ID or CID")
        .interact()
        .unwrap();
    match client.playback.get_playback_info(&playback_id) {
        Ok(p) => {
            println!("{}", serde_json::to_string_pretty(&p).unwrap());
            playback(p, client);
        }
        Err(e) => error!("Error getting playback info: {:?}", e),
    }
}
