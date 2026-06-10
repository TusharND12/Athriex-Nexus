use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use nexus_core::{
    Architecture, ImportantFile, LanguageStats, NexusResult, ScanResult, Technology,
    TimelineEvent, TimelineEventKind,
};
use nexus_memory::MemoryEngine;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language as TsLanguage, Parser, Query, QueryCursor};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::dependency_detect::{detect_dependencies, detect_frameworks};
use crate::language_detect::{detect_language, is_source_file, should_skip_dir};
use crate::structure::infer_layers;

pub struct ScannerEngine<'a> {
    memory: &'a MemoryEngine,
    project_root: PathBuf,
}

impl<'a> ScannerEngine<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        let project_root = memory.paths.project_root.clone();
        Self {
            memory,
            project_root,
        }
    }

    pub fn scan(&self) -> NexusResult<ScanResult> {
        let mut file_paths = Vec::new();
        let mut lang_stats: HashMap<String, LanguageStats> = HashMap::new();
        let mut important_files = Vec::new();
        let mut symbol_count = 0usize;

        for entry in WalkDir::new(&self.project_root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    !should_skip_dir(e.file_name().to_str().unwrap_or(""))
                } else {
                    true
                }
            })
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path().to_path_buf();
            if !is_source_file(&path) {
                continue;
            }

            let rel = path
                .strip_prefix(&self.project_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");

            file_paths.push(path.clone());

            if let Some(lang) = detect_language(&path) {
                let lines = fs::read_to_string(&path)
                    .map(|c| c.lines().count())
                    .unwrap_or(0);
                let stats = lang_stats.entry(lang.to_string()).or_insert(LanguageStats {
                    language: lang.to_string(),
                    file_count: 0,
                    line_count: 0,
                });
                stats.file_count += 1;
                stats.line_count += lines;

                let count = count_symbols(&path, lang).unwrap_or(0);
                symbol_count += count;
                if count > 5 {
                        important_files.push(ImportantFile {
                            path: rel.clone(),
                            reason: format!("Contains {count} top-level symbols"),
                            relevance: (count as f32 / 20.0).min(1.0),
                        });
                }
            }

            if is_entry_point(&rel) {
                important_files.push(ImportantFile {
                    path: rel,
                    reason: "Project entry point".to_string(),
                    relevance: 1.0,
                });
            }
        }

        let dependencies = detect_dependencies(&self.project_root);
        let frameworks = detect_frameworks(&self.project_root, &dependencies);
        let layers = infer_layers(&self.project_root, &file_paths);

        let technologies = build_technologies(&lang_stats, &frameworks, &dependencies);

        let mut architecture = Architecture {
            version: nexus_core::NEXUS_VERSION.to_string(),
            updated_at: Utc::now(),
            layers,
            technologies,
            dependencies: dependencies.clone(),
            entry_points: detect_entry_points(&self.project_root),
            important_files: {
                important_files.sort_by(|a, b| {
                    b.relevance
                        .partial_cmp(&a.relevance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                important_files.truncate(30);
                important_files
            },
        };

        let result = ScanResult {
            scanned_at: Utc::now(),
            files_analyzed: file_paths.len(),
            languages: lang_stats.into_values().collect(),
            frameworks,
            architecture: architecture.clone(),
        };

        self.memory.save_architecture(&architecture)?;

        let mut memory = self.memory.load_memory()?;
        memory.technologies = result
            .languages
            .iter()
            .map(|l| l.language.clone())
            .chain(result.frameworks.iter().cloned())
            .collect();
        memory.updated_at = Utc::now();
        self.memory.save_memory(&memory)?;

        let result_json = serde_json::to_string(&result)?;
        self.memory
            .record_scan(result.files_analyzed, &result_json)?;

        self.memory.append_timeline_event(TimelineEvent {
            id: Uuid::new_v4(),
            kind: TimelineEventKind::Scan,
            title: format!("Scanned {} files", result.files_analyzed),
            description: Some(format!(
                "{} languages, {} symbols indexed",
                result.languages.len(),
                symbol_count
            )),
            timestamp: Utc::now(),
            metadata: serde_json::json!({
                "files": result.files_analyzed,
                "frameworks": result.frameworks,
            }),
        })?;

        Ok(result)
    }
}

fn build_technologies(
    lang_stats: &HashMap<String, LanguageStats>,
    frameworks: &[String],
    deps: &[nexus_core::DependencyInfo],
) -> Vec<Technology> {
    let mut tech = Vec::new();
    for stats in lang_stats.values() {
        tech.push(Technology {
            name: stats.language.clone(),
            category: "language".to_string(),
            version: None,
            confidence: 1.0,
        });
    }
    for fw in frameworks {
        tech.push(Technology {
            name: fw.clone(),
            category: "framework".to_string(),
            version: None,
            confidence: 0.9,
        });
    }
    for dep in deps.iter().take(20) {
        tech.push(Technology {
            name: dep.name.clone(),
            category: "dependency".to_string(),
            version: dep.version.clone(),
            confidence: 0.7,
        });
    }
    tech
}

fn detect_entry_points(project_root: &std::path::Path) -> Vec<String> {
    let candidates = [
        "src/main.rs",
        "src/lib.rs",
        "main.rs",
        "index.js",
        "index.ts",
        "src/index.ts",
        "src/index.js",
        "app/main.py",
        "main.py",
        "src/app.tsx",
        "src/App.tsx",
    ];
    candidates
        .iter()
        .filter(|p| project_root.join(p).exists())
        .map(|s| s.to_string())
        .collect()
}

fn is_entry_point(rel: &str) -> bool {
    matches!(
        rel,
        "src/main.rs"
            | "src/lib.rs"
            | "main.rs"
            | "index.js"
            | "index.ts"
            | "src/index.ts"
            | "main.py"
            | "src/app.tsx"
            | "src/App.tsx"
    )
}

fn count_symbols(path: &PathBuf, lang: &str) -> NexusResult<usize> {
    let Some((ts_lang, query_src)) = language_query(lang) else {
        return Ok(0);
    };

    let content = fs::read_to_string(path)?;
    let mut parser = Parser::new();
    parser
        .set_language(&ts_lang)
        .map_err(|e| nexus_core::NexusError::Scan(e.to_string()))?;

    let tree = parser
        .parse(&content, None)
        .ok_or_else(|| nexus_core::NexusError::Scan("parse failed".into()))?;

    let query = Query::new(&ts_lang, query_src)
        .map_err(|e| nexus_core::NexusError::Scan(e.to_string()))?;
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
    Ok(matches.count())
}

fn language_query(lang: &str) -> Option<(TsLanguage, &'static str)> {
    match lang {
        "rust" => Some((
            tree_sitter_rust::LANGUAGE.into(),
            "(function_item name: (identifier) @name) (struct_item name: (type_identifier) @name) (enum_item name: (type_identifier) @name) (impl_item) (trait_item name: (type_identifier) @name)",
        )),
        "javascript" => Some((
            tree_sitter_javascript::LANGUAGE.into(),
            "(function_declaration name: (identifier) @name) (class_declaration name: (identifier) @name) (method_definition name: (property_identifier) @name)",
        )),
        "typescript" | "tsx" => Some((
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "(function_declaration name: (identifier) @name) (class_declaration name: (identifier) @name) (interface_declaration name: (type_identifier) @name)",
        )),
        "python" => Some((
            tree_sitter_python::LANGUAGE.into(),
            "(function_definition name: (identifier) @name) (class_definition name: (identifier) @name)",
        )),
        _ => None,
    }
}
