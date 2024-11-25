use serde_json::Value;
use std::{fs, path::Path};

use crate::utils::find_files;

#[derive(Debug)]
pub struct Task {
    pub command: String,
}

#[derive(Clone, Debug)]
pub struct Framework {
    pub name: &'static str,
    pub identity_files: &'static [&'static str],
    pub identity_keywords: &'static [&'static str],
}

pub struct Project {
    pub name: String,
    pub tasks: Vec<Task>,
    // optional framework for special cases
    pub framework: Option<Framework>,
}

pub const KNOWN_FRAMEWORKS: &[Framework] = &[
    Framework {
        name: "nextjs",
        identity_files: &[
            "next.config.js",
            "next.config.ts",
            "next.config.mjs",
            "next.config.cjs",
        ],
        identity_keywords: &[],
    },
    Framework {
        name: "nuxt",
        identity_files: &["nuxt.config.js", "nuxt.config.ts"],
        identity_keywords: &[],
    },
    Framework {
        name: "angular",
        identity_files: &["angular.json"],
        identity_keywords: &["@angular"],
    },
];
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

            if !framework.identity_keywords.is_empty()
                && fs::read_to_string(project_path.join("project.json"))
                    .expect("Failed to read project.json file")
                    .contains(framework.identity_keywords.iter().next().unwrap())
            {
                return Some(framework.clone());
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

        let tasks = match v.get("tasks").and_then(|t| t.as_object()) {
            Some(task_map) => task_map,
            None => return None,
        };

        let parsed_tasks: Option<Vec<Task>> = tasks
            .iter()
            .map(|(key, value)| {
                // Now you have both the key and the value
                println!("Processing task: {}", key);

                value.get("command").and_then(|cmd| {
                    Some(Task {
                        command: cmd.to_string(),
                    })
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
            name: v["name"].to_string(),
            tasks: Self::get_tasks(&project_config)
                .into_iter()
                .flatten()
                .collect(),
            framework: None,
        }
    }
}
