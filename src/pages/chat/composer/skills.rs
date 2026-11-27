//! The skills the composer's slash menu lists: the ones under the reader's
//! `~/.agents/skills`, and the ones a conversation's workspace keeps in its
//! `.agents/skills`. A workspace skill replaces a global one with the same id,
//! as an agent's own catalog does.

use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

/// One skill's command name and where it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Skill {
    pub id: String,
    pub source: Source,
}

/// The two places a skill can come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Source {
    Global,
    Project,
}

impl Source {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Project => "project",
        }
    }
}

/// The skills `workspace` adds to the reader's own, sorted by id.
pub(super) fn load(workspace: Option<&Path>) -> Vec<Skill> {
    let mut found = BTreeMap::new();
    if let Some(home) = home_dir() {
        read_skills(
            &home.join(".agents").join("skills"),
            Source::Global,
            &mut found,
        );
    }
    if let Some(workspace) = workspace {
        read_skills(
            &workspace.join(".agents").join("skills"),
            Source::Project,
            &mut found,
        );
    }
    found.into_values().collect()
}

/// Whether `skill` matches the lowercase `query` anywhere in its id, not only
/// at the start.
pub(super) fn matches(skill: &Skill, query: &str) -> bool {
    query.is_empty() || skill.id.to_lowercase().contains(query)
}

fn read_skills(dir: &Path, source: Source, found: &mut BTreeMap<String, Skill>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(folder) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Ok(text) = fs::read_to_string(path.join("SKILL.md")) else {
            continue;
        };
        let id = frontmatter_name(&text).unwrap_or(folder);
        found.insert(id.clone(), Skill { id, source });
    }
}

/// A skill's `name:` from its frontmatter, when it has one. The menu's id is
/// the folder's name otherwise.
fn frontmatter_name(text: &str) -> Option<String> {
    let mut lines = text.lines();
    if lines.next()?.trim_end() != "---" {
        return None;
    }
    for line in lines {
        let line = line.trim_end();
        if line.trim() == "---" {
            break;
        }
        let Some(value) = line.strip_prefix("name:") else {
            continue;
        };
        let value = unquote(value.trim());
        return (!value.is_empty()).then(|| value.to_owned());
    }
    None
}

fn unquote(value: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = value
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
        {
            return inner;
        }
    }
    value
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}
