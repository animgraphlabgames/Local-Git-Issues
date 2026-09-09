use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Label {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: i64,
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Issue {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub status: String,
    pub project_id: Option<i64>,
    pub project_title: Option<String>,
    pub labels: Vec<Label>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IssueEvent {
    pub id: i64,
    pub issue_id: i64,
    pub event_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IssueRevision {
    pub id: i64,
    pub issue_id: i64,
    pub body: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub id: i64,
    pub name: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub last_seen_tag: Option<String>,
    pub latest_tag: Option<String>,
    pub release_name: Option<String>,
    pub release_url: Option<String>,
    pub published_at: Option<String>,
    pub has_update: bool,
    pub last_checked_at: Option<String>,
    pub created_at: String,
}