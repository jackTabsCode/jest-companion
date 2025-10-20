use crate::config::Config;
use std::path::{Path, PathBuf};

pub fn resolve_path(config: &Config, virtual_path: &str, project_root: &Path) -> Option<PathBuf> {
    let p = normalize_virtual(virtual_path);

    // Try as-is
    if let Some((base, rem)) = find_longest_prefix(config, &p) {
        let mut out = project_root.join(base);
        if !rem.is_empty() {
            out = out.join(rem);
        }
        return Some(out);
    }

    // Try skipping the first segment
    let mut it = p.split('/');
    let _first = it.next()?;

    let without_first = it.collect::<Vec<_>>().join("/");
    if without_first.is_empty() {
        return None;
    }

    if let Some((base, rem)) = find_longest_prefix(config, &without_first) {
        let mut out = project_root.join(base);
        if !rem.is_empty() {
            out = out.join(rem);
        }

        return Some(out);
    }

    None
}

fn normalize_virtual(p: &str) -> String {
    p.replace('\\', "/").trim_start_matches('/').to_string()
}

fn find_longest_prefix<'a>(config: &'a Config, path: &'a str) -> Option<(&'a PathBuf, &'a str)> {
    let mut best: Option<(&PathBuf, &str, usize)> = None;

    for (k, base) in &config.projects {
        let k = k.trim_start_matches('/');

        if path == k {
            let rem = "";
            if best.is_none_or(|(_, _, bl)| k.len() > bl) {
                best = Some((base, rem, k.len()));
            }
        } else if let Some(rest) = path.strip_prefix(k).and_then(|s| s.strip_prefix('/'))
            && best.is_none_or(|(_, _, bl)| k.len() > bl)
        {
            best = Some((base, rest, k.len()));
        }
    }

    best.map(|(b, r, _)| (b, r))
}
