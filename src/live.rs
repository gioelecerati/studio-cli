use colored::*;
use livepeer_rs::vod::{Task, Vod};

const REGIONS: &'static [&'static str] = &["fra", "prg", "nyc", "lon", "lax", "mdw", "sin", "sao"];

pub fn streams(client: &livepeer_rs::Livepeer) -> bool {
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&[
            "My Streams",
            "Get Streams by User ID",
            "Get Stream By ID",
            "Create Stream",
            "< Back",
        ])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(index) => {
            let mut stream_list = vec![];
            let mut stream: Option<serde_json::Value> = None;
            let mut e: Option<_> = None;

            if index == 0 {
                let user_id = client.user.user_id.clone();
                let stream_value = client.stream.clone().get_streams_by_user_id(user_id);

                if let Ok(a) = stream_value {
                    stream_list = a;
                } else {
                    error!("Error getting streams: {:?}", stream_value);
                    e = Some(());
                }
            }

            if index == 1 {
                let user_id = dialoguer::Input::<String>::new()
                    .with_prompt("Enter user ID")
                    .interact()
                    .unwrap();

                let stream_value = client.stream.clone().get_streams_by_user_id(user_id);

                if let Ok(a) = stream_value {
                    stream_list = a;
                } else {
                    error!("Error getting assets: {:?}", stream_value);
                    e = Some(());
                }
            }

            if index == 2 {
                let stream_id = dialoguer::Input::<String>::new()
                    .with_prompt("Enter stream ID")
                    .interact()
                    .unwrap();
                let single_stream = client
                    .stream
                    .clone()
                    .get_stream_by_id(String::from(stream_id));
                if let Ok(a) = single_stream {
                    stream = Some(a);
                } else {
                    error!("Error getting asset: {:?}", single_stream);
                    e = Some(());
                }
            }

            if stream.is_some() {
                inspect_stream(stream, client);
                streams(client);
            }

            if index == 3 {
                // Create stream
                let name = dialoguer::Input::<String>::new()
                    .with_prompt("Enter stream name")
                    .interact()
                    .unwrap();

                client.stream.clone().create_stream(
                    &name,
                    &vec![livepeer_rs::data::stream::Profile {
                        bitrate: 250000,
                        fps: 0,
                        height: 240,
                        name: String::from("240p0"),
                        width: 426,
                        gop: None,
                    }],
                );

                streams(client);
                std::process::exit(0);
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
                    let list = stream_list;
                    if index == 0 {
                        // length of list
                        let count = list.len();
                        info!("Streams found: {}", count);
                        streams(client);
                    }

                    if list.len() == 0 {
                        warn!("No streams found");
                        crate::init();
                        std::process::exit(0);
                    }

                    if index == 1 {
                        // selection from list by id
                        let ids = list
                            .iter()
                            .map(|x| {
                                format!(
                                    "{} - {:?} - {} - {:?} - {}",
                                    x.id,
                                    x.stream_key.clone(),
                                    x.name,
                                    x.playback_id.clone(),
                                    x.is_active
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
                                let id = list[index].id.clone();
                                let stream_value =
                                    client.stream.clone().get_stream_by_id(String::from(id));

                                if let Ok(a) = stream_value {
                                    stream = Some(a);
                                    inspect_stream(stream, client);
                                } else {
                                    error!("Error getting stream: {:?}", stream_value);
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
    streams(client);
    return false;
}

pub fn inspect_stream(stream: Option<serde_json::Value>, client: &livepeer_rs::Livepeer) {
    let a = stream.unwrap();
    let task = client
        .task
        .get_task_by_output_asset_id(String::from(a["id"].as_str().unwrap()));

    let pretty_asset = serde_json::to_string_pretty(&a).unwrap();
    println!("{}", pretty_asset);

    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&[
            "< Back",
            "< Home",
            "Playback Stream",
            "Change policy",
            "Push",
            "Test Push into Region",
            "Get running push",
            "Test on all regions",
        ])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(index) => {
            if index == 0 {
                streams(client);
            }

            if index == 1 {
                crate::list_options(&client);
                std::process::exit(0);
            }

            if index == 2 {
                // run command ffplay with playbackURL
                let playback_url = a["playbackUrl"].as_str();

                match playback_url {
                    Some(url) => {
                        info!("Playback URL: {}", url);
                        info!("Playing stream...");
                        info!("Wait for ffplay to load...");
                        let output = std::process::Command::new("ffplay")
                            .arg(url)
                            .output()
                            .expect("failed to execute process");
                    }
                    None => {
                        error!("No playback URL found");
                        streams(client);
                    }
                }

                streams(client);
            }

            if index == 3 {
                // print todo
                info!("TODO Change policy");
                streams(client);
            }

            if index == 4 {
                let ffmpeg_path = get_ffmpeg_path();
                let stream_client = client.clone();

                let current_folder_string = std::env::current_dir()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                let files = crate::assets::upload::list_files_and_folders(&current_folder_string);

                let file_to_push = get_file_to_push(&current_folder_string);
                let stream = a.clone();
                if let Ok(ffp) = ffmpeg_path {
                    //std::thread::spawn(move || {
                    stream_client.rtmp.push_to_region(
                        &stream["streamKey"].to_string().replace("\"", ""),
                        &file_to_push.to_string(),
                        &String::from(REGIONS[0]),
                        &ffp,
                        &mut Some(stream["playbackId"].to_string().replace('"', "")),
                    );
                    //});
                    streams(client);
                } else {
                    error!("FFMPEG not found");
                    streams(client);
                }
            }

            if index == 5 {
                // print todo
                // test single region
                let index =
                    dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .items(&REGIONS)
                        .default(0)
                        .interact_on_opt(&crate::Term::stderr())
                        .unwrap();

                let region_vec = REGIONS.to_vec();

                let region_selected = region_vec[index.unwrap()].to_string();

                let ffmpeg_path = get_ffmpeg_path();
                let stream_client = client.clone();

                let current_folder_string = std::env::current_dir()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                let files = crate::assets::upload::list_files_and_folders(&current_folder_string);

                let file_to_push = get_file_to_push(&current_folder_string);
                let stream = a.clone();

                if let Ok(ffp) = ffmpeg_path.clone() {
                    let push = stream_client.rtmp.push_to_region(
                        &stream["streamKey"].to_string().replace("\"", ""),
                        &file_to_push.to_string(),
                        &region_selected.to_string(),
                        &ffp,
                        &mut Some(stream["playbackId"].to_string().replace('"', "")),
                    );

                    if let Ok(e) = push {
                        info!("Push to region {} successful", region_selected);
                    } else {
                        error!("Push to region {} failed", region_selected);
                        error!("Status: {:?}", push);
                    }

                    // sleep 3 seconds
                    std::thread::sleep(std::time::Duration::from_secs(3));
                } else {
                    error!("FFMPEG not found");
                    streams(client);
                }

                streams(client);
            }

            if index == 6 {
                // print todo
                find_threads(&a["playbackId"].to_string().replace('"', ""));
                streams(client);
            }

            if index == 7 {
                let ffmpeg_path = get_ffmpeg_path();
                let stream_client = client.clone();

                let current_folder_string = std::env::current_dir()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                let files = crate::assets::upload::list_files_and_folders(&current_folder_string);

                let file_to_push = get_file_to_push(&current_folder_string);
                let stream = a.clone();
                for region in REGIONS {
                    info!("Testing region: {}", region);
                    if let Ok(ffp) = ffmpeg_path.clone() {
                        let push = stream_client.rtmp.push_to_region(
                            &stream["streamKey"].to_string().replace("\"", ""),
                            &file_to_push.to_string(),
                            &region.to_string(),
                            &ffp,
                            &mut Some(stream["playbackId"].to_string().replace('"', "")),
                        );

                        if let Ok(e) = push {
                            info!("Push to region {} successful", region);
                        } else {
                            error!("Push to region {} failed", region);
                            error!("Status: {:?}", push);
                        }

                        // sleep 3 seconds
                        std::thread::sleep(std::time::Duration::from_secs(3));
                    } else {
                        error!("FFMPEG not found");
                        streams(client);
                    }
                }

                streams(client);
            }
        }
        None => {
            error!("No selection made");
        }
    }
}

pub fn get_file_to_push(current_folder_string: &String) -> String {
    let files = crate::assets::upload::list_files_and_folders(&current_folder_string);
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&files)
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();
    match selection {
        // Match selection
        // If selected path is 0 (..), use the parent folder as work dir and call list_files again
        // If selected path is a folder, use it as work dir and call list_files again
        // If selected path is a file, upload it
        Some(index) => {
            if index == 0 {
                let parent_folder = std::path::Path::new(current_folder_string)
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                return get_file_to_push(&parent_folder);
            } else {
                if files[index].ends_with("/") {
                    let new_folder = files[index].clone();
                    return get_file_to_push(&new_folder);
                } else {
                    let path_of_file = &files[index];
                    let file_name = std::path::Path::new(path_of_file)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    return path_of_file.to_string();
                }
            }
        }
        None => {
            error!("No selection made, going back");
            return String::from("/tmp/video.mp4");
        }
    }
}

pub fn get_ffmpeg_path() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let ffmpeg_path = which::which("ffmpeg");
        if ffmpeg_path.is_ok() {
            let path = ffmpeg_path.unwrap().to_str().unwrap().to_string();
            let mut ffmpeg_path_ref = FFMPEG_PATH.lock().unwrap();
            *ffmpeg_path_ref = Some(path.clone());
            return format!("{}", path);
        } else {
            return no_ffmpeg_in_path;
        }
    }

    #[cfg(target_os = "linux")]
    {
        let ffmpeg_path = which::which("ffmpeg");
        if ffmpeg_path.is_ok() {
            let path = ffmpeg_path.unwrap().to_str().unwrap().to_string();
            return Ok(format!("{}", path));
        } else {
            return Err("No ffmpeg in path".to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        let ffmpeg_path = which::which("ffmpeg");
        if ffmpeg_path.is_ok() {
            let path = ffmpeg_path.unwrap().to_str().unwrap().to_string();
            return Ok(format!("{}", path));
        } else {
            return Err("No ffmpeg in path".to_string());
        }
    }
}

pub fn get_ffplay_path() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let ffplay_path = which::which("ffplay");
        if ffplay_path.is_ok() {
            let path = ffplay_path.unwrap().to_str().unwrap().to_string();
            return Ok(format!("{}", path));
        } else {
            return Err("No ffplay in path".to_string());
        }
    }

    #[cfg(target_os = "linux")]
    {
        let ffplay_path = which::which("ffplay");
        if ffplay_path.is_ok() {
            let path = ffplay_path.unwrap().to_str().unwrap().to_string();
            return Ok(format!("{}", path));
        } else {
            return Err("No ffplay in path".to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        let ffplay_path = which::which("ffplay");
        if ffplay_path.is_ok() {
            let path = ffplay_path.unwrap().to_str().unwrap().to_string();
            return Ok(format!("{}", path));
        } else {
            return Err("No ffplay in path".to_string());
        }
    }
}

pub fn find_threads(thread_id: &str) -> String {
    // ps awxx | grep "d7ea1fe6"
    let mut cmd = std::process::Command::new("ps");
    cmd.arg("awxx");
    cmd.arg("|");
    cmd.arg("grep");
    cmd.arg(format!("{}", thread_id));

    println!("running command {:?}", cmd);

    let output = cmd.output().expect("failed to execute process");

    let output_string = String::from_utf8_lossy(&output.stdout);
    println!("output thread search: {}", output_string);
    return output_string.to_string();
}
