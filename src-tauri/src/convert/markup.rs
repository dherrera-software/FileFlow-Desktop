use scraper::{ElementRef, Html, Node};

use super::text::{html_document, normalize_newlines};

pub fn markdown_to_html(markdown: &str, title: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    let normalized = normalize_newlines(markdown);
    let parser = pulldown_cmark::Parser::new_ext(&normalized, options);
    let mut body = String::new();
    pulldown_cmark::html::push_html(&mut body, parser);
    html_document(title, body.trim(), true)
}

pub fn html_to_markdown(html: &str) -> Result<String, String> {
    let document = Html::parse_document(html);
    let markdown = document
        .root_element()
        .children()
        .map(walk_node)
        .collect::<String>();
    let markdown = markdown
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    let collapsed = collapse_blank_lines(&markdown).trim().to_string();
    Ok(format!("{collapsed}\n"))
}

fn walk_node(node: ego_tree::NodeRef<'_, Node>) -> String {
    match node.value() {
        Node::Text(text) => collapse_inline_space(text),
        Node::Element(element) => {
            let tag = element.name();
            if matches!(tag, "script" | "style" | "noscript" | "template") {
                return String::new();
            }
            walk_element(ElementRef::wrap(node).expect("element node"), tag)
        }
        _ => String::new(),
    }
}

fn walk_element(element: ElementRef<'_>, tag: &str) -> String {
    let children = || element.children().map(walk_node).collect::<String>();
    match tag {
        "br" => "  \n".to_string(),
        "hr" => "\n\n---\n\n".to_string(),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag.chars().nth(1).unwrap_or('1');
            let hashes = "#".repeat(level.to_digit(10).unwrap_or(1) as usize);
            format!("\n\n{hashes} {}\n\n", children().trim())
        }
        "p" => format!("\n\n{}\n\n", children().trim()),
        "strong" | "b" => format!("**{}**", children()),
        "em" | "i" => format!("*{}*", children()),
        "del" | "s" => format!("~~{}~~", children()),
        "code" => format!("`{}`", plain_text(element)),
        "pre" => {
            let code = plain_text(element).trim_end_matches('\n').to_string();
            format!("\n\n```\n{code}\n```\n\n")
        }
        "a" => {
            let href = element.attr("href").unwrap_or("");
            format!("[{}]({href})", children())
        }
        "img" => {
            let alt = element.attr("alt").unwrap_or("");
            let src = element.attr("src").unwrap_or("");
            format!("![{alt}]({src})")
        }
        "ul" | "ol" => {
            let items = element
                .children()
                .filter_map(ElementRef::wrap)
                .filter(|child| child.value().name() == "li")
                .enumerate()
                .map(|(index, child)| {
                    let marker = if tag == "ol" {
                        format!("{}. ", index + 1)
                    } else {
                        "- ".to_string()
                    };
                    let content = child
                        .children()
                        .map(walk_node)
                        .collect::<String>()
                        .trim()
                        .replace('\n', " ");
                    format!("{marker}{content}")
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("\n\n{items}\n\n")
        }
        "blockquote" => {
            let content = children().trim().to_string();
            let quoted = content
                .lines()
                .map(|line| format!("> {line}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("\n\n{quoted}\n\n")
        }
        "table" => table_to_markdown(element),
        _ => children(),
    }
}

fn table_to_markdown(element: ElementRef<'_>) -> String {
    let rows = element
        .descendants()
        .filter_map(ElementRef::wrap)
        .filter(|node| node.value().name() == "tr")
        .map(|row| {
            row.children()
                .filter_map(ElementRef::wrap)
                .filter(|cell| matches!(cell.value().name(), "td" | "th"))
                .map(|cell| plain_text(cell).trim().replace('|', "\\|"))
                .collect::<Vec<_>>()
        })
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return String::new();
    }
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    let normalize = |row: &[String]| {
        let mut next = row.to_vec();
        next.resize(width, String::new());
        next
    };
    let head = normalize(&rows[0]);
    let separator = vec!["---".to_string(); width];
    let mut lines = vec![format!("| {} |", head.join(" | ")), format!("| {} |", separator.join(" | "))];
    for row in rows.iter().skip(1) {
        lines.push(format!("| {} |", normalize(row).join(" | ")));
    }
    format!("\n\n{}\n\n", lines.join("\n"))
}

fn plain_text(element: ElementRef<'_>) -> String {
    element
        .text()
        .collect::<Vec<_>>()
        .join("")
}

fn collapse_inline_space(value: &str) -> String {
    let mut output = String::new();
    let mut pending = false;
    for character in value.chars() {
        if character.is_whitespace() {
            pending = true;
        } else {
            if pending && !output.is_empty() {
                output.push(' ');
            }
            pending = false;
            output.push(character);
        }
    }
    if pending && !output.is_empty() {
        output.push(' ');
    }
    output
}

fn collapse_blank_lines(value: &str) -> String {
    let mut output = String::new();
    let mut blanks = 0;
    for line in value.lines() {
        if line.trim().is_empty() {
            blanks += 1;
            if blanks <= 2 && !output.is_empty() {
                output.push('\n');
            }
        } else {
            blanks = 0;
            output.push_str(line);
            output.push('\n');
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_heading_and_emphasis_become_html() {
        let html = markdown_to_html("# Título\n\nTexto **fuerte** y `code`", "Doc");
        assert!(html.contains("<h1>Título</h1>"));
        assert!(html.contains("<strong>fuerte</strong>"));
        assert!(html.contains("<code>code</code>"));
        assert!(html.contains("<title>Doc</title>"));
    }

    #[test]
    fn html_heading_list_and_link_become_markdown() {
        let markdown = html_to_markdown(
            "<html><body><h2>Hola</h2><p>Visita <a href=\"https://example.com\">ejemplo</a></p><ul><li>Uno</li><li>Dos</li></ul><script>alert(1)</script></body></html>",
        )
        .unwrap();
        assert!(markdown.contains("## Hola"));
        assert!(markdown.contains("[ejemplo](https://example.com)"));
        assert!(markdown.contains("- Uno"));
        assert!(markdown.contains("- Dos"));
        assert!(!markdown.contains("alert"));
    }

    #[test]
    fn escape_html_is_used_for_titles() {
        let html = markdown_to_html("Hola", "<script>");
        assert!(html.contains("<title>&lt;script&gt;</title>"));
    }
}
