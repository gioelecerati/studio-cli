use colored::*;

pub fn playback(playbackInfo: serde_json::Value, client: &livepeer_rs::Livepeer) {
    let mut urls = Vec::new();
    let mut hrns = Vec::new();

    let sources = playbackInfo["meta"]["source"].as_array().unwrap();
    for source in sources {
        let hrn = source["hrn"].as_str().unwrap();
        let url = source["url"].as_str().unwrap();
        urls.push(url);
        hrns.push(hrn);
    }

    let mut strings_to_select: Vec<String> = vec![String::from("< Back")];
    let mut c = 0;
    for hrn in hrns {
        strings_to_select.push(format!("{} - {}", hrn, urls[c]));
        c += 1;
    }

    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select a URL to play")
        .items(&strings_to_select[..])
        .default(0)
        .interact()
        .unwrap();

    if selection == 0 {
        crate::init();
    } else {
        let playback_url = urls[selection - 1];
        let ffplay_path = crate::live::get_ffplay_path();

        if ffplay_path.is_err() {
            error!("ffplay not found");
            playback(playbackInfo.clone(), client);
        }

        let ffplay = ffplay_path.unwrap();

        info!("Playback URL: {}", playback_url);
        info!("Playing asset...");
        info!("Wait for ffplay to load...");
        let output = std::process::Command::new(ffplay)
            .arg(playback_url)
            .output()
            .expect("failed to execute process");
        crate::assets::assets(client);
    }
}
