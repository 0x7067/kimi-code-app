//! Markdown → HTML rendering for agent messages.

use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

pub fn md_to_html(text: &str) -> String {
    let options =
        pulldown_cmark::Options::ENABLE_TABLES | pulldown_cmark::Options::ENABLE_STRIKETHROUGH;
    let parser = pulldown_cmark::Parser::new_ext(text, options);

    let mut in_diff = false;
    let mut diff_buf = String::new();
    let mut events: Vec<Event<'_>> = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
                if lang.as_ref() == "diff" =>
            {
                in_diff = true;
                diff_buf.clear();
                events.push(Event::Html(r#"<div class="md-diff">"#.into()));
            }
            Event::End(TagEnd::CodeBlock) if in_diff => {
                in_diff = false;
                for line in diff_buf.lines() {
                    let class = if line.starts_with('+') && !line.starts_with("+++") {
                        "add"
                    } else if line.starts_with('-') && !line.starts_with("---") {
                        "del"
                    } else if line.starts_with("@@") {
                        "hunk"
                    } else {
                        ""
                    };
                    let esc = html_escape(line);
                    events.push(Event::Html(
                        format!(
                            r#"<span class="md-diff-line {}">{}</span>"#,
                            class, esc
                        )
                        .into(),
                    ));
                }
                events.push(Event::Html("</div>".into()));
            }
            Event::Text(t) if in_diff => {
                diff_buf.push_str(&t);
            }
            // F-002.8: wrap regular code blocks so a hover copy button can be
            // overlaid; clicks are handled by a delegated listener in app.rs.
            Event::Start(Tag::CodeBlock(_)) => {
                events.push(Event::Html(
                    r#"<div class="code-block"><button class="code-copy-btn" type="button" aria-label="Copy code">Copy</button>"#
                        .into(),
                ));
                events.push(event);
            }
            Event::End(TagEnd::CodeBlock) => {
                events.push(event);
                events.push(Event::Html("</div>".into()));
            }
            _ => events.push(event),
        }
    }

    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, events.into_iter());
    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_blocks_get_copy_button_wrapper() {
        let html = md_to_html("```rust\nfn main() {}\n```");
        assert!(html.contains(r#"<div class="code-block">"#));
        assert!(html.contains(r#"<button class="code-copy-btn""#));
        assert!(html.contains("fn main() {}"));
        assert!(html.contains("</div>"));
    }

    #[test]
    fn diff_blocks_keep_diff_rendering_without_copy_button() {
        let html = md_to_html("```diff\n+added\n-removed\n```");
        assert!(html.contains(r#"<div class="md-diff">"#));
        assert!(html.contains("md-diff-line add"));
        assert!(!html.contains("code-copy-btn"));
    }

    #[test]
    fn plain_text_is_untouched() {
        let html = md_to_html("hello **world**");
        assert!(!html.contains("code-block"));
        assert!(html.contains("<strong>world</strong>"));
    }
}
