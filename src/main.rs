mod detection;
mod utils;

use std::path::Path;

fn main() {
    let start_path = Path::new("/home/toby/Documents/code/work/tigertrace"); // Change this to "." for current directory
    search(start_path);
}

fn search(path: &Path) {
    let projects = detection::Project::detect(path);
    for project in projects {
        let framework_name = project.framework.map_or("None", |f| f.name);
        println!(
            "Project {:#} found with framework {:#} and has {:?} tasks",
            project.name, framework_name, project.tasks
        );
    }
}
