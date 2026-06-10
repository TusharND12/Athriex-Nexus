use std::path::Path;

pub fn detect_language(path: &Path) -> Option<&'static str> {
    path.extension().and_then(|ext| match ext.to_str()? {
        "rs" => Some("rust"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "ts" | "mts" | "cts" => Some("typescript"),
        "tsx" => Some("tsx"),
        "py" | "pyw" => Some("python"),
        "go" => Some("go"),
        "java" => Some("java"),
        "rb" => Some("ruby"),
        "php" => Some("php"),
        "cs" => Some("csharp"),
        "cpp" | "cc" | "cxx" => Some("cpp"),
        "c" | "h" => Some("c"),
        "swift" => Some("swift"),
        "kt" | "kts" => Some("kotlin"),
        "sql" => Some("sql"),
        "sh" | "bash" => Some("shell"),
        "toml" => Some("toml"),
        "json" => Some("json"),
        "yaml" | "yml" => Some("yaml"),
        "md" => Some("markdown"),
        _ => None,
    })
}

pub fn is_source_file(path: &Path) -> bool {
    detect_language(path).is_some()
}

const SKIP_DIRS: &[&str] = &[
    ".git",
    ".nexus",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    "__pycache__",
    ".venv",
    "venv",
    "vendor",
    ".cargo",
];

pub fn should_skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name) || name.starts_with('.') && name != ".github"
}
