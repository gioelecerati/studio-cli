use livepeer_rs::ai::Generate;

pub fn generate(client: &livepeer_rs::Livepeer) {
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&[
            "Text to Image",
            "< Back",
        ])
        .default(0)
        .interact_on_opt(&crate::Term::stderr())
        .unwrap();
    let index = selection.unwrap();
    match index {
        0 => {
            let prompt = dialoguer::Input::<String>::new()
                .with_prompt("Insert a prompt")
                .interact()
                .unwrap();
            let response = client.generate.text_to_image(&prompt);
            if let Ok(r) = response {
                println!("{}", r);
            } else {
                error!("Error generating image: {:?}", response);
                generate(client);
            }
        }
        1 => {
            crate::list_options(&client);
            std::process::exit(0);
        }
        _ => {
            println!("Invalid option");
        }
    }
}