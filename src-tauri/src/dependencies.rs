use crate::models::Dependency;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Deserialize;
use tauri::{AppHandle, Emitter};

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    html_url: String,
    published_at: Option<String>,
}

pub fn parse_repo_slug(input: &str) -> Option<(String, String)> {
    let clean = input
        .trim()
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_start_matches("github.com/")
        .trim_end_matches('/')
        .split('#')
        .next()?
        .split('?')
        .next()?;
    let parts: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

pub fn fetch_all_dependencies(conn: &Connection) -> Result<Vec<Dependency>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, repo_owner, repo_name, last_seen_tag, latest_tag, release_name, release_url, published_at, has_update, last_checked_at, created_at 
             FROM dependencies ORDER BY has_update DESC, name ASC"
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |r| {
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
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn poll_dependencies(conn: &Connection, app: &AppHandle, force: bool) -> Result<Vec<Dependency>, String> {
    let client = reqwest::blocking::Client::builder().user_agent("local-git-issues").build().map_err(|e| e.to_string())?;

    let deps = fetch_all_dependencies(conn)?;
    let now = Utc::now();

    for dep in &deps {
        if !force {
            if let Some(ref last) = dep.last_checked_at {
                if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(last) {
                    if (now - parsed.with_timezone(&Utc)).num_hours() < 24 { continue; }
                }
            }
        }

        if let Ok(res) = client
            .get(format!("https://api.github.com/repos/{}/{}/releases/latest", dep.repo_owner, dep.repo_name))
            .header("Accept", "application/vnd.github+json")
            .send()
        {
            if res.status().is_success() {
                if let Ok(release) = res.json::<GithubRelease>() {
                    let is_new = dep.last_seen_tag.as_ref().is_some_and(|seen| seen != &release.tag_name);
                    let initial = dep.last_seen_tag.is_none();
                    let target_last_seen = if initial { Some(release.tag_name.clone()) } else { dep.last_seen_tag.clone() };
                    let has_update = if is_new { true } else { dep.has_update };

                    let _ = conn.execute(
                        "UPDATE dependencies 
                         SET last_seen_tag = ?1, latest_tag = ?2, release_name = ?3, release_url = ?4, published_at = ?5, has_update = ?6, last_checked_at = ?7 
                         WHERE id = ?8",
                        params![
                            target_last_seen,
                            release.tag_name,
                            release.name.unwrap_or_else(|| release.tag_name.clone()),
                            release.html_url,
                            release.published_at,
                            if has_update { 1 } else { 0 },
                            now.to_rfc3339(),
                            dep.id
                        ],
                    );
                }
            }
        }
    }

    let _ = app.emit("dependencies-updated", ());
    fetch_all_dependencies(conn)
}