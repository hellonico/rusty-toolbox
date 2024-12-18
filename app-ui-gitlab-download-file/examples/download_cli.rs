use std::env;
use app_ui_gitlab_download_file::{load_config, GitLabClient};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <config.yaml>", args[0]);
        std::process::exit(1);
    }

    let config_path = &args[1];
    let config = load_config(config_path);
    env_logger::init();

    let client = GitLabClient::new(&config.gitlab_host, &config.personal_token, &config.project_name);
    client.timed_download(&config.file_path, &config.branch, &config.output_folder);
    client.display_folder_size(&config.output_folder);
}
