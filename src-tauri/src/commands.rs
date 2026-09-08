use crate::autostart;
use crate::db::{fetch_labels_for_issue, DbState};
use crate::models::{Issue, IssueEvent, IssueRevision, Label, Project};
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
        let inverse = if event_type == "label_added" {
            "label_removed"
        } else {
            "label_added"
        };

        let prev_inverse_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM issue_events 
                 WHERE issue_id = ?1 AND event_type = ?2 AND new_value = ?3 AND created_at >= ?4 
                 ORDER BY id DESC LIMIT 1",
                params![issue_id, inverse, new_value.unwrap_or(""), threshold],
                |r| r.get(0),
            )
            .ok();

        if let Some(id) = prev_inverse_id {
            let _ = conn.execute("DELETE FROM issue_events WHERE id = ?1", params![id]);
            return;
        }

        let duplicate: bool = conn
            .query_row(
                "SELECT 1 FROM issue_events 
                 WHERE issue_id = ?1 AND event_type = ?2 AND new_value = ?3 AND created_at >= ?4 
                 ORDER BY id DESC LIMIT 1",
                params![issue_id, event_type, new_value.unwrap_or(""), threshold],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if duplicate {
            return;
        }
    } else if event_type == "status_change" {
        let recent: Option<(i64, Option<String>)> = conn
            .query_row(
                "SELECT id, old_value FROM issue_events 
                 WHERE issue_id = ?1 AND event_type = 'status_change' AND created_at >= ?2 
                 ORDER BY id DESC LIMIT 1",
                params![issue_id, threshold],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();

        if let Some((prev_id, prev_old)) = recent {
            if prev_old.as_deref() == new_value {
                let _ = conn.execute("DELETE FROM issue_events WHERE id = ?1", params![prev_id]);
                return;
            } else {
                let _ = conn.execute(
                    "UPDATE issue_events SET new_value = ?1, created_at = ?2 WHERE id = ?3",
                    params![new_value, now.to_rfc3339(), prev_id],
                );
                return;
            }
        }
    } else if event_type == "project_change" {
        let recent: Option<(i64, Option<String>)> = conn
            .query_row(
                "SELECT id, old_value FROM issue_events 
                 WHERE issue_id = ?1 AND event_type = 'project_change' AND created_at >= ?2 
                 ORDER BY id DESC LIMIT 1",
                params![issue_id, threshold],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();

        if let Some((prev_id, prev_old)) = recent {
            if prev_old.as_deref() == new_value {
                let _ = conn.execute("DELETE FROM issue_events WHERE id = ?1", params![prev_id]);
                return;
            } else {
                let _ = conn.execute(
                    "UPDATE issue_events SET new_value = ?1, created_at = ?2 WHERE id = ?3",
                    params![new_value, now.to_rfc3339(), prev_id],
                );
                return;
            }
        }
    } else if event_type == "title_change" {
        let recent: Option<(i64, Option<String>)> = conn
            .query_row(
                "SELECT id, old_value FROM issue_events 
                 WHERE issue_id = ?1 AND event_type = 'title_change' AND created_at >= ?2 
                 ORDER BY id DESC LIMIT 1",
                params![issue_id, threshold],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();

        if let Some((prev_id, prev_old)) = recent {
            if prev_old.as_deref() == new_value {
                let _ = conn.execute("DELETE FROM issue_events WHERE id = ?1", params![prev_id]);
                return;
            } else {
                let _ = conn.execute(
                    "UPDATE issue_events SET new_value = ?1, created_at = ?2 WHERE id = ?3",
                    params![new_value, now.to_rfc3339(), prev_id],
                );
                return;
            }
        }
    }

    let _ = conn.execute(
        "INSERT INTO issue_events (issue_id, event_type, old_value, new_value, metadata, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![issue_id, event_type, old_value, new_value, metadata, now.to_rfc3339()],
    );
}

pub fn get_attachments_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .or_else(|_| app.path().app_local_data_dir())
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let dir = app_dir.join("attachments");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn cleanup_orphaned_attachments(
    conn: &rusqlite::Connection,
    app: &tauri::AppHandle,
) -> Result<usize, String> {
    let dir = get_attachments_dir(app)?;
    if !dir.exists() {
        return Ok(0);
    }

    let mut stmt = conn
        .prepare("SELECT body FROM issues UNION ALL SELECT body FROM issue_revisions")
        .map_err(|e| e.to_string())?;

    let bodies: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    let all_text = bodies.join("\n");
    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now();
    let mut removed = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if !all_text.contains(file_name) {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(duration) = now.duration_since(modified) {
                            if duration.as_secs() > 1800 {
                                if std::fs::remove_file(&path).is_ok() {
                                    removed += 1;
                                }
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
pub fn save_attachment(
    name: String,
    data: Vec<u8>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let clean_name = std::path::Path::new(&name)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;
    let dir = get_attachments_dir(&app)?;
    let path = dir.join(clean_name);
    std::fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(clean_name.to_string())
}

#[tauri::command]
pub fn get_attachment(
    name: String,
    app: tauri::AppHandle,
) -> Result<Vec<u8>, String> {
    let clean_name = std::path::Path::new(&name)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;
    let dir = get_attachments_dir(&app)?;
    let path = dir.join(clean_name);
    std::fs::read(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_issues(state: State<DbState>) -> Result<Vec<Issue>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title, i.body, i.status, i.project_id, p.title, i.created_at 
             FROM issues i
             LEFT JOIN projects p ON i.project_id = p.id
             ORDER BY i.id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(i64, String, String, String, Option<i64>, Option<String>, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    drop(stmt);

    let mut result = Vec::with_capacity(rows.len());
    for (id, title, body, status, project_id, project_title, created_at) in rows {
        let labels = fetch_labels_for_issue(&conn, id).unwrap_or_default();
        result.push(Issue {
            id,
            title,
            body,
            status,
            project_id,
            project_title,
            labels,
            created_at,
        });
    }
    Ok(result)
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
    let lids = parse_id_vec(label_ids.unwrap_or_default());

    conn.execute(
        "INSERT INTO issues (title, body, status, project_id, created_at) VALUES (?1, ?2, 'open', ?3, ?4)",
        params![title, body, pid, created_at],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    for lid in lids {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO issue_labels (issue_id, label_id) VALUES (?1, ?2)",
            params![id, lid],
        );
    }

    let _ = conn.execute(
        "INSERT INTO issue_revisions (issue_id, body, created_at) VALUES (?1, ?2, ?3)",
        params![id, body, created_at],
    );

    let project_title: Option<String> = pid.and_then(|p| {
        conn.query_row("SELECT title FROM projects WHERE id = ?1", params![p], |r| r.get(0)).ok()
    });
    let labels = fetch_labels_for_issue(&conn, id).unwrap_or_default();

    Ok(Issue {
        id,
        title,
        body,
        status: "open".into(),
        project_id: pid,
        project_title,
        labels,
        created_at,
    })
}

#[tauri::command]
pub fn update_issue(
    id: i64,
    title: String,
    body: String,
    state: State<DbState>,
) -> Result<Issue, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let current: Option<(String, String)> = conn
        .query_row(
            "SELECT title, body FROM issues WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    if let Some((ref old_t, ref old_b)) = current {
        if old_t != &title {
            log_event(&conn, id, "title_change", Some(old_t), Some(&title), None);
        }
        if old_b != &body {
            let rev_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM issue_revisions WHERE issue_id = ?1", params![id], |r| r.get(0))
                .unwrap_or(0);

            let now = Utc::now().to_rfc3339();
            if rev_count == 0 {
                let _ = conn.execute(
                    "INSERT INTO issue_revisions (issue_id, body, created_at) VALUES (?1, ?2, ?3)",
                    params![id, old_b, now],
                );
            }

            let _ = conn.execute(
                "INSERT INTO issue_revisions (issue_id, body, created_at) VALUES (?1, ?2, ?3)",
                params![id, body, now],
            );
        }
    }

    conn.execute(
        "UPDATE issues SET title = ?1, body = ?2 WHERE id = ?3",
        params![title, body, id],
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title, i.body, i.status, i.project_id, p.title, i.created_at 
             FROM issues i
             LEFT JOIN projects p ON i.project_id = p.id
             WHERE i.id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let issue = stmt
        .query_row(params![id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    drop(stmt);

    let labels = fetch_labels_for_issue(&conn, id).unwrap_or_default();
    Ok(Issue {
        id: issue.0,
        title: issue.1,
        body: issue.2,
        status: issue.3,
        project_id: issue.4,
        project_title: issue.5,
        labels,
        created_at: issue.6,
    })
}

#[tauri::command]
pub fn toggle_issue_status(id: i64, state: State<DbState>) -> Result<Issue, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let current_status: String = conn
        .query_row("SELECT status FROM issues WHERE id = ?1", params![id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let next_status = if current_status == "open" { "closed" } else { "open" };

    log_event(
        &conn,
        id,
        "status_change",
        Some(&current_status),
        Some(next_status),
        None,
    );

    conn.execute("UPDATE issues SET status = ?1 WHERE id = ?2", params![next_status, id])
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title, i.body, i.status, i.project_id, p.title, i.created_at 
             FROM issues i
             LEFT JOIN projects p ON i.project_id = p.id
             WHERE i.id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let issue = stmt
        .query_row(params![id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    drop(stmt);

    let labels = fetch_labels_for_issue(&conn, id).unwrap_or_default();
    Ok(Issue {
        id: issue.0,
        title: issue.1,
        body: issue.2,
        status: issue.3,
        project_id: issue.4,
        project_title: issue.5,
        labels,
        created_at: issue.6,
    })
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

    let old_pid: Option<i64> = conn
        .query_row("SELECT project_id FROM issues WHERE id = ?1", params![id], |r| r.get(0))
        .unwrap_or(None);

    if old_pid != pid {
        let old_title: Option<String> = old_pid.and_then(|p| {
            conn.query_row("SELECT title FROM projects WHERE id = ?1", params![p], |r| r.get(0)).ok()
        });
        let new_title: Option<String> = pid.and_then(|p| {
            conn.query_row("SELECT title FROM projects WHERE id = ?1", params![p], |r| r.get(0)).ok()
        });
        log_event(
            &conn,
            id,
            "project_change",
            old_title.as_deref(),
            new_title.as_deref(),
            None,
        );
    }

    let mut old_labels_stmt = conn
        .prepare("SELECT label_id FROM issue_labels WHERE issue_id = ?1")
        .map_err(|e| e.to_string())?;
    let old_lids: Vec<i64> = old_labels_stmt
        .query_map(params![id], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    drop(old_labels_stmt);

    for added_id in lids.iter().filter(|x| !old_lids.contains(x)) {
        if let Ok((name, color)) = conn.query_row::<(String, String), _, _>(
            "SELECT name, color FROM labels WHERE id = ?1",
            params![added_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ) {
            log_event(&conn, id, "label_added", None, Some(&name), Some(&color));
        }
    }

    for removed_id in old_lids.iter().filter(|x| !lids.contains(x)) {
        if let Ok((name, color)) = conn.query_row::<(String, String), _, _>(
            "SELECT name, color FROM labels WHERE id = ?1",
            params![removed_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ) {
            log_event(&conn, id, "label_removed", None, Some(&name), Some(&color));
        }
    }

    conn.execute(
        "UPDATE issues SET project_id = ?1 WHERE id = ?2",
        params![pid, id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM issue_labels WHERE issue_id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    for lid in lids {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO issue_labels (issue_id, label_id) VALUES (?1, ?2)",
            params![id, lid],
        );
    }

    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title, i.body, i.status, i.project_id, p.title, i.created_at 
             FROM issues i
             LEFT JOIN projects p ON i.project_id = p.id
             WHERE i.id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let issue = stmt
        .query_row(params![id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    drop(stmt);

    let labels = fetch_labels_for_issue(&conn, id).unwrap_or_default();
    Ok(Issue {
        id: issue.0,
        title: issue.1,
        body: issue.2,
        status: issue.3,
        project_id: issue.4,
        project_title: issue.5,
        labels,
        created_at: issue.6,
    })
}

#[tauri::command]
pub fn get_issue_events(issue_id: i64, state: State<DbState>) -> Result<Vec<IssueEvent>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, issue_id, event_type, old_value, new_value, metadata, created_at 
             FROM issue_events 
             WHERE issue_id = ?1 
             ORDER BY id ASC",
        )
        .map_err(|e| e.to_string())?;

    let iter = stmt
        .query_map(params![issue_id], |row| {
            Ok(IssueEvent {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                event_type: row.get(2)?,
                old_value: row.get(3)?,
                new_value: row.get(4)?,
                metadata: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut events = Vec::new();
    for ev in iter {
        events.push(ev.map_err(|e| e.to_string())?);
    }
    Ok(events)
}

#[tauri::command]
pub fn get_issue_revisions(
    issue_id: i64,
    state: State<DbState>,
) -> Result<Vec<IssueRevision>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, issue_id, body, created_at 
             FROM issue_revisions 
             WHERE issue_id = ?1 
             ORDER BY id ASC",
        )
        .map_err(|e| e.to_string())?;

    let iter = stmt
        .query_map(params![issue_id], |row| {
            Ok(IssueRevision {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                body: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut revisions = Vec::new();
    for rev in iter {
        revisions.push(rev.map_err(|e| e.to_string())?);
    }
    Ok(revisions)
}

#[tauri::command]
pub fn get_labels(state: State<DbState>) -> Result<Vec<Label>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, color, description FROM labels ORDER BY name ASC")
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map([], |row| {
            Ok(Label {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                description: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut labels = Vec::new();
    for l in iter {
        labels.push(l.map_err(|e| e.to_string())?);
    }
    Ok(labels)
}

#[tauri::command]
pub fn create_label(
    name: String,
    color: String,
    description: String,
    state: State<DbState>,
) -> Result<Label, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO labels (name, color, description) VALUES (?1, ?2, ?3)",
        params![name, color, description],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    Ok(Label {
        id,
        name,
        color,
        description,
    })
}

#[tauri::command]
pub fn update_label(
    id: i64,
    name: String,
    color: String,
    description: String,
    state: State<DbState>,
) -> Result<Label, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE labels SET name = ?1, color = ?2, description = ?3 WHERE id = ?4",
        params![name, color, description, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(Label {
        id,
        name,
        color,
        description,
    })
}

#[tauri::command]
pub fn delete_label(id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let _ = conn.execute("DELETE FROM issue_labels WHERE label_id = ?1", params![id]);
    conn.execute("DELETE FROM labels WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_projects(state: State<DbState>) -> Result<Vec<Project>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, title, description FROM projects ORDER BY id DESC")
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    for p in iter {
        list.push(p.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[tauri::command]
pub fn create_project(
    title: String,
    description: String,
    state: State<DbState>,
) -> Result<Project, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO projects (title, description) VALUES (?1, ?2)",
        params![title, description],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    Ok(Project {
        id,
        title,
        description,
    })
}

#[tauri::command]
pub fn update_project(
    id: i64,
    title: String,
    description: String,
    state: State<DbState>,
) -> Result<Project, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE projects SET title = ?1, description = ?2 WHERE id = ?3",
        params![title, description, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(Project {
        id,
        title,
        description,
    })
}

#[tauri::command]
pub fn delete_project(id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let _ = conn.execute(
        "UPDATE issues SET project_id = NULL WHERE project_id = ?1",
        params![id],
    );
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
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