use serde_json::Value;
use std::{fs, path::Path};

use crate::utils::find_files;

use super::frameworks::KNOWN_FRAMEWORKS;

#[derive(Debug, Clone)]
pub enum ProjectType {
    Application,
    Library,
}

#[derive(Debug)]
pub struct Task {
    pub command: String,
    pub subcommands: Vec<String>,
}
#[derive(Debug)]
pub struct DeepDetectionMatcher {
    pub path: &'static str,
    pub keyword: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct Framework {
    pub name: &'static str,
    pub identity_files: &'static [&'static str],
    pub proj_identity_keywords: &'static [&'static str],
    pub deep_detection_matchers: &'static [DeepDetectionMatcher],
    pub commands: &'static [&'static str],
}

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub project_type: ProjectType,
    pub tasks: Vec<Task>,
    pub framework: Option<Framework>,
}

impl Project {
    pub fn detect(base_repo_path: &Path) -> Vec<Project> {
        let mut projects = Vec::new();

        let project_json_paths = find_files(base_repo_path, &["project.json"]);

        for project_json_path in project_json_paths {
            let containing_path = Path::new(&project_json_path)
                .parent()
                .expect("Failed to get parent path");

            let mut project = Project::parse_config(&project_json_path);

            project.framework = Self::detect_framework(&containing_path);
            if project.framework.is_none() {
                project.framework = Self::deep_detect_framework(&containing_path);
            }
            projects.push(project);
        }

        return projects;
    }

    fn detect_framework(project_path: &Path) -> Option<Framework> {
        for framework in KNOWN_FRAMEWORKS {
            for identity_file in framework.identity_files {
                let identity_file_path = project_path.join(identity_file);
                if identity_file_path.exists() {
                    return Some(framework.clone());
                }
            }

            if !framework.proj_identity_keywords.is_empty()
                && fs::read_to_string(project_path.join("project.json"))
                    .expect("Failed to read project.json file")
                    .contains(framework.proj_identity_keywords.iter().next().unwrap())
            {
                return Some(framework.clone());
            }
        }

        None
    }

    fn deep_detect_framework(project_path: &Path) -> Option<Framework> {
        for framework in KNOWN_FRAMEWORKS {
            for matcher in framework.deep_detection_matchers {
                let matcher_path = project_path.join(&matcher.path);
                if matcher_path.exists() {
                    let matcher_content =
                        fs::read_to_string(&matcher_path).expect("Failed to read matcher file");
                    if matcher_content.contains(&matcher.keyword) {
                        return Some(framework.clone());
                    }
                }
            }
        }

        None
    }

    fn get_tasks(project_json_content: &String) -> Option<Vec<Task>> {
        // Try to parse the JSON, if it fails -> peace out with None
        let v: Value = match serde_json::from_str(project_json_content) {
            Ok(parsed) => parsed,
            Err(_) => return None,
        };

        let tasks = match v.get("targets").and_then(|t| t.as_object()) {
            Some(task_map) => task_map,
            None => return None,
        };

        let parsed_tasks: Option<Vec<Task>> = tasks
            .iter()
            .map(|(key, value)| {
                if value.get("configurations") != None {
                    let subcommands = value["configurations"]
                        .as_object()
                        .unwrap()
                        .iter()
                        .map(|(key, _)| key.to_string())
                        .collect();
                    return Some(Task {
                        command: key.to_string(),
                        subcommands,
                    });
                }
                Some(Task {
                    command: key.to_string(),
                    subcommands: vec![],
                })
            })
            .collect(); // collect will return None if any task conversion failed

        parsed_tasks
    }

    fn parse_config(project_json_path: &String) -> Project {
        let project_config =
            fs::read_to_string(project_json_path).expect("Failed to read project.json file");
        let v: Value =
            serde_json::from_str(&project_config).expect("Failed to parse project.json file");
        Project {
            name: v["name"].as_str().unwrap().to_string(),
            project_type: match v["projectType"].as_str().unwrap() {
                "library" => ProjectType::Library,
                _ => ProjectType::Application,
            },
            tasks: Self::get_tasks(&project_config)
                .into_iter()
                .flatten()
                .collect(),
            framework: None,
        }
    }
}
