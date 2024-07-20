use livepeer_rs::user::UserTrait;

pub fn users(client: &livepeer_rs::Livepeer) {
    let options = ["My Info", "Get user info by ID", "< Back"];
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&options)
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();

    match selection {
        Some(0) => {
            let user_info = &client.user.info;
            println!("{}", serde_json::to_string_pretty(&user_info).unwrap());
        }
        Some(1) => {
            let user_id = dialoguer::Input::<String>::new()
                .with_prompt("Enter user ID")
                .interact()
                .unwrap();

            match client.user_api.get_user_info_by_id(user_id) {
                Ok(info) => println!("{}", serde_json::to_string_pretty(&info).unwrap()),
                Err(e) => error!("Error getting user info: {:?}", e),
            }
        }
        Some(2) => {
            crate::list_options(&client);
            std::process::exit(0);
        }
        Some(_) => error!("Invalid selection"),
        None => error!("No selection made"),
    }

    users(client);
}