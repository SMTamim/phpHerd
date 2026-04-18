use std::path::Path;

/// Detect the framework used by a project directory
pub fn detect_framework(project_path: &Path) -> FrameworkType {
    if project_path.join("artisan").exists() {
        FrameworkType::Laravel
    } else if project_path.join("wp-config.php").exists()
        || project_path.join("wp-config-sample.php").exists()
    {
        FrameworkType::WordPress
    } else if project_path.join("symfony.lock").exists()
        || project_path.join("bin").join("console").exists()
    {
        FrameworkType::Symfony
    } else if project_path.join("index.php").exists() {
        FrameworkType::StaticPhp
    } else if project_path.join("index.html").exists() {
        FrameworkType::StaticHtml
    } else {
        FrameworkType::Unknown
    }
}

/// Get the document root for a framework
pub fn document_root(project_path: &Path, framework: &FrameworkType) -> String {
    let root = match framework {
        FrameworkType::Laravel => project_path.join("public"),
        FrameworkType::Symfony => project_path.join("public"),
        FrameworkType::WordPress => project_path.to_path_buf(),
        FrameworkType::StaticPhp => project_path.to_path_buf(),
        FrameworkType::StaticHtml => project_path.to_path_buf(),
        FrameworkType::Unknown => project_path.join("public"),
    };
    root.to_string_lossy().to_string()
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrameworkType {
    Laravel,
    WordPress,
    Symfony,
    StaticPhp,
    StaticHtml,
    Unknown,
}
