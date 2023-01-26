use livepeer_rs::vod::{Task, Vod};

pub fn tasks(client: &livepeer_rs::Livepeer) {
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&[
            "My Tasks",
            "Get Tasks by User ID",
            "Get Task By ID",
            "< Back",
        ])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(index) => {
            let mut tasks_list = serde_json::Value::Null;
            let mut task: Option<serde_json::Value> = None;
            let mut e: Option<_> = None;

            if index == 0 {
                // Get my tasks
                let tasks_value = client.task.list_tasks();

                if let Ok(a) = tasks_value {
                    tasks_list = a;
                } else {
                    error!("Error getting tasks: {:?}", tasks_value);
                    e = Some(());
                }
            }

            if index == 1 {
                // Get tasks by user ID
                let user_id = dialoguer::Input::<String>::new()
                    .with_prompt("Enter user ID")
                    .interact()
                    .unwrap();

                let tasks_value = client.task.get_tasks_by_user_id(user_id);

                if let Ok(a) = tasks_value {
                    tasks_list = a;
                } else {
                    info!("Error getting tasks: {:?}", tasks_value);
                    e = Some(());
                }
            }

            if index == 2 {
                // Get task by task ID
                let task_id = dialoguer::Input::<String>::new()
                    .with_prompt("Enter task ID")
                    .interact()
                    .unwrap();

                let task_value = client.task.get_task_by_id(task_id);

                if let Ok(a) = task_value {
                    task = Some(a);
                } else {
                    error!("Error getting task: {:?}", task_value);
                    e = Some(());
                }
            }

            if task.is_some() {
                inspect_task(task, client);
                tasks(client)
            }

            if index == 3 {
                crate::init();
                std::process::exit(0);
            }

            let selection =
                dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .items(&["Count", "List", "< Back"])
                    .default(0)
                    .interact_on_opt(&crate::Term::stderr())
                    .unwrap();

            match selection {
                Some(i) => {
                    let list = tasks_list.as_array().unwrap();
                    if i == 0 {
                        // length of list
                        let count = list.len();
                        info!("Tasks found: {}", count);
                        tasks(client);
                    }

                    if list.len() == 0 {
                        warn!("No tasks found");
                        crate::init();
                        std::process::exit(0);
                    }

                    if i == 1 {
                        let ids = list
                            .iter()
                            .map(|x| {
                                format!(
                                    "{} - {}",
                                    x["id"].as_str().unwrap(),
                                    x["status"]["phase"].as_str().unwrap()
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
                                let task_value = client.task.get_task_by_id(String::from(id));

                                if let Ok(a) = task_value {
                                    task = Some(a);
                                    inspect_task(task, client)
                                } else {
                                    error!("Error getting task: {:?}", task_value);
                                    e = Some(())
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
            info!("No selection made");
        }
    }
}

pub fn inspect_task(task: Option<serde_json::Value>, client: &livepeer_rs::Livepeer) {
    let a = task.unwrap();
    let asset = client
        .asset
        .get_asset_by_id(String::from(a["outputAssetId"].as_str().unwrap()));

    let pretty_task = serde_json::to_string_pretty(&a).unwrap();
    println!("{}", pretty_task);

    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&["Get output asset", "Track task status", "< Back", "< Home"])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(index) => {
            if index == 0 {
                if let Ok(t) = asset {
                    let pretty_asset = serde_json::to_string_pretty(&t).unwrap();
                    println!("{}", pretty_asset);
                } else {
                    error!("Error getting asset: {:?}", asset);
                }
            }

            if index == 1 {
                track_task_status(a, client);
            }

            if index == 2 {
                tasks(client);
            }

            if index == 3 {
                crate::init();
                std::process::exit(0);
            }
        }
        None => {
            info!("No selection made");
        }
    }
}

use std::{cmp::min, fmt::Write};

pub fn track_task_status(task: serde_json::Value, client: &livepeer_rs::Livepeer) {
    // Get task.id, then get task from livepeer client and check status.phase and status.progress
    // Spawn a indicatif progress bar and update it with the progress value
    // Stay in this function until status.phase is "completed" or "failed" & then return to tasks menu

    let task_id = task["id"].as_str().unwrap();
    let mut task = client.task.get_task_by_id(String::from(task_id));

    let pb = indicatif::ProgressBar::new(100);
    pb.set_style(indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {msg}")
        .unwrap()
        .progress_chars("#>-"));
    pb.enable_steady_tick(std::time::Duration::from_millis(120));


    loop {
        if let Ok(t) = task {
            let phase = t["status"]["phase"].as_str().unwrap();
            let progress = t["status"]["progress"].as_f64();

            match progress {
                Some(p) => {
                    let pro = p * 100.0;
                    let percentage = pro as u64;
                    let pstring = format!("{}%", percentage);
                    pb.set_position(percentage);
                    pb.set_message(pstring);
                }
                None => {
                    if phase == "completed" || phase == "failed" {
                        pb.finish();
                        break;
                    }

                    debug!("No progress value found");

                    std::thread::sleep(std::time::Duration::from_secs(3));
                }
            }

            if phase == "completed" || phase == "failed" {
                pb.finish();
                println!("Task status completion reached: {}", phase);
                if phase == "failed" {
                    error!("Task failed");
                }
                break;
            }
        } else {
            debug!("Error getting task: {:?}", task);
            break;
        }

        task = client.task.get_task_by_id(String::from(task_id));

        // sleep 3 seconds
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
