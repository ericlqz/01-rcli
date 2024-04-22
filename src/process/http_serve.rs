use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Serving {:?} on http://{}", path, addr);
    let state = HttpServeState { path: path.clone() };

    let router = Router::new()
        .route("/*path", get(file_handler))
        .nest_service("/tower", ServeDir::new(path))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> (StatusCode, Html<String>) {
    let p = std::path::Path::new(&state.path).join(path.clone());
    info!("Reading file {:?}", p);
    if !p.exists() {
        (
            StatusCode::NOT_FOUND,
            Html(format!("File {} not found", p.display())),
        )
    } else {
        match p.is_dir() {
            false => match tokio::fs::read_to_string(p).await {
                Ok(content) => {
                    info!("Read {} bytes", content.len());
                    (StatusCode::OK, Html(content))
                }
                Err(e) => {
                    warn!("Error reading file: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Html(e.to_string()))
                }
            },
            true => {
                // TODO: 待优化项: 1. 支持根目录; 2. 支持文件内容展示
                let sub_items = list_files_and_directories(p).await.unwrap();
                let si = sub_items
                    .into_iter()
                    .map(|i| format!("<li><a href=\"/{}/{}\">{}</a></li>", path, i, i))
                    .collect::<Vec<String>>();
                let res = format!("<html><body><ul>{}</ul></body></html>", si.join("\n"));

                (StatusCode::OK, Html(res))
            }
        }
    }
}

async fn list_files_and_directories(dir: PathBuf) -> anyhow::Result<Vec<String>, anyhow::Error> {
    let mut entries = tokio::fs::read_dir(dir).await?;
    let mut file_names = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_owned());

        if let Some(name) = file_name {
            if path.is_dir() {
                file_names.push(name + "/");
            } else {
                file_names.push(name);
            }
        }
    }

    Ok(file_names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_handler() {
        let state = Arc::new(HttpServeState {
            path: PathBuf::from("."),
        });
        let (status, _) = file_handler(State(state), Path("Cargo.toml".to_string())).await;
        assert_eq!(status, StatusCode::OK);
        // assert!(content.trim().starts_with("[package"));
    }
}
