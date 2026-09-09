use crate::autostart;
use crate::db::{fetch_labels_for_issue, DbState};
use crate::dependencies::{fetch_all_dependencies, parse_repo_slug, poll_dependencies};
use crate::models::{Dependency, Issue, IssueEvent, IssueRevision, Label, Project};
use chrono::Utc;
use rusqlite::params;
use tauri::{Manager, State};

fn parse_optional_id(val: Option<serde_json::Value>) -> Option<i64> {
    match val {
        Some(serde_json::Value::Number(n)) => n.as_i64(),
        Some(serde_json::Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed == "null" {
                None
            } else {
                trimmed.parse::<i64>().ok()
            }
        }
        _ => None,
    }
}

fn parse_id_vec(vals: Vec<serde_json::Value>) -> Vec<i64> {
    vals.into_iter()
        .filter_map(|v| match v {
            serde_json::Value::Number(n) => n.as_i64(),
            serde_json::Value::String(s) => s.trim().parse::<i64>().ok(),
            _ => None,
        })
        .collect()
}

fn body_references_issue(body: &str, target_id: i64) -> bool {
    let needle = format!("#{}", target_id);
    let bytes = body.as_bytes();
    let needle_bytes = needle.as_bytes();
    let mut pos = 0;
    while let Some(idx) = body[pos..].find(&needle) {
        let abs_idx = pos + idx;
        if (abs_idx == 0 || !bytes[abs_idx - 1].is_ascii_alphanumeric())
            && (abs_idx + needle_bytes.len() >= bytes.len() || !bytes[abs_idx + needle_bytes.len()].is_ascii_digit())
        {
            return true;
        }
        pos = abs_idx + needle_bytes.len();
    }
    false
}

fn log_event(
    conn: &rusqlite::Connection,
    issue_id: i64,
    event_type: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    metadata: Option<&str>,
) {
    let now = Utc::now();
    let threshold = (now - chrono::Duration::seconds(60)).to_rfc3339();

    if event_type == "label_added" || event_type == "label_removed" {
        let inverse = if event_type == "label_added" { "label_removed" } else { "label_added" };
        if let Ok(prev_id) = conn.query_row(
            "SELECT id FROM issue_events WHERE issue_id = ?1 AND event_type = ?2 AND new_value = ?3 AND created_at >= ?4 ORDER BY id DESC LIMIT 1",
            params![issue_id, inverse, new_value.unwrap_or(""), threshold],
            |r| r.get::<_, i64>(0),
        ) {
            let _ = conn.execute("DELETE FROM issue_events WHERE id = ?1", params![prev_id]);
            return;
        }

        if conn.query_row(
            "SELECT 1 FROM issue_events WHERE issue_id = ?1 AND event_type = ?2 AND new_value = ?3 AND created_at >= ?4 ORDER BY id DESC LIMIT 1",
            params![issue_id, event_type, new_value.unwrap_or(""), threshold],
            |_| Ok(true),
        ).unwrap_or(false) {
            return;
        }
    } else if matches!(event_type, "status_change" | "project_change" | "title_change") {
        if let Ok((prev_id, prev_old)) = conn.query_row(
            "SELECT id, old_value FROM issue_events WHERE issue_id = ?1 AND event_type = ?2 AND created_at >= ?3 ORDER BY id DESC LIMIT 1",
            params![issue_id, event_type, threshold],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?)),
        ) {
            if prev_old.as_deref() == new_value {
                let _ = conn.execute("DELETE FROM issue_events WHERE id = ?1", params![prev_id]);
            } else {
                let _ = conn.execute(
                    "UPDATE issue_events SET new_value = ?1, created_at = ?2 WHERE id = ?3",
                    params![new_value, now.to_rfc3339(), prev_id],
                );
            }
            return;
        }
    }

    let _ = conn.execute(
        "INSERT INTO issue_events (issue_id, event_type, old_value, new_value, metadata, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![issue_id, event_type, old_value, new_value, metadata, now.to_rfc3339()],
    );
}

pub fn get_attachments_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .or_else(|_| app.path().app_local_data_dir())
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("attachments");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn cleanup_orphaned_attachments(conn: &rusqlite::Connection, app: &tauri::AppHandle) -> Result<usize, String> {
    let dir = get_attachments_dir(app)?;
    if !dir.exists() {
        return Ok(0);
    }

    let bodies: Vec<String> = conn
        .prepare("SELECT body FROM issues UNION ALL SELECT body FROM issue_revisions")
        .map_err(|e| e.to_string())?
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    let all_text = bodies.join("\n");
    let now = std::time::SystemTime::now();
    let mut removed = 0;

    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if !all_text.contains(file_name) {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if now.duration_since(modified).map(|d| d.as_secs() > 1800).unwrap_or(false)
                                && std::fs::remove_file(&path).is_ok()
                            {
                                removed += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(removed)
}

#[tauri::command]
pub fn save_attachment(name: String, data: Vec<u8>, app: tauri::AppHandle) -> Result<String, String> {
    let clean = std::path::Path::new(&name)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;
    std::fs::write(get_attachments_dir(&app)?.join(clean), data).map_err(|e| e.to_string())?;
    Ok(clean.to_string())
}

#[tauri::command]
pub fn get_attachment(name: String, app: tauri::AppHandle) -> Result<Vec<u8>, String> {
    let clean = std::path::Path::new(&name)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;
    std::fs::read(get_attachments_dir(&app)?.join(clean)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_issues(state: State<DbState>) -> Result<Vec<Issue>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let raw_issues: Vec<(i64, String, String, String, Option<i64>, Option<String>, String)> = {
        let mut stmt = conn
            .prepare(
                "SELECT i.id, i.title, i.body, i.status, i.project_id, p.title, i.created_at 
                 FROM issues i LEFT JOIN projects p ON i.project_id = p.id ORDER BY i.id DESC",
            )
            .map_err(|e| e.to_string())?;

        let mapped = stmt
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        mapped
    };

    raw_issues
        .into_iter()
        .map(|(id, title, body, status, project_id, project_title, created_at)| {
            Ok(Issue {
                id,
                title,
                body,
                status,
                project_id,
                project_title,
                labels: fetch_labels_for_issue(&conn, id).unwrap_or_default(),
                created_at,
            })
        })
        .collect()
}

#[tauri::command]
pub fn create_issue(
    title: String,
    body: String,
    project_id: Option<serde_json::Value>,
    label_ids: Option<Vec<serde_json::Value>>,
    state: State<DbState>,
) -> Result<Issue, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let created_at = Utc::now().to_rfc3339();
    let pid = parse_optional_id(project_id);

    conn.execute(
        "INSERT INTO issues (title, body, status, project_id, created_at) VALUES (?1, ?2, 'open', ?3, ?4)",
        params![title, body, pid, created_at],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    for lid in parse_id_vec(label_ids.unwrap_or_default()) {
        let _ = conn.execute("INSERT OR IGNORE INTO issue_labels (issue_id, label_id) VALUES (?1, ?2)", params![id, lid]);
    }

    let _ = conn.execute("INSERT INTO issue_revisions (issue_id, body, created_at) VALUES (?1, ?2, ?3)", params![id, body, created_at]);

    Ok(Issue {
        id,
        title,
        body,
        status: "open".into(),
        project_id: pid,
        project_title: pid.and_then(|p| conn.query_row("SELECT title FROM projects WHERE id = ?1", params![p], |r| r.get(0)).ok()),
        labels: fetch_labels_for_issue(&conn, id).unwrap_or_default(),
        created_at,
    })
}

#[tauri::command]
pub fn update_issue(id: i64, title: String, body: String, state: State<DbState>) -> Result<Issue, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    if let Ok((old_t, old_b)) = conn.query_row(
        "SELECT title, body FROM issues WHERE id = ?1",
        params![id],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    ) {
        if old_t != title {
            log_event(&conn, id, "title_change", Some(&old_t), Some(&title), None);
        }
        if old_b != body {
            let now = Utc::now().to_rfc3339();
            if conn.query_row("SELECT COUNT(*) FROM issue_revisions WHERE issue_id = ?1", params![id], |r| r.get::<_, i64>(0)).unwrap_or(0) == 0 {
                let _ = conn.execute("INSERT INTO issue_revisions (issue_id, body, created_at) VALUES (?1, ?2, ?3)", params![id, old_b, now]);
            }
            let _ = conn.execute("INSERT INTO issue_revisions (issue_id, body, created_at) VALUES (?1, ?2, ?3)", params![id, body, now]);
        }
    }

    conn.execute("UPDATE issues SET title = ?1, body = ?2 WHERE id = ?3", params![title, body, id])
        .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT i.id, i.title, i.body, i.status, i.project_id, p.title, i.created_at 
         FROM issues i LEFT JOIN projects p ON i.project_id = p.id WHERE i.id = ?1",
        params![id],
        |r| {
            Ok(Issue {
                id: r.get(0)?,
                title: r.get(1)?,
                body: r.get(2)?,
                status: r.get(3)?,
                project_id: r.get(4)?,
                project_title: r.get(5)?,
                labels: fetch_labels_for_issue(&conn, id).unwrap_or_default(),
                created_at: r.get(6)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_issue_status(id: i64, state: State<DbState>) -> Result<Issue, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let cur: String = conn.query_row("SELECT status FROM issues WHERE id = ?1", params![id], |r| r.get(0)).map_err(|e| e.to_string())?;
    let next = if cur == "open" { "closed" } else { "open" };

    log_event(&conn, id, "status_change", Some(&cur), Some(next), None);
    conn.execute("UPDATE issues SET status = ?1 WHERE id = ?2", params![next, id]).map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT i.id, i.title, i.body, i.status, i.project_id, p.title, i.created_at 
         FROM issues i LEFT JOIN projects p ON i.project_id = p.id WHERE i.id = ?1",
        params![id],
        |r| {
            Ok(Issue {
                id: r.get(0)?,
                title: r.get(1)?,
                body: r.get(2)?,
                status: r.get(3)?,
                project_id: r.get(4)?,
                project_title: r.get(5)?,
                labels: fetch_labels_for_issue(&conn, id).unwrap_or_default(),
                created_at: r.get(6)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_issue_meta(
    id: i64,
    project_id: Option<serde_json::Value>,
    label_ids: Option<Vec<serde_json::Value>>,
    state: State<DbState>,
) -> Result<Issue, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let pid = parse_optional_id(project_id);
    let lids = parse_id_vec(label_ids.unwrap_or_default());

    let old_pid: Option<i64> = conn.query_row("SELECT project_id FROM issues WHERE id = ?1", params![id], |r| r.get(0)).unwrap_or(None);
    if old_pid != pid {
        let old_title = old_pid.and_then(|p| conn.query_row("SELECT title FROM projects WHERE id = ?1", params![p], |r| r.get::<_, String>(0)).ok());
        let new_title = pid.and_then(|p| conn.query_row("SELECT title FROM projects WHERE id = ?1", params![p], |r| r.get::<_, String>(0)).ok());
        log_event(&conn, id, "project_change", old_title.as_deref(), new_title.as_deref(), None);
    }

    let old_lids: Vec<i64> = conn
        .prepare("SELECT label_id FROM issue_labels WHERE issue_id = ?1")
        .map_err(|e| e.to_string())?
        .query_map(params![id], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    for added_id in lids.iter().filter(|x| !old_lids.contains(x)) {
        if let Ok((name, color)) = conn.query_row::<(String, String), _, _>("SELECT name, color FROM labels WHERE id = ?1", params![added_id], |r| Ok((r.get(0)?, r.get(1)?))) {
            log_event(&conn, id, "label_added", None, Some(&name), Some(&color));
        }
    }

    for removed_id in old_lids.iter().filter(|x| !lids.contains(x)) {
        if let Ok((name, color)) = conn.query_row::<(String, String), _, _>("SELECT name, color FROM labels WHERE id = ?1", params![removed_id], |r| Ok((r.get(0)?, r.get(1)?))) {
            log_event(&conn, id, "label_removed", None, Some(&name), Some(&color));
        }
    }

    conn.execute("UPDATE issues SET project_id = ?1 WHERE id = ?2", params![pid, id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM issue_labels WHERE issue_id = ?1", params![id]).map_err(|e| e.to_string())?;

    for lid in lids {
        let _ = conn.execute("INSERT OR IGNORE INTO issue_labels (issue_id, label_id) VALUES (?1, ?2)", params![id, lid]);
    }

    conn.query_row(
        "SELECT i.id, i.title, i.body, i.status, i.project_id, p.title, i.created_at 
         FROM issues i LEFT JOIN projects p ON i.project_id = p.id WHERE i.id = ?1",
        params![id],
        |r| {
            Ok(Issue {
                id: r.get(0)?,
                title: r.get(1)?,
                body: r.get(2)?,
                status: r.get(3)?,
                project_id: r.get(4)?,
                project_title: r.get(5)?,
                labels: fetch_labels_for_issue(&conn, id).unwrap_or_default(),
                created_at: r.get(6)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_issue_events(issue_id: i64, state: State<DbState>) -> Result<Vec<IssueEvent>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut events: Vec<IssueEvent> = conn
        .prepare("SELECT id, issue_id, event_type, old_value, new_value, metadata, created_at FROM issue_events WHERE issue_id = ?1 ORDER BY id ASC")
        .map_err(|e| e.to_string())?
        .query_map(params![issue_id], |r| {
            Ok(IssueEvent {
                id: r.get(0)?,
                issue_id: r.get(1)?,
                event_type: r.get(2)?,
                old_value: r.get(3)?,
                new_value: r.get(4)?,
                metadata: r.get(5)?,
                created_at: r.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    let other_issues: Vec<(i64, String, String, String)> = conn
        .prepare("SELECT id, title, body, created_at FROM issues WHERE id != ?1")
        .map_err(|e| e.to_string())?
        .query_map(params![issue_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    let mut synth = -100000;
    for (src_id, src_title, src_body, src_created) in other_issues {
        if body_references_issue(&src_body, issue_id) {
            events.push(IssueEvent {
                id: synth,
                issue_id,
                event_type: "cross_reference".into(),
                old_value: Some(src_id.to_string()),
                new_value: Some(src_title),
                metadata: None,
                created_at: src_created,
            });
            synth -= 1;
        }
    }

    events.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(events)
}

#[tauri::command]
pub fn get_issue_revisions(issue_id: i64, state: State<DbState>) -> Result<Vec<IssueRevision>, String> {
    conn_query_all(&state, "SELECT id, issue_id, body, created_at FROM issue_revisions WHERE issue_id = ?1 ORDER BY id ASC", params![issue_id], |r| {
        Ok(IssueRevision {
            id: r.get(0)?,
            issue_id: r.get(1)?,
            body: r.get(2)?,
            created_at: r.get(3)?,
        })
    })
}

#[tauri::command]
pub fn get_labels(state: State<DbState>) -> Result<Vec<Label>, String> {
    conn_query_all(&state, "SELECT id, name, color, description FROM labels ORDER BY name ASC", [], |r| {
        Ok(Label {
            id: r.get(0)?,
            name: r.get(1)?,
            color: r.get(2)?,
            description: r.get(3)?,
        })
    })
}

#[tauri::command]
pub fn create_label(name: String, color: String, description: String, state: State<DbState>) -> Result<Label, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO labels (name, color, description) VALUES (?1, ?2, ?3)", params![name, color, description])
        .map_err(|e| e.to_string())?;
    Ok(Label { id: conn.last_insert_rowid(), name, color, description })
}

#[tauri::command]
pub fn update_label(id: i64, name: String, color: String, description: String, state: State<DbState>) -> Result<Label, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE labels SET name = ?1, color = ?2, description = ?3 WHERE id = ?4", params![name, color, description, id])
        .map_err(|e| e.to_string())?;
    Ok(Label { id, name, color, description })
}

#[tauri::command]
pub fn delete_label(id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let _ = conn.execute("DELETE FROM issue_labels WHERE label_id = ?1", params![id]);
    conn.execute("DELETE FROM labels WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_projects(state: State<DbState>) -> Result<Vec<Project>, String> {
    conn_query_all(&state, "SELECT id, title, description FROM projects ORDER BY id DESC", [], |r| {
        Ok(Project {
            id: r.get(0)?,
            title: r.get(1)?,
            description: r.get(2)?,
        })
    })
}

#[tauri::command]
pub fn create_project(title: String, description: String, state: State<DbState>) -> Result<Project, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO projects (title, description) VALUES (?1, ?2)", params![title, description])
        .map_err(|e| e.to_string())?;
    Ok(Project { id: conn.last_insert_rowid(), title, description })
}

#[tauri::command]
pub fn update_project(id: i64, title: String, description: String, state: State<DbState>) -> Result<Project, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE projects SET title = ?1, description = ?2 WHERE id = ?3", params![title, description, id])
        .map_err(|e| e.to_string())?;
    Ok(Project { id, title, description })
}

#[tauri::command]
pub fn delete_project(id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let _ = conn.execute("UPDATE issues SET project_id = NULL WHERE project_id = ?1", params![id]);
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_dependencies(state: State<DbState>) -> Result<Vec<Dependency>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    fetch_all_dependencies(&conn)
}

#[tauri::command]
pub fn check_dependencies(app: tauri::AppHandle, state: State<DbState>) -> Result<Vec<Dependency>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    poll_dependencies(&conn, &app, true)
}

#[tauri::command]
pub fn acknowledge_dependency(id: i64, state: State<DbState>) -> Result<Dependency, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE dependencies SET last_seen_tag = latest_tag, has_update = 0 WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, name, repo_owner, repo_name, last_seen_tag, latest_tag, release_name, release_url, published_at, has_update, last_checked_at, created_at 
         FROM dependencies WHERE id = ?1",
        params![id],
        |r| {
            Ok(Dependency {
                id: r.get(0)?,
                name: r.get(1)?,
                repo_owner: r.get(2)?,
                repo_name: r.get(3)?,
                last_seen_tag: r.get(4)?,
                latest_tag: r.get(5)?,
                release_name: r.get(6)?,
                release_url: r.get(7)?,
                published_at: r.get(8)?,
                has_update: r.get::<_, i64>(9)? != 0,
                last_checked_at: r.get(10)?,
                created_at: r.get(11)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_dependency(name: String, target: String, app: tauri::AppHandle, state: State<DbState>) -> Result<Dependency, String> {
    let (owner, repo) = parse_repo_slug(&target).ok_or_else(|| "Invalid GitHub repository format".to_string())?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO dependencies (name, repo_owner, repo_name, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![name.trim(), owner, repo, Utc::now().to_rfc3339()],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let _ = poll_dependencies(&conn, &app, true);
    conn.query_row(
        "SELECT id, name, repo_owner, repo_name, last_seen_tag, latest_tag, release_name, release_url, published_at, has_update, last_checked_at, created_at 
         FROM dependencies WHERE id = ?1",
        params![id],
        |r| {
            Ok(Dependency {
                id: r.get(0)?,
                name: r.get(1)?,
                repo_owner: r.get(2)?,
                repo_name: r.get(3)?,
                last_seen_tag: r.get(4)?,
                latest_tag: r.get(5)?,
                release_name: r.get(6)?,
                release_url: r.get(7)?,
                published_at: r.get(8)?,
                has_update: r.get::<_, i64>(9)? != 0,
                last_checked_at: r.get(10)?,
                created_at: r.get(11)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_dependency(id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM dependencies WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_autostart() -> Result<bool, String> {
    Ok(autostart::is_autostart_enabled())
}

#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    autostart::set_autostart_enabled(enabled)
}

fn conn_query_all<T, P, F>(state: &State<DbState>, sql: &str, params: P, map: F) -> Result<Vec<T>, String>
where
    P: rusqlite::Params,
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
{
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params, map)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}