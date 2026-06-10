use nexus_core::NexusResult;
use nexus_knowledge::KnowledgeEngine;
use nexus_memory::{db_err, MemoryEngine};

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub doc_id: String,
    pub source: String,
    pub snippet: String,
    pub rank: f64,
}

pub struct SearchEngine<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> SearchEngine<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    pub fn ask(&self, query: &str, limit: usize) -> NexusResult<Vec<SearchHit>> {
        let mut hits = self.fts_search(query, limit)?;

        let knowledge = KnowledgeEngine::new(self.memory);
        let related_files = knowledge.find_related_files(query)?;
        for file in related_files {
            hits.push(SearchHit {
                doc_id: file.clone(),
                source: "file".to_string(),
                snippet: format!("Related file: {file}"),
                rank: 0.5,
            });
        }

        let decisions = self.memory.load_decisions()?;
        let q = query.to_lowercase();
        for d in decisions.decisions {
            if d.content.to_lowercase().contains(&q)
                || d.rationale
                    .as_ref()
                    .map(|r| r.to_lowercase().contains(&q))
                    .unwrap_or(false)
            {
                hits.push(SearchHit {
                    doc_id: d.id.to_string(),
                    source: "decision".to_string(),
                    snippet: d.content,
                    rank: 0.9,
                });
            }
        }

        hits.sort_by(|a, b| {
            b.rank
                .partial_cmp(&a.rank)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit);
        Ok(hits)
    }

    pub fn format_answer(&self, query: &str, limit: usize) -> NexusResult<String> {
        let hits = self.ask(query, limit)?;
        if hits.is_empty() {
            return Ok(format!("No results found for: \"{query}\""));
        }

        let mut output = format!("ANSWER: \"{query}\"\n\n");
        for (i, hit) in hits.iter().enumerate() {
            output.push_str(&format!(
                "{}. [{}] {}\n   {}\n\n",
                i + 1,
                hit.source,
                hit.doc_id,
                hit.snippet
            ));
        }
        Ok(output)
    }

    fn fts_search(&self, query: &str, limit: usize) -> NexusResult<Vec<SearchHit>> {
        let sanitized = query
            .split_whitespace()
            .map(|w| format!("\"{w}\""))
            .collect::<Vec<_>>()
            .join(" OR ");

        let mut stmt = self
            .memory
            .connection()
            .prepare(
                "SELECT doc_id, source, snippet(memory_fts, 2, '>>', '<<', '…', 20) as snip, rank
             FROM memory_fts WHERE memory_fts MATCH ?1 ORDER BY rank LIMIT ?2",
            )
            .map_err(db_err)?;

        let rows = stmt
            .query_map(rusqlite::params![sanitized, limit as i64], |row| {
                Ok(SearchHit {
                    doc_id: row.get(0)?,
                    source: row.get(1)?,
                    snippet: row.get(2)?,
                    rank: row.get(3)?,
                })
            })
            .map_err(db_err)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(db_err)
    }
}
