use table_to_title::{export, format, parse};

use axum::{
    Router,
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

// ── Error type ────────────────────────────────────────────────────────────────

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.0.to_string()).into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(e.into())
    }
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct ApiRequest {
    tsv: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    options: format::FormatOptions,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn serve_index() -> impl IntoResponse {
    let html = include_str!("../static/index.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

async fn api_preview(
    Json(req): Json<ApiRequest>,
) -> Result<impl IntoResponse, AppError> {
    let rows = parse::parse_tsv(&req.tsv)?;
    let data = format::build_author_data(&rows);
    let html = format::build_preview_html(&data, &req.options, &req.title);
    Ok(([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html))
}

async fn api_export_docx(
    Json(req): Json<ApiRequest>,
) -> Result<impl IntoResponse, AppError> {
    let rows = parse::parse_tsv(&req.tsv)?;
    let data = format::build_author_data(&rows);
    let bytes = export::build_docx(&data, &req.options, &req.title)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"authors.docx\""),
    );
    Ok((headers, Body::from(bytes)))
}

/// Accept raw Excel bytes, convert to TSV, return as plain text.
/// The frontend puts this in the textarea and triggers a normal preview.
async fn api_upload(body: Bytes) -> Result<impl IntoResponse, AppError> {
    let tsv = parse::excel_bytes_to_tsv(&body)?;

    if tsv.trim().is_empty() {
        return Err(anyhow::anyhow!("The first sheet appears to be empty.").into());
    }

    // Validate columns up front so the user gets a clear error immediately
    parse::parse_tsv(&tsv).map_err(|e| {
        anyhow::anyhow!(
            "Excel parsed OK but columns don't match the expected biorxiv format: {e}\n\
             Make sure the author table is on the first sheet."
        )
    })?;

    Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], tsv))
}

async fn api_export_txt(
    Json(req): Json<ApiRequest>,
) -> Result<impl IntoResponse, AppError> {
    let rows = parse::parse_tsv(&req.tsv)?;
    let data = format::build_author_data(&rows);
    let text = export::build_txt(&data, &req.options, &req.title);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"authors.txt\""),
    );
    Ok((headers, text))
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = format!("127.0.0.1:{}", port);

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/preview", post(api_preview))
        .route("/api/export/docx", post(api_export_docx))
        .route("/api/export/txt", post(api_export_txt))
        .route("/api/upload", post(api_upload))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    let url = format!("http://localhost:{}", port);
    println!("Listening on {}", url);

    // Open browser after binding (non-fatal if it fails)
    tokio::task::spawn_blocking(move || {
        let _ = open::that(&url);
    });

    axum::serve(listener, app).await.unwrap();
}
