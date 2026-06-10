use nexus_core::{
    Architecture, Decision, EdgeRelation, KnowledgeEdge, KnowledgeGraph, KnowledgeNode,
    NexusResult, NodeKind, ScanResult, Task,
};
use nexus_memory::{db_err, MemoryEngine};
use uuid::Uuid;

pub struct KnowledgeEngine<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> KnowledgeEngine<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    pub fn build_from_scan(&self, scan: &ScanResult) -> NexusResult<KnowledgeGraph> {
        let mut graph = KnowledgeGraph::default();
        self.ingest_architecture(&mut graph, &scan.architecture)?;
        self.ingest_decisions(&mut graph)?;
        self.ingest_tasks(&mut graph)?;
        self.persist_graph(&graph)?;
        Ok(graph)
    }

    pub fn rebuild(&self) -> NexusResult<KnowledgeGraph> {
        let architecture = self.memory.load_architecture()?;
        let mut graph = KnowledgeGraph::default();
        self.ingest_architecture(&mut graph, &architecture)?;
        self.ingest_decisions(&mut graph)?;
        self.ingest_tasks(&mut graph)?;
        self.persist_graph(&graph)?;
        Ok(graph)
    }

    pub fn load_graph(&self) -> NexusResult<KnowledgeGraph> {
        self.rebuild()
    }

    pub fn persist_graph_from(&self, graph: &KnowledgeGraph) -> NexusResult<()> {
        self.persist_graph(graph)
    }

    fn ingest_architecture(
        &self,
        graph: &mut KnowledgeGraph,
        arch: &Architecture,
    ) -> NexusResult<()> {
        for layer in &arch.layers {
            let layer_id = Uuid::new_v4();
            graph.nodes.push(KnowledgeNode {
                id: layer_id,
                kind: NodeKind::Module,
                name: layer.name.clone(),
                path: None,
                metadata: serde_json::json!({ "component_count": layer.components.len() }),
            });

            for comp in &layer.components {
                let file_id = Uuid::new_v4();
                graph.nodes.push(KnowledgeNode {
                    id: file_id,
                    kind: NodeKind::File,
                    name: comp.name.clone(),
                    path: Some(comp.path.clone()),
                    metadata: serde_json::json!({ "kind": format!("{:?}", comp.kind) }),
                });
                graph.edges.push(KnowledgeEdge {
                    id: Uuid::new_v4(),
                    from: layer_id,
                    to: file_id,
                    relation: EdgeRelation::Owns,
                    weight: 1.0,
                });
            }
        }

        for dep in &arch.dependencies {
            let pkg_id = Uuid::new_v4();
            graph.nodes.push(KnowledgeNode {
                id: pkg_id,
                kind: NodeKind::Package,
                name: dep.name.clone(),
                path: Some(dep.source_file.clone()),
                metadata: serde_json::json!({ "version": dep.version, "kind": dep.kind }),
            });
        }

        Ok(())
    }

    fn ingest_decisions(&self, graph: &mut KnowledgeGraph) -> NexusResult<()> {
        let decisions = self.memory.load_decisions()?.decisions;
        for d in decisions {
            add_decision_node(graph, &d);
        }
        Ok(())
    }

    fn ingest_tasks(&self, graph: &mut KnowledgeGraph) -> NexusResult<()> {
        let tasks = self.memory.load_tasks()?.tasks;
        for t in tasks {
            add_task_node(graph, &t);
        }
        Ok(())
    }

    fn persist_graph(&self, graph: &KnowledgeGraph) -> NexusResult<()> {
        let conn = self.memory.connection();
        conn.execute("DELETE FROM knowledge_edges", [])
            .map_err(db_err)?;
        conn.execute("DELETE FROM knowledge_nodes", [])
            .map_err(db_err)?;

        for node in &graph.nodes {
            conn.execute(
                "INSERT INTO knowledge_nodes (id, kind, name, path, metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    node.id.to_string(),
                    format!("{:?}", node.kind).to_lowercase(),
                    node.name,
                    node.path,
                    node.metadata.to_string(),
                ],
            ).map_err(db_err)?;
        }

        for edge in &graph.edges {
            conn.execute(
                "INSERT INTO knowledge_edges (id, from_id, to_id, relation, weight) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    edge.id.to_string(),
                    edge.from.to_string(),
                    edge.to.to_string(),
                    format!("{:?}", edge.relation).to_lowercase(),
                    edge.weight,
                ],
            ).map_err(db_err)?;
        }

        Ok(())
    }

    pub fn find_related_files(&self, query: &str) -> NexusResult<Vec<String>> {
        let graph = self.rebuild()?;
        let q = query.to_lowercase();
        Ok(graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::File)
            .filter(|n| {
                n.name.to_lowercase().contains(&q)
                    || n.path
                        .as_ref()
                        .map(|p| p.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .filter_map(|n| n.path.clone())
            .collect())
    }
}

fn add_decision_node(graph: &mut KnowledgeGraph, d: &Decision) {
    graph.nodes.push(KnowledgeNode {
        id: d.id,
        kind: NodeKind::Decision,
        name: truncate(&d.content, 60),
        path: None,
        metadata: serde_json::json!({ "tags": d.tags, "status": format!("{:?}", d.status) }),
    });
}

fn add_task_node(graph: &mut KnowledgeGraph, t: &Task) {
    let task_id = t.id;
    graph.nodes.push(KnowledgeNode {
        id: task_id,
        kind: NodeKind::Task,
        name: t.title.clone(),
        path: None,
        metadata: serde_json::json!({
            "status": format!("{:?}", t.status),
            "priority": format!("{:?}", t.priority),
        }),
    });
    for file in &t.related_files {
        let file_id = Uuid::new_v4();
        graph.nodes.push(KnowledgeNode {
            id: file_id,
            kind: NodeKind::File,
            name: file.rsplit('/').next().unwrap_or(file).to_string(),
            path: Some(file.clone()),
            metadata: serde_json::json!({}),
        });
        graph.edges.push(KnowledgeEdge {
            id: Uuid::new_v4(),
            from: task_id,
            to: file_id,
            relation: EdgeRelation::RelatedTo,
            weight: 0.8,
        });
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
