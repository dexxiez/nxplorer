use serde_json::Value;
use std::{fs, path::Path};

use std::error::Error;
use std::fmt;

use crate::utils::find_files;

use super::frameworks::KNOWN_FRAMEWORKS;

#[derive(Debug)]
pub enum ProjectError {
    IoError(std::io::Error),
    JsonParseError(serde_json::Error),
    MissingField(&'static str),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::JsonParseError(e) => write!(f, "JSON parsing error: {}", e),
            Self::MissingField(field) => write!(f, "Missing required field: {}", field),
        }
    }
}

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

impl Error for ProjectError {}

// Some helpful conversions
impl From<std::io::Error> for ProjectError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<serde_json::Error> for ProjectError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonParseError(err)
    }
}

impl Project {
    pub fn detect(base_repo_path: &Path) -> Vec<Project> {
        let mut projects = Vec::new();
        let project_json_paths = find_files(base_repo_path, &["project.json"]);

        for project_json_path in project_json_paths {
            let containing_path = match Path::new(&project_json_path).parent() {
                Some(path) => path,
                None => continue,
            };

            // Handle the Result from parse_config
            match Project::parse_config(&project_json_path) {
                Ok(mut project) => {
                    project.framework = Self::detect_framework(&containing_path);
                    if project.framework.is_none() {
                        project.framework = Self::deep_detect_framework(&containing_path);
                    }
                    projects.push(project);
                }
                Err(e) => {
                    eprintln!("Failed to parse project {}: {}", project_json_path, e);
                    continue;
                }
            }
        }

        projects
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

    fn parse_config(project_json_path: &String) -> Result<Project, ProjectError> {
        let project_config = fs::read_to_string(project_json_path)?;

        let v: Value = serde_json::from_str(&project_config)?;

        let name = v["name"]
            .as_str()
            .ok_or(ProjectError::MissingField("name"))?
            .to_string();

        let project_type = v["projectType"]
            .as_str()
            .ok_or(ProjectError::MissingField("projectType"))
            .map(|t| match t {
                "library" => ProjectType::Library,
                _other => ProjectType::Application,
            })?;

        let tasks = Self::get_tasks(&project_config)
            .unwrap_or_default()
            .into_iter()
            .collect();

        Ok(Project {
            name,
            project_type,
            tasks,
            framework: None,
        })
    }
}
