//! 登録したWebサイトのアイコンを、そのサイトから直接取ってくる。
//!
//! 外部のアイコン取得サービスを使うと登録先のドメインが第三者に渡るので使わない。
//! HTMLに書かれたアイコンのうち大きいものから順に試し、無ければ /favicon.ico を見る。

use image::{imageops::FilterType, ImageFormat};
use std::{
    fs,
    io::Read,
    path::Path,
    time::{Duration, Instant},
};
use url::Url;

/// 取得全体にかける時間の上限。これを過ぎたら諦めて 🌐 の表示に任せる。
const DEADLINE: Duration = Duration::from_secs(8);
/// 1回の通信の上限。
const REQUEST_TIMEOUT: Duration = Duration::from_secs(4);
const MAX_HTML_BYTES: u64 = 1024 * 1024;
const MAX_IMAGE_BYTES: u64 = 2 * 1024 * 1024;
/// キーの表示に足りる大きさに縮めて保存する。
const SAVED_SIZE: u32 = 256;
/// 試す候補の数。大きい順に並べるので、上位だけで足りる。
const MAX_CANDIDATES: usize = 5;
/// この大きさが取れたら、残りの候補は試さない。
const GOOD_ENOUGH: u32 = 128;
/// サイズ指定の無い apple-touch-icon は、仕様上の標準である180pxとみなす。
const APPLE_TOUCH_ICON_SIZE: u32 = 180;

/// 取れなかった理由は画面に出さないので、文字列で十分。
pub fn fetch(page_url: &str, destination: &Path) -> Result<(), String> {
    let started = Instant::now();
    // ボット除けで弾かれにくいよう、ブラウザ互換のUser-Agentを名乗る。
    let agent = ureq::AgentBuilder::new()
        .timeout(REQUEST_TIMEOUT)
        .redirects(5)
        .user_agent(concat!(
            "Mozilla/5.0 (compatible; OpenSesame/",
            env!("CARGO_PKG_VERSION"),
            ")"
        ))
        .build();

    let mut candidates = Vec::new();
    // HTMLが取れなくても /favicon.ico は試せるので、ここでの失敗は続行する。
    let base = match agent.get(page_url).call() {
        Ok(response) => {
            let final_url = Url::parse(response.get_url()).ok();
            let mut html = String::new();
            let _ = response
                .into_reader()
                .take(MAX_HTML_BYTES)
                .read_to_string(&mut html);
            if let Some(base) = &final_url {
                candidates = icon_candidates(&html, base);
            }
            final_url
        }
        Err(_) => None,
    };
    let base = base
        .or_else(|| Url::parse(page_url).ok())
        .ok_or_else(|| "URLを解釈できません".to_string())?;
    // HTMLに書かれていなくても、慣習の場所に置いているサイトが多い。
    for (size, path) in [(APPLE_TOUCH_ICON_SIZE, "/apple-touch-icon.png"), (0, "/favicon.ico")] {
        if let Ok(url) = base.join(path) {
            if !candidates.iter().any(|(_, existing)| *existing == url) {
                candidates.push((size, url));
            }
        }
    }
    // 同じ大きさなら、HTMLに先に書かれたものを優先する。
    candidates.sort_by(|a, b| b.0.cmp(&a.0));

    // 宣言された大きさは当てにならないので、実際に読んだ大きさで一番のものを使う。
    let mut best: Option<image::DynamicImage> = None;
    for (_, candidate) in candidates.into_iter().take(MAX_CANDIDATES) {
        let Some(remaining) = DEADLINE.checked_sub(started.elapsed()) else {
            break;
        };
        let Ok(bytes) = download(&agent, &candidate, remaining) else {
            continue;
        };
        let Ok(image) = image::load_from_memory(&bytes) else {
            continue;
        };
        let side = image.width().min(image.height());
        if side == 0 || best.as_ref().is_some_and(|best| best.width().min(best.height()) >= side) {
            continue;
        }
        best = Some(image);
        if side >= GOOD_ENOUGH {
            break;
        }
    }
    let image = best.ok_or_else(|| "アイコンが見つかりません".to_string())?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("アイコンキャッシュを作成できません: {error}"))?;
    }
    let image = if image.width() > SAVED_SIZE || image.height() > SAVED_SIZE {
        image.resize(SAVED_SIZE, SAVED_SIZE, FilterType::Lanczos3)
    } else {
        image
    };
    image
        .save_with_format(destination, ImageFormat::Png)
        .map_err(|error| format!("アイコンを保存できません: {error}"))
}

/// 全体の上限を超えないよう、1回の待ち時間を残り時間で打ち切る。
fn download(agent: &ureq::Agent, url: &Url, remaining: Duration) -> Result<Vec<u8>, String> {
    let response = agent
        .get(url.as_str())
        .timeout(remaining.min(REQUEST_TIMEOUT))
        .call()
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(MAX_IMAGE_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

/// HTML中の `<link rel="...icon...">` を、宣言された大きさとともに返す。
/// SVGは変換の手段を持たないので除く。
fn icon_candidates(html: &str, base: &Url) -> Vec<(u32, Url)> {
    let mut found: Vec<(u32, Url)> = Vec::new();
    for tag in link_tags(html) {
        let attributes = parse_attributes(tag);
        let attribute = |name: &str| {
            attributes
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.as_str())
        };
        let rel = attribute("rel").unwrap_or("").to_ascii_lowercase();
        let rels: Vec<&str> = rel.split_ascii_whitespace().collect();
        let apple = rels
            .iter()
            .any(|value| value.starts_with("apple-touch-icon"));
        if !apple && !rels.contains(&"icon") {
            continue;
        }
        let Some(href) = attribute("href").filter(|href| !href.trim().is_empty()) else {
            continue;
        };
        let is_svg = attribute("type").is_some_and(|kind| kind.contains("svg"))
            || href.split(['?', '#']).next().unwrap_or("").ends_with(".svg");
        if is_svg {
            continue;
        }
        let Ok(url) = base.join(href.trim()) else {
            continue;
        };
        if !matches!(url.scheme(), "http" | "https") {
            continue;
        }
        let declared = attribute("sizes").and_then(largest_size);
        let size = declared.unwrap_or(if apple { APPLE_TOUCH_ICON_SIZE } else { 0 });
        if !found.iter().any(|(_, existing)| *existing == url) {
            found.push((size, url));
        }
    }
    found
}

/// `sizes="16x16 32x32"` のうち最大の一辺。`any` はSVG向けなので数えない。
fn largest_size(sizes: &str) -> Option<u32> {
    sizes
        .split_ascii_whitespace()
        .filter_map(|size| {
            let (width, height) = size.to_ascii_lowercase().split_once('x').map(|(w, h)| {
                (w.parse::<u32>().ok(), h.parse::<u32>().ok())
            })?;
            Some(width?.max(height?))
        })
        .max()
}

/// `<link` から次の `>` までを取り出す。属性値の中の `>` は考慮しない簡易版で、
/// head内のlinkタグを拾う用途には足りる。
fn link_tags(html: &str) -> Vec<&str> {
    let lower = html.to_ascii_lowercase();
    let mut tags = Vec::new();
    let mut cursor = 0;
    while let Some(offset) = lower[cursor..].find("<link") {
        let start = cursor + offset + "<link".len();
        let Some(length) = lower[start..].find('>') else {
            break;
        };
        tags.push(&html[start..start + length]);
        cursor = start + length;
    }
    tags
}

/// 属性名は小文字にそろえる。値の引用符は外す。
fn parse_attributes(tag: &str) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    let mut chars = tag.char_indices().peekable();
    while let Some(&(start, c)) = chars.peek() {
        if c.is_whitespace() || c == '/' {
            chars.next();
            continue;
        }
        let mut end = start;
        while let Some(&(index, c)) = chars.peek() {
            if c.is_whitespace() || c == '=' || c == '/' {
                break;
            }
            end = index + c.len_utf8();
            chars.next();
        }
        if end == start {
            chars.next();
            continue;
        }
        let name = tag[start..end].to_ascii_lowercase();
        while chars.peek().is_some_and(|&(_, c)| c.is_whitespace()) {
            chars.next();
        }
        let mut value = String::new();
        if chars.peek().is_some_and(|&(_, c)| c == '=') {
            chars.next();
            while chars.peek().is_some_and(|&(_, c)| c.is_whitespace()) {
                chars.next();
            }
            match chars.peek().map(|&(_, c)| c) {
                Some(quote @ ('"' | '\'')) => {
                    chars.next();
                    for (_, c) in chars.by_ref() {
                        if c == quote {
                            break;
                        }
                        value.push(c);
                    }
                }
                _ => {
                    while let Some(&(_, c)) = chars.peek() {
                        if c.is_whitespace() {
                            break;
                        }
                        value.push(c);
                        chars.next();
                    }
                }
            }
        }
        attributes.push((name, value));
    }
    attributes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_the_largest_raster_icon() {
        let html = r#"<html><head>
            <LINK rel="icon" href="/favicon-32.png" sizes="32x32">
            <link rel='icon' type='image/svg+xml' href='/icon.svg'>
            <link href="/apple.png" rel="apple-touch-icon">
            <link rel="shortcut icon" href="https://cdn.example.com/f.ico">
            <link rel="icon" sizes="16x16 192x192" href="icons/big.png" />
            <link rel="stylesheet" href="/style.css">
            <link rel="icon" href="data:image/png;base64,AAAA">
        </head></html>"#;
        let base = Url::parse("https://example.com/docs/page").unwrap();
        let mut found = icon_candidates(html, &base);
        found.sort_by(|a, b| b.0.cmp(&a.0));
        let found: Vec<String> = found.into_iter().map(|(_, url)| String::from(url)).collect();
        assert_eq!(
            found,
            [
                "https://example.com/docs/icons/big.png",
                "https://example.com/apple.png",
                "https://example.com/favicon-32.png",
                "https://cdn.example.com/f.ico",
            ]
        );
    }

    #[test]
    fn reads_unquoted_and_spaced_attributes() {
        let attributes = parse_attributes(r#" rel = icon href=/a.png data-x sizes="48x48""#);
        assert_eq!(
            attributes,
            [
                ("rel".to_string(), "icon".to_string()),
                ("href".to_string(), "/a.png".to_string()),
                ("data-x".to_string(), String::new()),
                ("sizes".to_string(), "48x48".to_string()),
            ]
        );
    }

    #[test]
    fn ignores_sizes_any() {
        assert_eq!(largest_size("any"), None);
        assert_eq!(largest_size("57x57 120X120"), Some(120));
    }

    /// 実際のサイトに通信するので通常は走らせない。`cargo test -- --ignored` で確かめる。
    #[test]
    #[ignore]
    fn fetches_real_sites() {
        let dir = std::env::temp_dir().join(format!("opensesame-favicon-{}", uuid::Uuid::new_v4()));
        for (index, site) in ["https://github.com", "https://www.wikipedia.org", "https://www.apple.com/jp/"]
            .iter()
            .enumerate()
        {
            let path = dir.join(format!("{index}.png"));
            fetch(site, &path).unwrap_or_else(|error| panic!("{site}: {error}"));
            let image = image::open(&path).unwrap();
            println!("{site}: {}x{}", image.width(), image.height());
        }
        fs::remove_dir_all(&dir).unwrap();
    }
}
