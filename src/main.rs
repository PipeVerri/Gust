mod project;

fn main() {
    if let Err(e) = project::Project::setup_project() {
        eprintln!("Error: {}", e);
    }
}