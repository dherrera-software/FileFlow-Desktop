pub fn txt_to_markdown(text: &str) -> String {
    let normalized = normalize_newlines(text).trim().to_string();
    if normalized.is_empty() {
        return "\n".to_string();
    }
    let paragraphs = normalized
        .split("\n\n")
        .map(str::trim)
        .filter(|paragraph| !paragraph.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{paragraphs}\n")
}

pub fn txt_to_html(text: &str, title: &str) -> String {
    let paragraphs = normalize_newlines(text)
        .split("\n\n")
        .map(|paragraph| {
            let body = escape_html(paragraph).replace('\n', "<br>");
            format!("<p>{body}</p>")
        })
        .collect::<Vec<_>>()
        .join("\n");
    html_document(title, &paragraphs, false)
}

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn html_document(title: &str, body: &str, rich: bool) -> String {
    let extra = if rich {
        "\npre{overflow:auto;padding:1rem;border-radius:.5rem;background:#f3f4f1}\ncode{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.92em}\nblockquote{margin:1rem 0;padding-left:1rem;border-left:3px solid #d6d9cf;color:#4c5048}\nimg{max-width:100%}\ntable{border-collapse:collapse}td,th{border:1px solid #d6d9cf;padding:.4rem .6rem}"
    } else {
        ""
    };
    let content = if body.trim().is_empty() {
        "<p></p>"
    } else {
        body
    };
    format!(
        r#"<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{}</title>
<style>
body{{max-width:46rem;margin:3rem auto;padding:0 1.25rem;font-family:system-ui,-apple-system,"Segoe UI",sans-serif;line-height:1.65;color:#1a1d16;background:#fff}}
p{{margin:0 0 1rem}}{extra}
</style>
</head>
<body>
{content}
</body>
</html>"#,
        escape_html(title)
    )
}

pub fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_keeps_paragraphs() {
        assert_eq!(txt_to_markdown("Uno\n\nDos\n"), "Uno\n\nDos\n");
    }

    #[test]
    fn html_escapes_markup() {
        let html = txt_to_html("a < b & c", "Nota");
        assert!(html.contains("<title>Nota</title>"));
        assert!(html.contains("a &lt; b &amp; c"));
    }
}
