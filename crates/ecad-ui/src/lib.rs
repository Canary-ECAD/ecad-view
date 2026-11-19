//! Interfaz web de ECAD View.
//!
//! El mismo código se renderiza en el servidor (feature `ssr`) y se hidrata en
//! el navegador (feature `hydrate`, compilado a WASM por `cargo leptos`).

use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{A, Redirect, Route, Router, Routes};
use leptos_router::path;
use serde::{Deserialize, Serialize};

mod viewer;

#[cfg(feature = "ssr")]
pub mod store;

pub use viewer::PageViewer;

/// Resumen del proyecto abierto, para la cabecera y la lista de páginas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub pages: Vec<PageInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageInfo {
    /// Número de página, desde 1 (es el de la URL `/pagina/{n}`).
    pub number: usize,
    /// Designación EPLAN de la página (`==EES&EFS#01/1`).
    pub title: String,
}

#[server]
pub async fn get_project() -> Result<ProjectInfo, ServerFnError> {
    let store = use_context::<store::ProjectStore>()
        .ok_or_else(|| ServerFnError::new("no hay ningún proyecto abierto"))?;
    Ok(store.info())
}

/// Documento HTML completo que devuelve el servidor.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="es">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let project = Resource::new(|| (), |_| get_project());

    view! {
        <Stylesheet id="leptos" href="/pkg/ecad.css" />
        <Title text="ECAD View" />
        <Router>
            <div class="app">
                <header class="topbar">
                    <span class="brand">"ECAD View"</span>
                    <Transition fallback=|| ()>
                        {move || Suspend::new(async move {
                            project.await.map(|p| view! { <span class="project">{p.name}</span> })
                        })}
                    </Transition>
                </header>
                <nav class="sidebar" aria-label="Páginas">
                    <Transition fallback=|| view! { <p class="muted">"Cargando…"</p> }>
                        {move || Suspend::new(async move {
                            project.await.map(|p| view! { <PageList pages=p.pages /> })
                        })}
                    </Transition>
                </nav>
                <main class="main">
                    <Routes fallback=|| view! { <p class="muted pad">"Página no encontrada."</p> }>
                        <Route path=path!("") view=|| view! { <Redirect path="/pagina/1" /> } />
                        <Route path=path!("/pagina/:n") view=PageViewer />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn PageList(pages: Vec<PageInfo>) -> impl IntoView {
    view! {
        <ol class="pages">
            {pages
                .into_iter()
                .map(|p| {
                    view! {
                        <li>
                            <A href=format!("/pagina/{}", p.number)>
                                <span class="num">{p.number}</span>
                                <span class="title">{p.title}</span>
                            </A>
                        </li>
                    }
                })
                .collect_view()}
        </ol>
    }
}

/// Punto de entrada del WASM en el navegador.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
