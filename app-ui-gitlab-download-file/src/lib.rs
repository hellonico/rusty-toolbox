use gitlab::api::projects::repository::files::File;
use gitlab::api::{projects, Query};
use gitlab::{api, Gitlab};
use log::info;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use base64::Engine;
use base64::engine::general_purpose;

pub struct GitLabClient {
    client: Gitlab,
    project_name: String,
}

#[derive(Debug, Deserialize)]
struct Project {
    // name: String,
    id: i32,
}

#[derive(Deserialize)]
struct FileResponse {
    content: String,
}

impl GitLabClient {
    pub fn new(gitlab_host: &str, personal_token: &str, project_name: &str) -> Self {
        let cleaned_host = gitlab_host
            .strip_prefix("https://")
            .or_else(|| gitlab_host.strip_prefix("http://"))
            .unwrap_or(gitlab_host);
        let client = if gitlab_host.starts_with("https://") {
            Gitlab::new(cleaned_host, personal_token).expect("Failed to create GitLab client")
        } else if gitlab_host.starts_with("http://") {
            Gitlab::new_insecure(cleaned_host, personal_token).expect("Failed to create GitLab client")
        } else {
            panic!("Invalid GitLab host URL. Must start with 'https://' or 'http://'.");
        };
        Self {
            client,
            project_name: project_name.to_string(),
        }
    }

    pub fn display_folder_size(&self, output_folder: &str) -> f64 {
        let folder_size = folder_size(PathBuf::from(output_folder).as_path())
            .expect("Failed to compute folder size");
        let folder_size_mb = folder_size as f64 / 1_048_576.0;
        info!("Output folder [{}] size: {:.2} MB", output_folder, folder_size_mb);
        folder_size_mb
    }

    pub fn timed_download(&self, file_path: &str, branch: &str, output_folder: &str) -> Duration {
        let start = Instant::now();
        self.download(file_path, branch, output_folder);
        let duration = start.elapsed();
        info!("Download completed in: {:.2?}", duration);
        duration
    }

    pub fn download(&self, file_path: &str, branch: &str, output_folder: &str) {
        let endpoint = projects::Project::builder()
            .project(&self.project_name)
            .build()
            .unwrap();
        let project: Project = endpoint.query(&self.client).unwrap();

        info!("Downloading {:} from {:} [{:}]", file_path, self.project_name, project.id);

        let file_endpoint = File::builder()
            .project(project.id.to_string())
            .file_path(file_path)
            .ref_(branch)
            .build()
            .unwrap();
        let raw_json: Vec<u8> = api::raw(file_endpoint).query(&self.client).unwrap();
        let raw_json_str =
            String::from_utf8(raw_json).expect("Failed to convert raw JSON to string");
        let file_response: FileResponse =
            serde_json::from_str(&raw_json_str).expect("Failed to parse JSON");
        let decoded_content =
            general_purpose::STANDARD.decode(&file_response.content).expect("Failed to decode base64 content");
        let output_file_path = Path::new(output_folder).join(file_path);
        if let Some(parent_dir) = output_file_path.parent() {
            fs::create_dir_all(parent_dir).expect("Failed to create intermediary folders");
        }
        fs::write(&output_file_path, &decoded_content).expect("Failed to save the file");
    }
}

fn folder_size(path: &Path) -> std::io::Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                size += folder_size(&entry.path())?;
            } else {
                size += metadata.len();
            }
        }
    }
    Ok(size)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GitlabConfig {
    pub gitlab_host: String,
    pub personal_token: String,
    pub project_name: String,
    pub file_path: String,
    pub branch: String,
    pub output_folder: String,
}

pub fn load_config(file_path: &str) -> GitlabConfig {
    let config_content = fs::read_to_string(file_path)
        .expect("Failed to read the configuration file");
    serde_yaml::from_str(&config_content)
        .expect("Failed to parse the configuration file")
}