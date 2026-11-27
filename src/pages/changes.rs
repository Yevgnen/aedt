//! The changes panel: what the conversation's folder holds against Git, beside
//! the conversation while the reader asks for it.

use std::path::Path;
use std::process::Command;
use std::rc::Rc;

use aedt_store as store;
use aedt_ui::{button, diff, empty, icon, motion, sizes};
use nsh_base::theme;
use nsh_core::prelude::*;

/// One file the working tree changed against `HEAD`, as the panel lists it.
#[derive(Clone, Default, PartialEq)]
struct Changed {
    /// The file's path from the repository's root, as Git names it.
    path: SharedString,
    added: usize,
    removed: usize,
    patch: SharedString,
}

/// What the panel shows, and which folder it read.
#[derive(Clone, Default, PartialEq)]
struct Snapshot {
    root: Option<SharedString>,
    files: Vec<Changed>,
    loading: bool,
}

/// The mark beside the conversation's Zed mark: shows the panel, or hides it
/// again. It shows only where the conversation works in a folder.
pub(crate) fn toggle(
    workspace: impl Fn() -> Option<SharedString> + Copy + 'static,
    open: Signal<bool>,
) -> impl IntoElement {
    when(
        move || workspace().is_some(),
        move || {
            when(
                move || open.get(),
                move || {
                    button::ghost_icon(icon::diff(14.0), "Hide the changes")
                        .on_press(move || motion::set_panel(open, false))
                },
            )
            .otherwise(move || {
                button::ghost_icon(icon::diff(14.0), "Show the changes")
                    .on_press(move || motion::set_panel(open, true))
            })
        },
    )
}

/// The changes panel, at the conversation's right half: the folder's files that
/// differ from `HEAD`, each a foldable diff card under its Git path.
pub(crate) fn panel(session_id: u32) -> View {
    let root = computed(move || store::session_path(session_id));
    let snapshot = signal(Snapshot::default());
    let reload = signal(0_u32);
    // Which cards the reader folded away; a file absent here shows its diff.
    let folded = signal(Vec::<SharedString>::new());

    effect(move || {
        let root = root.get();
        // A press on the refresh mark asks for the same folder again.
        let _ = reload.get();
        load(snapshot, root);
    });

    view()
        .w(move || Length::from(percentage(0.5)))
        .min_w(px(0.0))
        .h_full()
        .flex_none()
        .flex_col()
        .bg(move || theme().background)
        .border_l_1()
        .border_l_color(move || theme().border)
        .child(top(reload))
        .child(
            when(
                move || {
                    let root = root.get();
                    snapshot.with(|snapshot| snapshot.loading || snapshot.root != root)
                },
                || view().flex_1(),
            )
            .otherwise(move || {
                when(
                    move || snapshot.with(|snapshot| snapshot.files.is_empty()),
                    || empty::empty("No changes"),
                )
                .otherwise(move || list(snapshot, folded))
            }),
        )
}

/// The panel's top row: its title, and the mark that reads the folder again.
fn top(reload: Signal<u32>) -> impl IntoElement {
    view()
        .h(px(sizes::TOPBAR))
        .flex_row()
        .items_center()
        .gap_2()
        .px_3()
        .border_b_1()
        .border_b_color(move || theme().border)
        .child(
            text("Changes")
                .flex_1()
                .min_w(px(0.0))
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            button::ghost_icon(icon::refresh(14.0), "Refresh the changes")
                .on_press(move || reload.update(|reload| *reload = reload.wrapping_add(1))),
        )
}

/// The changed files, each card under the one before it.
fn list(snapshot: Signal<Snapshot>, folded: Signal<Vec<SharedString>>) -> impl IntoElement {
    scroll().flex_1().min_h(px(0.0)).scrollbar(false).child(
        view().w_full().flex_col().gap_2().p_3().child(each(
            move || snapshot.with(|snapshot| snapshot.files.clone()),
            |file| file.path.clone(),
            move |file, _| card(file, folded),
        )),
    )
}

/// One file's diff, folded by the reader's press on its header.
fn card(file: ReadSignal<Changed>, folded: Signal<Vec<SharedString>>) -> impl IntoElement {
    let path = Rc::new(file.with_untracked(|file| file.path.clone()));
    let shown = {
        let path = Rc::clone(&path);
        move || !folded.with(|folded| folded.iter().any(|folded| folded.as_str() == path.as_str()))
    };
    let toggle = {
        let path = Rc::clone(&path);
        move || {
            folded.update(|folded| {
                match folded
                    .iter()
                    .position(|folded| folded.as_str() == path.as_str())
                {
                    Some(at) => {
                        folded.remove(at);
                    }
                    None => folded.push((*path).clone()),
                }
            });
        }
    };

    diff::foldable_card(
        move || {
            file.with(|file| diff::Change {
                path: file.path.clone(),
                added: file.added,
                removed: file.removed,
                patch: file.patch.clone(),
            })
        },
        shown,
        toggle,
    )
}

/// Reads `root` on the blocking threads and writes what it found, keeping the
/// files already shown while the same folder is read again.
fn load(snapshot: Signal<Snapshot>, root: Option<SharedString>) {
    let Some(root) = root else {
        snapshot.set(Snapshot::default());
        return;
    };
    let previous = snapshot.with_untracked(|previous| {
        (previous.root.as_ref() == Some(&root)).then(|| previous.files.clone())
    });
    let loading = previous.is_none();
    snapshot.set(Snapshot {
        root: Some(root.clone()),
        files: previous.unwrap_or_default(),
        loading,
    });

    let read_root = root.clone();
    spawn(async move {
        let files = match blocking(move || read_changes(&read_root)).await {
            Ok(Ok(files)) => files,
            Ok(Err(reason)) => {
                eprintln!("changes: failed to read the repository: {reason}");
                Vec::new()
            }
            Err(error) => {
                eprintln!("changes: the Git reader stopped: {error}");
                Vec::new()
            }
        };
        snapshot.set(Snapshot {
            root: Some(root),
            files,
            loading: false,
        });
    });
}

/// Reads every change in the repository `root` against `HEAD`, each file's
/// patch as Git writes it. A folder Git does not hold is an error.
fn read_changes(root: &str) -> Result<Vec<Changed>, String> {
    let status = git(root, &["status", "--porcelain=v1", "-z"])?;
    let head = git(root, &["rev-parse", "--verify", "HEAD"]).is_ok();
    let mut files = Vec::new();
    let mut entries = status.split('\0');
    while let Some(entry) = entries.next() {
        let Some((code, path)) = entry.split_at_checked(2) else {
            continue;
        };
        let Some(path) = path.strip_prefix(' ') else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        // A rename or copy names the path it came from in the record after it,
        // which the card does not show.
        if code.contains('R') || code.contains('C') {
            entries.next();
        }

        let patch = if code.starts_with("??") {
            untracked_diff(root, path)?
        } else if head {
            git(root, &["diff", "HEAD", "--no-color", "--", path])?
        } else {
            git(root, &["diff", "--cached", "--no-color", "--", path])?
        };
        let (added, removed) = count_lines(&patch);
        files.push(Changed {
            path: path.to_owned().into(),
            added,
            removed,
            patch: patch.into(),
        });
    }

    Ok(files)
}

/// Runs Git in `root` with `args` and returns what it wrote, or why it failed.
fn git(root: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| format!("git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(format!("git {}: {}", args.join(" "), stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The patch of a file Git does not track yet, as if it were added to nothing.
fn untracked_diff(root: &str, path: &str) -> Result<String, String> {
    let file = Path::new(root).join(path);
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--no-index", "--no-color", "--", "/dev/null"])
        .arg(&file)
        .output()
        .map_err(|error| format!("git diff --no-index: {error}"))?;
    // A difference exits 1 and a file with nothing in it exits 0: both are
    // patches the panel can show, empty or not.
    if matches!(output.status.code(), Some(0 | 1)) {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);

    Err(format!("git diff --no-index: {}", stderr.trim()))
}

/// How many lines `patch` adds and removes, its file headers left out.
fn count_lines(patch: &str) -> (usize, usize) {
    let mut added = 0;
    let mut removed = 0;
    for line in patch.lines() {
        if line.starts_with("+++ ") || line.starts_with("--- ") {
            continue;
        }
        match line.as_bytes().first() {
            Some(b'+') => added += 1,
            Some(b'-') => removed += 1,
            _ => {}
        }
    }

    (added, removed)
}
