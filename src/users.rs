use livepeer_rs::user::UserTrait;

pub fn users(client: &livepeer_rs::Livepeer) {
    // Select functionality {my info}
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&["My Info", "Get user info by ID", "< Back"])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(index) => {
            if index == 0 {
                let user_info = &client.user.info;
                let s_value_user_info = serde_json::to_string_pretty(&user_info).unwrap();
                println!("{}", s_value_user_info);
            }

            if index == 1 {
                let user_id = dialoguer::Input::<String>::new()
                    .with_prompt("Enter user ID")
                    .interact()
                    .unwrap();

                let user_info = &client.user_api.get_user_info_by_id(user_id);

                if let Ok(info) = user_info {
                    let s_value_user_info = serde_json::to_string_pretty(&info).unwrap();
                    println!("{}", s_value_user_info);
                } else {
                    error!("Error getting user info: {:?}", user_info);
                }
            }

            if index == 2 {
                crate::init();
                std::process::exit(0);
            }
        }
        None => {
            error!("No selection made");
        }
    }

    users(client);
}
