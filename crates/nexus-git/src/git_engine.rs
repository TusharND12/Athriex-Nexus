use std::path::Path;

use chrono::{TimeZone, Utc};
use git2::{Repository, Status, StatusOptions};
use nexus_core::{CommitSummary, GitSummary, NexusResult};

pub struct GitEngine {
    repo: Option<Repository>,
    project_root: std::path::PathBuf,
}

impl GitEngine {
    pub fn open(project_root: impl AsRef<Path>) -> NexusResult<Self> {
        let project_root = project_root.as_ref().to_path_buf();
        let repo = Repository::discover(&project_root)
            .map_err(|e| nexus_core::NexusError::Git(e.message().to_string()))
            .ok();
        Ok(Self { repo, project_root })
    }

    pub fn is_git_repo(&self) -> bool {
        self.repo.is_some()
    }

    pub fn summarize(&self, commit_limit: usize) -> NexusResult<GitSummary> {
        let Some(repo) = &self.repo else {
            return Ok(GitSummary::default());
        };

        let head = repo.head().ok();
        let branch = head
            .as_ref()
            .and_then(|h| h.shorthand())
            .unwrap_or("unknown")
            .to_string();

        let mut revwalk = repo.revwalk().map_err(git_err)?;
        if let Some(head) = &head {
            revwalk.push(head.target().unwrap()).map_err(git_err)?;
        }
        revwalk.set_sorting(git2::Sort::TIME).map_err(git_err)?;

        let mut recent_commits = Vec::new();
        let mut commit_count = 0usize;

        for oid in revwalk {
            let oid = oid.map_err(git_err)?;
            commit_count += 1;
            if recent_commits.len() < commit_limit {
                if let Ok(commit) = repo.find_commit(oid) {
                    let author = commit.author();
                    recent_commits.push(CommitSummary {
                        hash: oid.to_string()[..8].to_string(),
                        message: commit.summary().unwrap_or("no message").to_string(),
                        author: author.name().unwrap_or("unknown").to_string(),
                        timestamp: Utc
                            .timestamp_opt(commit.time().seconds(), 0)
                            .single()
                            .unwrap_or_else(Utc::now),
                    });
                }
            }
        }

        let dirty_files = self.dirty_files()?;

        Ok(GitSummary {
            branch,
            commit_count,
            recent_commits,
            dirty_files,
        })
    }

    pub fn dirty_files(&self) -> NexusResult<Vec<String>> {
        let Some(repo) = &self.repo else {
            return Ok(vec![]);
        };

        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .exclude_submodules(true);

        let statuses = repo.statuses(Some(&mut opts)).map_err(git_err)?;
        let mut files = Vec::new();

        for entry in statuses.iter() {
            if entry.status() != Status::CURRENT {
                if let Some(path) = entry.path() {
                    files.push(path.to_string());
                }
            }
        }

        Ok(files)
    }

    pub fn recent_commit_messages(&self, limit: usize) -> NexusResult<Vec<String>> {
        let summary = self.summarize(limit)?;
        Ok(summary
            .recent_commits
            .into_iter()
            .map(|c| format!("[{}] {} — {}", c.hash, c.message, c.author))
            .collect())
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}

fn git_err(e: git2::Error) -> nexus_core::NexusError {
    nexus_core::NexusError::Git(e.message().to_string())
}
