//! Visor de una página de esquema: carga el SVG saneado del servidor, lo
//! incrusta y permite zoom (rueda, botones) y desplazamiento (arrastrar).
//!
//! El zoom cambia el `viewBox` del SVG en lugar de escalar con CSS, así las
//! líneas se redibujan nítidas a cualquier aumento. El `viewBox` siempre tiene
//! la proporción del contenedor, de modo que pasar de píxeles a coordenadas
//! del dibujo es una regla de tres.

use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use wasm_bindgen::JsCast;

/// Factor de zoom de los botones y del teclado.
const STEP: f64 = 1.25;
/// Límites de zoom respecto a la página ajustada.
const MAX_ZOOM: f64 = 40.0;
const MIN_ZOOM: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq)]
struct ViewBox {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl ViewBox {
    fn parse(s: &str) -> Option<Self> {
        let v: Vec<f64> = s
            .split([' ', ','])
            .filter(|p| !p.is_empty())
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()?;
        match v[..] {
            [x, y, w, h] if w > 0.0 && h > 0.0 => Some(Self { x, y, w, h }),
            _ => None,
        }
    }

    /// La página entera centrada, con la proporción `aspect` (ancho / alto).
    fn fit(page: Self, aspect: f64) -> Self {
        let margin = 1.02;
        let (mut w, mut h) = (page.w * margin, page.h * margin);
        if w / h < aspect {
            w = h * aspect;
        } else {
            h = w / aspect;
        }
        Self {
            x: page.x + (page.w - w) / 2.0,
            y: page.y + (page.h - h) / 2.0,
            w,
            h,
        }
    }

    /// Zoom de factor `f` (< 1 acerca) manteniendo fijo el punto (`px`, `py`)
    /// expresado en fracción del contenedor (0..1).
    fn zoom_at(self, f: f64, px: f64, py: f64) -> Self {
        let (ux, uy) = (self.x + px * self.w, self.y + py * self.h);
        let (w, h) = (self.w * f, self.h * f);
        Self {
            x: ux - px * w,
            y: uy - py * h,
            w,
            h,
        }
    }
}

impl std::fmt::Display for ViewBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {} {}", self.x, self.y, self.w, self.h)
    }
}

#[component]
pub fn PageViewer() -> impl IntoView {
    let params = use_params_map();
    let number = Memo::new(move |_| {
        params
            .read()
            .get("n")
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(1)
    });

    let host = NodeRef::<html::Div>::new();
    // Caja de la página tal como viene en el SVG, y la vista actual.
    let page_box = RwSignal::new(None::<ViewBox>);
    let view_box = RwSignal::new(None::<ViewBox>);
    let status = RwSignal::new(String::new());
    // Identifica la última carga, para descartar respuestas atrasadas.
    let generation = StoredValue::new(0u32);
    let drag = StoredValue::new(None::<(i32, i32)>);

    let size = move || {
        host.get_untracked()
            .map(|el| el.get_bounding_client_rect())
            .filter(|r| r.width() > 0.0 && r.height() > 0.0)
    };
    let fit = move || {
        if let (Some(page), Some(r)) = (page_box.get_untracked(), size()) {
            view_box.set(Some(ViewBox::fit(page, r.width() / r.height())));
        }
    };
    let zoom = move |f: f64, px: f64, py: f64| {
        let (Some(vb), Some(page), Some(r)) =
            (view_box.get_untracked(), page_box.get_untracked(), size())
        else {
            return;
        };
        let fitted = ViewBox::fit(page, r.width() / r.height());
        let level = fitted.w / (vb.w * f);
        if (MIN_ZOOM..=MAX_ZOOM).contains(&level) {
            view_box.set(Some(vb.zoom_at(f, px, py)));
        }
    };

    // Carga del SVG al entrar o cambiar de página (solo en el navegador).
    Effect::new(move |_| {
        let n = number.get();
        let Some(el) = host.get() else { return };
        let this = generation.get_value() + 1;
        generation.set_value(this);
        status.set("Cargando…".into());
        spawn_local(async move {
            let t0 = now();
            let result = fetch_text(&format!("/datos/paginas/{n}")).await;
            if generation.get_value() != this {
                return;
            }
            match result {
                Ok(svg) => {
                    el.set_inner_html(&svg);
                    let Some(root) = el.first_element_child() else {
                        status.set("La página está vacía".into());
                        return;
                    };
                    let page = root
                        .get_attribute("viewBox")
                        .and_then(|v| ViewBox::parse(&v));
                    for (k, v) in [
                        ("width", "100%"),
                        ("height", "100%"),
                        ("preserveAspectRatio", "xMidYMid meet"),
                    ] {
                        let _ = root.set_attribute(k, v);
                    }
                    page_box.set(page);
                    fit();
                    status.set(format!("{:.0} ms", now() - t0));
                }
                Err(e) => status.set(format!("Error al cargar: {e}")),
            }
        });
    });

    // Aplica la vista al SVG.
    Effect::new(move |_| {
        if let (Some(vb), Some(el)) = (view_box.get(), host.get())
            && let Some(root) = el.first_element_child()
        {
            let _ = root.set_attribute("viewBox", &vb.to_string());
        }
    });

    let resize = window_event_listener(ev::resize, move |_| fit());
    on_cleanup(move || resize.remove());

    let on_wheel = move |e: ev::WheelEvent| {
        e.prevent_default();
        let Some(r) = size() else { return };
        let px = (f64::from(e.client_x()) - r.left()) / r.width();
        let py = (f64::from(e.client_y()) - r.top()) / r.height();
        // Proporcional al desplazamiento: suave con touchpad, por pasos con rueda.
        let f = (e.delta_y().clamp(-200.0, 200.0) * 0.0015).exp();
        zoom(f, px, py);
    };
    let on_down = move |e: ev::PointerEvent| {
        if e.button() != 0 {
            return;
        }
        if let Some(t) = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        {
            let _ = t.set_pointer_capture(e.pointer_id());
        }
        drag.set_value(Some((e.client_x(), e.client_y())));
    };
    let on_move = move |e: ev::PointerEvent| {
        let (Some((x0, y0)), Some(vb), Some(r)) =
            (drag.get_value(), view_box.get_untracked(), size())
        else {
            return;
        };
        let (dx, dy) = (e.client_x() - x0, e.client_y() - y0);
        drag.set_value(Some((e.client_x(), e.client_y())));
        view_box.set(Some(ViewBox {
            x: vb.x - f64::from(dx) * vb.w / r.width(),
            y: vb.y - f64::from(dy) * vb.h / r.height(),
            ..vb
        }));
    };
    let on_up = move |_: ev::PointerEvent| drag.set_value(None);
    let on_key = move |e: ev::KeyboardEvent| match e.key().as_str() {
        "+" | "=" => zoom(1.0 / STEP, 0.5, 0.5),
        "-" => zoom(STEP, 0.5, 0.5),
        "0" => fit(),
        _ => {}
    };

    let percent = move || {
        let (vb, page, r) = (view_box.get()?, page_box.get()?, size()?);
        let fitted = ViewBox::fit(page, r.width() / r.height());
        Some(format!("{:.0} %", 100.0 * fitted.w / vb.w))
    };

    view! {
        <section class="viewer">
            <div class="toolbar" role="toolbar" aria-label="Vista de página">
                <button on:click=move |_| zoom(STEP, 0.5, 0.5) title="Alejar (−)">"−"</button>
                <span class="zoom">{percent}</span>
                <button on:click=move |_| zoom(1.0 / STEP, 0.5, 0.5) title="Acercar (+)">"+"</button>
                <button on:click=move |_| fit() title="Ajustar página (0)">"Ajustar"</button>
                <span class="spacer"></span>
                <span class="muted">"Página " {number}</span>
                <span class="muted status">{move || status.get()}</span>
            </div>
            <div
                class="canvas"
                node_ref=host
                tabindex="0"
                on:wheel=on_wheel
                on:pointerdown=on_down
                on:pointermove=on_move
                on:pointerup=on_up
                on:pointercancel=on_up
                on:dblclick=move |_| fit()
                on:keydown=on_key
            ></div>
        </section>
    }
}

fn now() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0.0, |p| p.now())
}

async fn fetch_text(url: &str) -> Result<String, String> {
    use wasm_bindgen_futures::JsFuture;
    let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let window = web_sys::window().ok_or("sin ventana")?;
    let resp: web_sys::Response = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(js)?
        .dyn_into()
        .map_err(js)?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let text = JsFuture::from(resp.text().map_err(js)?).await.map_err(js)?;
    text.as_string().ok_or_else(|| "respuesta sin texto".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_keeps_page_centered_with_container_aspect() {
        let page = ViewBox::parse("0 0 420 297").unwrap();
        let vb = ViewBox::fit(page, 2.0);
        assert!((vb.w / vb.h - 2.0).abs() < 1e-9);
        assert!((vb.x + vb.w / 2.0 - 210.0).abs() < 1e-9);
        assert!((vb.y + vb.h / 2.0 - 148.5).abs() < 1e-9);
    }

    #[test]
    fn zoom_keeps_point_under_cursor() {
        let vb = ViewBox {
            x: 0.0,
            y: 0.0,
            w: 400.0,
            h: 200.0,
        };
        let z = vb.zoom_at(0.5, 0.25, 0.75);
        assert_eq!((z.x + 0.25 * z.w, z.y + 0.75 * z.h), (100.0, 150.0));
        assert_eq!((z.w, z.h), (200.0, 100.0));
    }
}
