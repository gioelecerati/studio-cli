use livepeer_rs::vod::Vod;

pub mod resumable;

pub fn upload_asset(client: &livepeer_rs::Livepeer) {
    // Choose type of upload
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&["Upload from URL", "Upload from File", "< Back"])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(index) => {
            if index == 0 {
                upload_from_url(client);
            }
            if index == 1 {
                upload_from_file(client);
            }
            if index == 2 {
                super::assets(client);
            }
        }
        None => {
            error!("No selection made, going back");
        }
    }
}

pub fn upload_from_url(client: &livepeer_rs::Livepeer) {
    let url = dialoguer::Input::<String>::new()
        .with_prompt("Enter URL")
        .interact()
        .unwrap();

    let mut asset_name = dialoguer::Input::<String>::new()
        .with_prompt("Enter name for asset (blank for random id)")
        .default("livepeer_rs_import".to_string())
        .interact()
        .unwrap();

    let up_result = client.asset.import_asset(url, asset_name);

    if let Ok(a) = up_result {
        println!("Asset uploaded: {:?}", a);
    } else {
        error!("Error uploading asset: {:?}", up_result);
    }
    super::assets(client);
}

pub fn upload_from_file(client: &livepeer_rs::Livepeer) {
    // Choose between direct and resumable upload
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&["Direct Upload", "Resumable Upload", "< Back"])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(index) => {
            if index == 0 {
                let current_folder_string = std::env::current_dir()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                do_upload(client, &current_folder_string, false);
            }
            if index == 1 {
                println!("Not implemented yet, use direct uploads for now");
            }
            if index == 2 {
                upload_asset(client);
            }
        }
        None => {
            error!("No selection made, going back");
        }
    }
}

pub fn do_upload(client: &livepeer_rs::Livepeer, current_folder_string: &String, resumable: bool) {
    // read from disk recent-uploads
    let recent_string =
        crate::auth::get_string_from_disk(&String::from("recent"), &String::from("uploads"));
    let mut recents = None;
    if recent_string.is_some() {
        recents = Some(recent_string.unwrap());
    }
    let files = list_files_and_folders(current_folder_string, recents);

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
                do_upload(client, &parent_folder, resumable);
            } else {
                if files[index].ends_with("/") {
                    let new_folder = files[index].clone();
                    do_upload(client, &new_folder, resumable);
                } else {
                    let mut path_of_file = &files[index];
                    let file_name = std::path::Path::new(path_of_file)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();

                    let path_of_file = &path_of_file.replace("<RECENT> ", "").clone();

                    let recent_file_name = format!("<RECENT> {}", path_of_file);

                    crate::auth::save_string_to_disk(
                        &String::from("recent"),
                        &String::from("uploads"),
                        &recent_file_name,
                    );

                    // Ask a name for the asset
                    let mut asset_name = dialoguer::Input::<String>::new()
                        .with_prompt("Enter name for asset (blank to use file name)")
                        .default(file_name)
                        .interact()
                        .unwrap();

                    if asset_name == "" {
                        // create random string
                        asset_name = nanoid::nanoid!(10);
                    }

                    info!("Generating presigned urls");

                    let urls = client.asset.get_presigned_url(asset_name);

                    match urls {
                        Ok(urls) => {
                            // create indicatif spinner
                            let spinner = indicatif::ProgressBar::new_spinner();
                            spinner.enable_steady_tick(std::time::Duration::from_millis(120));
                            spinner.set_style(
                                indicatif::ProgressStyle::with_template("{spinner:.blue} {msg}")
                                    .unwrap()
                                    // For more spinners check out the cli-spinners project:
                                    // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
                                    .tick_strings(&[
                                        "▹▹▹▹▹",
                                        "▸▹▹▹▹",
                                        "▹▸▹▹▹",
                                        "▹▹▸▹▹",
                                        "▹▹▹▸▹",
                                        "▹▹▹▹▸",
                                        "▪▪▪▪▪",
                                    ]),
                            );
                            spinner.set_message("Uploading...");
                            let upload_url = String::from(urls["url"].as_str().unwrap());
                            // get absolute path of file
                            let path_of_file = std::path::Path::new(path_of_file)
                                .canonicalize()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string();

                            let mut up_result = Err(livepeer_rs::errors::Error::UNKNOWN);
                            if resumable {
                                error!("Resumable upload not implemented yet in livepeer_rs!")
                            } else {
                                up_result = client.asset.upload_asset(upload_url, path_of_file);
                            }

                            spinner.finish();

                            match up_result {
                                Ok(_) => {
                                    println!("Upload successful");
                                }
                                Err(e) => {
                                    error!("Error: {:?}", e);
                                }
                            }
                            super::assets(client);
                        }
                        Err(e) => {
                            error!("Error: {:?}", e);
                        }
                    }
                }
            }
        }
        None => {
            error!("No selection made, going back");
        }
    }
}

pub fn list_files_and_folders(path: &String, recents: Option<String>) -> Vec<String> {
    let mut files = vec![];

    // get all files and folders. if folder, add "/" to end
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let path_string = path.to_str().unwrap().to_string();
        if path.is_dir() {
            if path_string.contains("/.") {
                continue;
            }
            files.push(format!("{}/", path_string));
        } else {
            let video_extensions = vec![
                "mp4", "mkv", "avi", "mov", "flv", "wmv", "webm", "mpeg", "mpg", "m4v", "3gp",
            ];

            // if file has extension

            let extension = path.extension();

            println!("Extension: {:?}", extension);
            if extension.is_none() {
                files.push(path_string.clone());
                continue;
            }

            let ext = extension.unwrap().to_str();

            if ext.is_none() {
                files.push(path_string.clone());
                continue;
            }

            if !video_extensions.contains(&ext.unwrap()) {
                continue;
            }

            // if file contains "/." (hidden file), skip
            if path_string.contains("/.") {
                continue;
            }

            files.push(path_string);
        }
    }

    // Reorder files alphabetically
    files = files.iter().map(|x| x.to_string()).collect::<Vec<String>>();

    files.sort();

    use std::cmp::Ordering;

    // Reorder files to have directories first
    files.sort_by(|a, b| {
        if a.ends_with("/") && !b.ends_with("/") {
            Ordering::Less
        } else if !a.ends_with("/") && b.ends_with("/") {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    // Put .. at the top
    files.insert(0, String::from(".."));

    if recents.is_some() {
        let recent = recents.unwrap();
        files.insert(1, recent);
    }

    return files;
}
