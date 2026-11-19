//! Páginas de esquema en SVG.
//!
//! Las páginas que exporta EPLAN traen `<script>`, enlaces `javascript:` y
//! manejadores de eventos pensados para su propio visor. Para incrustarlas en
//! la interfaz se quitan con [`sanitize`].

use std::borrow::Cow;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::Result;

/// Elementos que se eliminan enteros, con su contenido.
const DROPPED_ELEMENTS: &[&str] = &["script", "foreignObject"];

/// Devuelve el SVG listo para incrustar en HTML: sin declaración XML ni
/// DOCTYPE, sin `<script>` ni `<foreignObject>`, sin atributos `on*` y sin
/// enlaces `javascript:`.
pub fn sanitize(svg: &str) -> Result<String> {
    sanitize_with(svg, None)
}

/// Como [`sanitize`], y además antepone `asset_base` a las rutas relativas de
/// `<image>` (EPLAN enlaza el logo como `EPLAN.png`, junto a la página).
pub fn sanitize_with(svg: &str, asset_base: Option<&str>) -> Result<String> {
    let mut reader = Reader::from_str(svg);
    let mut writer = Writer::new(Vec::with_capacity(svg.len()));
    // Profundidad dentro de un elemento eliminado (0 = no estamos en uno).
    let mut skip = 0usize;

    loop {
        let event = reader.read_event()?;
        if skip > 0 {
            match event {
                Event::Start(_) => skip += 1,
                Event::End(_) => skip -= 1,
                Event::Eof => break,
                _ => {}
            }
            continue;
        }
        match event {
            Event::Eof => break,
            Event::Decl(_) | Event::DocType(_) | Event::PI(_) => {}
            Event::Start(e) if is_dropped(&e) => skip = 1,
            Event::Empty(e) if is_dropped(&e) => {}
            Event::Start(e) => writer.write_event(Event::Start(clean(&e, asset_base)?))?,
            Event::Empty(e) => writer.write_event(Event::Empty(clean(&e, asset_base)?))?,
            other => writer.write_event(other)?,
        }
    }

    Ok(String::from_utf8(writer.into_inner()).expect("quick-xml escribe UTF-8"))
}

/// Primer `<title>` del documento: en EPLAN, la designación de la página
/// (`==EES==...&EFS#01/1`).
pub fn title(svg: &str) -> Option<String> {
    let mut reader = Reader::from_str(svg);
    let mut inside = false;
    let mut text = String::new();
    loop {
        match reader.read_event().ok()? {
            Event::Start(e) if e.local_name().as_ref() == "title" => inside = true,
            Event::Text(t) if inside => text.push_str(&t.xml10_content()),
            Event::GeneralRef(r) if inside => {
                if let Some(c) = r.resolve_char_ref().ok().flatten() {
                    text.push(c);
                } else {
                    let name = r.into_inner();
                    text.push_str(quick_xml::escape::resolve_xml_entity(&name)?);
                }
            }
            Event::End(_) if inside => return Some(text.trim().to_owned()),
            Event::Eof => return None,
            _ => {}
        }
    }
}

fn is_dropped(e: &BytesStart) -> bool {
    DROPPED_ELEMENTS.contains(&e.local_name().as_ref())
}

fn clean(e: &BytesStart, asset_base: Option<&str>) -> Result<BytesStart<'static>> {
    let is_image = e.local_name().as_ref() == "image";
    let mut out = e.to_owned();
    out.clear_attributes();
    for attr in e.attributes() {
        let attr = attr?;
        let key = attr.key.local_name();
        let key = key.as_ref();
        if key.len() > 2 && key.as_bytes()[..2].eq_ignore_ascii_case(b"on") {
            continue;
        }
        if key == "href" && is_script_url(&attr.value) {
            continue;
        }
        if let (true, "href", Some(base)) = (is_image, key, asset_base)
            && is_relative(&attr.value)
        {
            out.push_attribute(Attribute {
                key: attr.key,
                value: Cow::Owned(format!("{base}{}", attr.value)),
            });
            continue;
        }
        out.push_attribute(attr);
    }
    Ok(out)
}

fn is_relative(url: &str) -> bool {
    !(url.is_empty() || url.starts_with(['/', '#']) || url.contains(':'))
}

fn is_script_url(value: &str) -> bool {
    let v: String = value
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(11)
        .collect();
    v.eq_ignore_ascii_case("javascript:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_scripts_handlers_and_js_links() {
        let input = r#"<?xml version="1.0"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "x.dtd">
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" onload="init()">
<title>==A&amp;EFS#01/1</title>
<script xlink:href="SVGScripting.js"/>
<script><![CDATA[ alert(1) ]]><g/></script>
<a xlink:href=" JavaScript:jumpToFunction('x')" class="L"><text onclick="go()">K1</text></a>
<image xlink:href="EPLAN.png"/>
</svg>"#;
        let out = sanitize(input).unwrap();
        assert!(!out.contains("script"), "{out}");
        assert!(!out.contains("onload") && !out.contains("onclick"), "{out}");
        assert!(!out.to_lowercase().contains("javascript"), "{out}");
        assert!(!out.contains("<?xml") && !out.contains("DOCTYPE"), "{out}");
        assert!(out.contains(r#"<a class="L">"#), "{out}");
        assert!(out.contains(r#"xlink:href="EPLAN.png""#), "{out}");
        assert!(out.contains("==A&amp;EFS#01/1"), "{out}");
    }

    #[test]
    fn prefixes_relative_images() {
        let input = r##"<svg><image href="EPLAN.png"/><image href="data:image/png;base64,AA"/><use href="#s1"/></svg>"##;
        let out = sanitize_with(input, Some("/datos/recursos/")).unwrap();
        assert!(out.contains(r#"href="/datos/recursos/EPLAN.png""#), "{out}");
        assert!(out.contains(r#"href="data:image/png;base64,AA""#), "{out}");
        assert!(out.contains(r##"<use href="#s1"/>"##), "{out}");
    }

    #[test]
    fn reads_title() {
        let svg = r#"<svg><title> ==EES&amp;EFS#01/1 </title><g/></svg>"#;
        assert_eq!(title(svg).as_deref(), Some("==EES&EFS#01/1"));
    }
}
