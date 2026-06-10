use std::collections::HashMap;
use std::path::{Path, PathBuf};

use nexus_core::{ArchitectureComponent, ArchitectureLayer, ComponentKind};

pub fn infer_layers(project_root: &Path, file_paths: &[PathBuf]) -> Vec<ArchitectureLayer> {
    let mut buckets: HashMap<ComponentKind, Vec<ArchitectureComponent>> = HashMap::new();

    for path in file_paths {
        let rel = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        let kind = classify_path(&rel);
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        buckets
            .entry(kind.clone())
            .or_default()
            .push(ArchitectureComponent {
                name,
                path: rel,
                kind: kind.clone(),
                description: String::new(),
            });
    }

    let layer_order = [
        (ComponentKind::Frontend, "Frontend"),
        (ComponentKind::Backend, "Backend"),
        (ComponentKind::Api, "API"),
        (ComponentKind::Database, "Database"),
        (ComponentKind::Shared, "Shared"),
        (ComponentKind::Infrastructure, "Infrastructure"),
        (ComponentKind::Test, "Tests"),
        (ComponentKind::Config, "Configuration"),
        (ComponentKind::Other, "Other"),
    ];

    layer_order
        .iter()
        .filter_map(|(kind, name)| {
            let components = buckets.remove(kind)?;
            if components.is_empty() {
                return None;
            }
            Some(ArchitectureLayer {
                name: name.to_string(),
                description: format!("{} components detected by structure analysis", name),
                components: truncate_components(components, 50),
            })
        })
        .collect()
}

fn classify_path(rel: &str) -> ComponentKind {
    let lower = rel.to_lowercase();
    if lower.contains("/test") || lower.contains("/tests/") || lower.starts_with("test/") {
        return ComponentKind::Test;
    }
    if lower.contains("frontend")
        || lower.contains("/ui/")
        || lower.contains("/components/")
        || lower.contains("/pages/")
        || lower.contains("/app/")
        && !lower.contains("/api/")
    {
        return ComponentKind::Frontend;
    }
    if lower.contains("/api/") || lower.contains("routes/") || lower.contains("handlers/") {
        return ComponentKind::Api;
    }
    if lower.contains("database")
        || lower.contains("/db/")
        || lower.contains("migration")
        || lower.contains("schema")
    {
        return ComponentKind::Database;
    }
    if lower.contains("shared")
        || lower.contains("/lib/")
        || lower.contains("/common/")
        || lower.contains("/utils/")
    {
        return ComponentKind::Shared;
    }
    if lower.contains("docker")
        || lower.contains(".github")
        || lower.contains("infra")
        || lower.contains("deploy")
    {
        return ComponentKind::Infrastructure;
    }
    if lower.ends_with(".toml")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
        || lower.contains("config")
    {
        return ComponentKind::Config;
    }
    if lower.contains("src/") || lower.contains("crates/") || lower.contains("server/") {
        return ComponentKind::Backend;
    }
    ComponentKind::Other
}

fn truncate_components(mut components: Vec<ArchitectureComponent>, max: usize) -> Vec<ArchitectureComponent> {
    if components.len() > max {
        components.truncate(max);
    }
    components
}

pub fn format_architecture_tree(layers: &[ArchitectureLayer]) -> String {
    let mut output = String::new();
    for layer in layers {
        output.push_str(&format!("{}\n", layer.name));
        for comp in &layer.components {
            let leaf = comp.path.rsplit('/').next().unwrap_or(&comp.path);
            output.push_str(&format!("├── {leaf}\n"));
        }
        output.push('\n');
    }
    output
}
