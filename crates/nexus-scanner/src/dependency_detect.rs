use std::collections::HashMap;
use std::fs;
use std::path::Path;

use nexus_core::DependencyInfo;
use regex::Regex;

pub fn detect_dependencies(project_root: &Path) -> Vec<DependencyInfo> {
    let mut deps = Vec::new();
    deps.extend(parse_cargo_toml(&project_root.join("Cargo.toml")));
    deps.extend(parse_package_json(&project_root.join("package.json")));
    deps.extend(parse_pyproject(&project_root.join("pyproject.toml")));
    deps.extend(parse_requirements(&project_root.join("requirements.txt")));
    deps
}

fn parse_cargo_toml(path: &Path) -> Vec<DependencyInfo> {
    if !path.exists() {
        return vec![];
    }
    let Ok(content) = fs::read_to_string(path) else {
        return vec![];
    };
    let Ok(value) = content.parse::<toml::Value>() else {
        return vec![];
    };

    let mut deps = Vec::new();
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = value.get(section).and_then(|v| v.as_table()) {
            for (name, spec) in table {
                let version = match spec {
                    toml::Value::String(v) => Some(v.clone()),
                    toml::Value::Table(t) => t
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    _ => None,
                };
                deps.push(DependencyInfo {
                    name: name.clone(),
                    version,
                    kind: section.to_string(),
                    source_file: "Cargo.toml".to_string(),
                });
            }
        }
    }
    deps
}

fn parse_package_json(path: &Path) -> Vec<DependencyInfo> {
    if !path.exists() {
        return vec![];
    }
    let Ok(content) = fs::read_to_string(path) else {
        return vec![];
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return vec![];
    };

    let mut deps = Vec::new();
    for section in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(obj) = value.get(section).and_then(|v| v.as_object()) {
            for (name, ver) in obj {
                deps.push(DependencyInfo {
                    name: name.clone(),
                    version: ver.as_str().map(|s| s.to_string()),
                    kind: section.to_string(),
                    source_file: "package.json".to_string(),
                });
            }
        }
    }
    deps
}

fn parse_pyproject(path: &Path) -> Vec<DependencyInfo> {
    if !path.exists() {
        return vec![];
    }
    let Ok(content) = fs::read_to_string(path) else {
        return vec![];
    };
    let Ok(value) = content.parse::<toml::Value>() else {
        return vec![];
    };

    let mut deps = Vec::new();
    if let Some(table) = value
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
    {
        let re = Regex::new(r"^([a-zA-Z0-9_-]+)").ok();
        for dep in table {
            if let Some(s) = dep.as_str() {
                let name = re
                    .as_ref()
                    .and_then(|r| r.captures(s))
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| s.to_string());
                deps.push(DependencyInfo {
                    name,
                    version: Some(s.to_string()),
                    kind: "dependencies".to_string(),
                    source_file: "pyproject.toml".to_string(),
                });
            }
        }
    }
    deps
}

fn parse_requirements(path: &Path) -> Vec<DependencyInfo> {
    if !path.exists() {
        return vec![];
    }
    let Ok(content) = fs::read_to_string(path) else {
        return vec![];
    };
    let re = Regex::new(r"^([a-zA-Z0-9_-]+)").ok();
    let mut deps = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = re
            .as_ref()
            .and_then(|r| r.captures(line))
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| line.to_string());
        deps.push(DependencyInfo {
            name,
            version: Some(line.to_string()),
            kind: "requirements".to_string(),
            source_file: "requirements.txt".to_string(),
        });
    }
    deps
}

pub fn detect_frameworks(project_root: &Path, deps: &[DependencyInfo]) -> Vec<String> {
    let mut frameworks = HashMap::new();

    let signals: &[(&str, &[&str])] = &[
        ("React", &["react", "react-dom"]),
        ("Next.js", &["next"]),
        ("Vue", &["vue"]),
        ("Angular", &["@angular/core"]),
        ("Svelte", &["svelte"]),
        ("Express", &["express"]),
        ("FastAPI", &["fastapi"]),
        ("Django", &["django"]),
        ("Flask", &["flask"]),
        ("Actix", &["actix-web"]),
        ("Axum", &["axum"]),
        ("Rocket", &["rocket"]),
        ("Tokio", &["tokio"]),
        ("Tauri", &["tauri"]),
        ("Electron", &["electron"]),
    ];

    for dep in deps {
        let name_lower = dep.name.to_lowercase();
        for (framework, keys) in signals {
            if keys.iter().any(|k| name_lower == *k) {
                frameworks.insert(framework.to_string(), true);
            }
        }
    }

    if project_root.join("next.config.js").exists()
        || project_root.join("next.config.mjs").exists()
    {
        frameworks.insert("Next.js".to_string(), true);
    }
    if project_root.join("tauri.conf.json").exists() {
        frameworks.insert("Tauri".to_string(), true);
    }

    frameworks.into_keys().collect()
}
