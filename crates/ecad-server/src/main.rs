//! `ecad-server`: sirve la interfaz (Leptos SSR + WASM) y los datos del
//! proyecto.
//!
//! Prueba de concepto del sprint 0: abre un `.epdz` al arrancar y sirve sus
//! páginas. Se arranca con `cargo leptos watch` (ver `docs/v2/decisiones.md`);
//! el `.epdz` se elige con `ECAD_EPDZ` (por defecto, `samples/ejemplo.epdz`).

use std::path::{Path, PathBuf};
use std::time::Instant;

use axum::Router;
use axum::extract::{Extension, Path as UrlPath};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use ecad_core::{Epdz, svg};
use ecad_ui::store::{Project, ProjectStore, StoredPage};
use ecad_ui::{App, shell};
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::compression::CompressionLayer;
use tracing_subscriber::EnvFilter;

/// Prefijo con el que se sirven las imágenes que enlazan las páginas.
const ASSETS: &str = "/datos/recursos/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let epdz = std::env::var_os("ECAD_EPDZ")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("samples/ejemplo.epdz"));
    let store = {
        let epdz = epdz.clone();
        tokio::task::spawn_blocking(move || load(&epdz)).await??
    };

    let conf = get_configuration(None)?;
    let options = conf.leptos_options;
    let addr = options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .route("/datos/paginas/{n}", get(page_svg))
        .route("/datos/recursos/{name}", get(asset))
        .leptos_routes_with_context(
            &options,
            routes,
            {
                let store = store.clone();
                move || provide_context(store.clone())
            },
            {
                let options = options.clone();
                move || shell(options.clone())
            },
        )
        // El 404 también renderiza la App, que necesita el proyecto en contexto.
        .fallback(leptos_axum::file_and_error_handler_with_context(
            {
                let store = store.clone();
                move || provide_context(store.clone())
            },
            shell,
        ))
        .layer(Extension(store))
        .layer(CompressionLayer::new())
        .with_state(options);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("escuchando en http://{addr}");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

/// Abre el `.epdz` y prepara las páginas saneadas, en el orden del proyecto.
fn load(path: &Path) -> Result<ProjectStore, ecad_core::Error> {
    let t = Instant::now();
    let epdz = Epdz::open(path)?;
    let project = epdz.project();
    let mut pages = Vec::new();
    for page in &project.pages {
        let Some(data) = page.file.as_deref().and_then(|f| epdz.get(f)) else {
            tracing::warn!("{}: sin SVG en el archivo", page.name);
            continue;
        };
        let text = String::from_utf8_lossy(data);
        let title = match (page.designation.full.as_str(), &page.description) {
            ("", None) => svg::title(&text).unwrap_or_else(|| page.name.clone()),
            ("", Some(d)) => d.clone(),
            (designation, None) => designation.to_owned(),
            (designation, Some(d)) => format!("{designation} {d}"),
        };
        pages.push(StoredPage {
            title,
            svg: svg::sanitize_with(&text, Some(ASSETS))?,
        });
    }
    let assets = epdz
        .images()
        .filter(|e| e.name.contains("/pagesvg/") && content_type(&e.name).is_some())
        .filter_map(|e| Some((e.name.rsplit('/').next()?.to_owned(), e.data.clone())))
        .collect();

    let name = project.header.name.clone().unwrap_or_else(|| {
        path.file_stem()
            .map_or_else(String::new, |s| s.to_string_lossy().into_owned())
    });
    tracing::info!(
        "{name}: {} páginas, {} conexiones en {:?}",
        pages.len(),
        project.connections.len(),
        t.elapsed()
    );
    Ok(ProjectStore(std::sync::Arc::new(Project {
        name,
        pages,
        assets,
    })))
}

async fn page_svg(
    Extension(store): Extension<ProjectStore>,
    UrlPath(n): UrlPath<usize>,
) -> Response {
    match store.page(n) {
        Some(page) => (
            [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
            page.svg.clone(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn asset(
    Extension(store): Extension<ProjectStore>,
    UrlPath(name): UrlPath<String>,
) -> Response {
    match (store.0.assets.get(&name), content_type(&name)) {
        (Some(data), Some(ct)) => ([(header::CONTENT_TYPE, ct)], data.clone()).into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

fn content_type(name: &str) -> Option<&'static str> {
    let ext = Path::new(name).extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        _ => return None,
    })
}
