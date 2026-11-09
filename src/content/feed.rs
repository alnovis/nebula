//! Post HTML for feed readers, which drop `<style>` and inline SVG.
//!
//! Every `<figure>` with an inline `<svg>` becomes an `<img>` of the PNG that
//! `scripts/publish-figures.mjs` renders and uploads under the same name.

use sha2::{Digest, Sha256};

/// PNG name of a figure: the post slug and a hash of its `<svg>` markup
pub fn figure_name(slug: &str, svg: &str) -> String {
    let hash = format!("{:x}", Sha256::digest(svg.as_bytes()));
    format!("{}-{}.png", slug, &hash[..12])
}

/// Rewrite post HTML for a feed: drop the figure stylesheet and shared defs, figures become images
pub fn feed_html(html: &str, slug: &str, figures_url: &str) -> String {
    let html = remove_block(html, "<style>", "</style>");
    let html = remove_block(&html, "<svg width=\"0\" height=\"0\"", "</svg>");
    replace_figures(&html, slug, figures_url)
}

fn remove_block(html: &str, start: &str, end: &str) -> String {
    let Some(from) = html.find(start) else {
        return html.to_string();
    };
    let Some(len) = html[from..].find(end) else {
        return html.to_string();
    };
    let mut to = from + len + end.len();
    if html[to..].starts_with('\n') {
        to += 1;
    }
    format!("{}{}", &html[..from], &html[to..])
}

fn replace_figures(html: &str, slug: &str, figures_url: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find("<figure") {
        let Some(open_len) = rest[start..].find('>') else {
            break;
        };
        let inner_start = start + open_len + 1;
        let Some(inner_len) = rest[inner_start..].find("</figure>") else {
            break;
        };
        let inner = &rest[inner_start..inner_start + inner_len];
        let end = inner_start + inner_len + "</figure>".len();

        out.push_str(&rest[..start]);
        match figure_image(inner, slug, figures_url) {
            Some(figure) => out.push_str(&figure),
            None => out.push_str(&rest[start..end]),
        }
        rest = &rest[end..];
    }

    out.push_str(rest);
    out
}

fn figure_image(inner: &str, slug: &str, figures_url: &str) -> Option<String> {
    let svg_start = inner.find("<svg")?;
    let svg_end = inner.rfind("</svg>")? + "</svg>".len();
    let svg = &inner[svg_start..svg_end];

    let alt = attribute(svg, "aria-label").unwrap_or("");
    let caption = inner
        .find("<figcaption")
        .and_then(|from| {
            inner[from..]
                .find("</figcaption>")
                .map(|len| &inner[from..from + len + "</figcaption>".len()])
        })
        .unwrap_or("");

    Some(format!(
        "<figure><img src=\"{}/{}\" alt=\"{}\">{}</figure>",
        figures_url,
        figure_name(slug, svg),
        alt,
        caption
    ))
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{}=\"", name);
    let from = tag.find(&key)? + key.len();
    let len = tag[from..].find('"')?;
    Some(&tag[from..from + len])
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://cdn.example/figures";

    #[test]
    fn test_figure_becomes_image_with_caption() {
        let svg = r#"<svg viewBox="0 0 800 200" role="img" aria-label="Two boxes"><rect class="a"/></svg>"#;
        let html = format!(
            "<p>before</p>\n<figure class=\"rum-fig\">\n{}\n<figcaption class=\"rum-cap\">Caption.</figcaption>\n</figure>\n<p>after</p>",
            svg
        );

        let out = feed_html(&html, "post", URL);

        assert_eq!(
            out,
            format!(
                "<p>before</p>\n<figure><img src=\"{}/{}\" alt=\"Two boxes\"><figcaption class=\"rum-cap\">Caption.</figcaption></figure>\n<p>after</p>",
                URL,
                figure_name("post", svg)
            )
        );
    }

    #[test]
    fn test_figure_name_matches_publish_script() {
        // node -e 'console.log(require("crypto").createHash("sha256").update("<svg></svg>").digest("hex"))'
        assert_eq!(figure_name("post", "<svg></svg>"), "post-b12e0d83ce23.png");
    }

    #[test]
    fn test_style_and_hidden_defs_removed() {
        let html = "<style>\n.a{fill:red}\n</style>\n<svg width=\"0\" height=\"0\" aria-hidden=\"true\"><defs><marker id=\"m\"/></defs></svg>\n<p>text</p>";

        assert_eq!(feed_html(html, "post", URL), "<p>text</p>");
    }

    #[test]
    fn test_figure_without_svg_kept() {
        let html = "<figure><img src=\"a.png\"><figcaption>c</figcaption></figure>";

        assert_eq!(feed_html(html, "post", URL), html);
    }

    #[test]
    fn test_figure_without_caption() {
        let svg = r#"<svg viewBox="0 0 10 10" aria-label="x"></svg>"#;
        let html = format!("<figure class=\"f\">\n{}\n</figure>", svg);

        assert_eq!(
            feed_html(&html, "p", URL),
            format!(
                "<figure><img src=\"{}/{}\" alt=\"x\"></figure>",
                URL,
                figure_name("p", svg)
            )
        );
    }
}
