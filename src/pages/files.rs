//! The files panel: the folder a conversation works in, its directories opened
//! one at a time.

use std::collections::HashMap;

use aedt_ui::{button, empty, icon, sizes};
use nsh_base::theme;
use nsh_core::prelude::*;

/// One entry a folder holds, as the panel lists it.
#[derive(Clone, PartialEq, Eq)]
struct Entry {
    name: SharedString,
    path: SharedString,
    dir: bool,
}

/// One row of the tree: an entry, and how deep under the folder it sits.
#[derive(Clone, PartialEq, Eq)]
struct Row {
    name: SharedString,
    path: SharedString,
    dir: bool,
    depth: usize,
}

/// The files panel: the folder the conversation works in, with the toggle that
/// hides the panel at the window's right end. The tree follows `root`, the
/// folder of the conversation on show: it starts over when the reader moves to
/// another folder, and keeps its opened folders while they stay in one.
pub(crate) fn panel(root: Memo<Option<SharedString>>, open: Signal<bool>) -> View {
    // What each folder lists, by its path: the ones the reader opened so far.
    let listings = signal(HashMap::<SharedString, Vec<Entry>>::new());
    // The folders the reader opened.
    let opened = signal(Vec::<SharedString>::new());
    effect(move || {
        let root = root.get();
        listings.update(|listings| {
            listings.clear();
            if let Some(root) = &root {
                listings.insert(root.clone(), read(root));
            }
        });
        opened.set(Vec::new());
    });

    view()
        .w(px(sizes::SIDEBAR))
        .h_full()
        .flex_none()
        .bg(move || theme().sidebar)
        .border_l_1()
        .border_l_color(move || theme().sidebar_border)
        .child(top(open))
        .child(
            when(
                move || root.with(|root| root.is_some()),
                move || tree(root, opened, listings).into_element(),
            )
            .otherwise(|| empty::empty("No folder").into_element()),
        )
}

/// The panel's top row: the drag band every column starts with, and the toggle
/// that shuts the panel, on the line the app's other marks stand on at the
/// window's right edge.
fn top(open: Signal<bool>) -> impl IntoElement {
    view()
        .h(px(sizes::TOPBAR))
        .flex_row()
        .items_center()
        .justify_end()
        .pr(px(24.0))
        .window_drag(true)
        .child(
            button::ghost_icon(icon::panel_right(14.0), "Hide the files")
                .on_press(move || open.set(false)),
        )
}

/// The folder's rows, each opened folder's under it.
fn tree(
    root: Memo<Option<SharedString>>,
    opened: Signal<Vec<SharedString>>,
    listings: Signal<HashMap<SharedString, Vec<Entry>>>,
) -> impl IntoElement {
    let rows = computed(move || {
        let mut rows = Vec::new();
        if let Some(root) = root.get() {
            listings.with(|listings| walk(listings, &opened.get(), &root, 0, &mut rows));
        }

        rows
    });

    scroll().flex_1().min_h(px(0.0)).child(each(
        move || rows.get(),
        |row| row.path.clone(),
        move |row, _| entry_row(row, opened, listings),
    ))
}

/// Adds `folder`'s entries to `rows`, and those of the folders the reader
/// opened under it.
fn walk(
    listings: &HashMap<SharedString, Vec<Entry>>,
    opened: &[SharedString],
    folder: &SharedString,
    depth: usize,
    rows: &mut Vec<Row>,
) {
    let Some(entries) = listings.get(folder) else {
        return;
    };
    for entry in entries {
        rows.push(Row {
            name: entry.name.clone(),
            path: entry.path.clone(),
            dir: entry.dir,
            depth,
        });
        if entry.dir && opened.iter().any(|path| path == &entry.path) {
            walk(listings, opened, &entry.path, depth + 1, rows);
        }
    }
}

/// One row of the tree: its folded mark, its kind's mark, and its name. A
/// folder's row opens or shuts it.
fn entry_row(
    row: ReadSignal<Row>,
    opened: Signal<Vec<SharedString>>,
    listings: Signal<HashMap<SharedString, Vec<Entry>>>,
) -> impl IntoElement {
    let Row {
        name,
        path,
        dir,
        depth,
    } = row.with_untracked(|row| row.clone());

    let is_open = {
        let path = path.clone();
        move || opened.with(|opened| opened.iter().any(|opened| opened == &path))
    };
    let toggle = {
        let path = path.clone();
        move || {
            if opened.with(|opened| opened.iter().any(|opened| opened == &path)) {
                opened.update(|opened| opened.retain(|opened| opened != &path));
                return;
            }
            if !listings.with(|listings| listings.contains_key(&path)) {
                let entries = read(&path);
                listings.update(|listings| {
                    listings.insert(path.clone(), entries);
                });
            }
            opened.update(|opened| opened.push(path.clone()));
        }
    };

    let fold = if dir {
        view()
            .flex_none()
            .text_color(move || theme().muted_foreground)
            .child(
                when(is_open, || icon::chevron_down(12.0)).otherwise(|| icon::chevron_right(12.0)),
            )
            .into_element()
    } else {
        view().w(px(12.0)).flex_none().into_element()
    };
    let mark = if dir {
        icon::folder(14.0).into_element()
    } else {
        icon::file(14.0).into_element()
    };

    view()
        .w_full()
        .min_h(px(sizes::LIST_ROW))
        .flex_row()
        .items_center()
        .gap_1()
        .pl(px(sizes::SIDEBAR_INSET + depth as f32 * 12.0))
        .pr_2()
        .rounded(move || theme().radius)
        .text_color(move || theme().sidebar_foreground)
        .hover(|s| s.bg(move || theme().muted))
        .on_press(move || {
            if dir {
                toggle();
            }
        })
        .child(fold)
        .child(mark)
        .child(
            text(name)
                .flex_1()
                .min_w(px(0.0))
                .number_of_lines(1)
                .text_size(px(sizes::TEXT_COMPACT)),
        )
}

/// What `folder` holds: its folders first, then its files, each in name order,
/// the hidden ones left out. A folder that cannot be read lists nothing.
fn read(folder: &str) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut entries: Vec<Entry> = read
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        .map(|entry| {
            let path = entry.path();
            let dir = std::fs::metadata(&path)
                .map(|meta| meta.is_dir())
                .unwrap_or(false);

            Entry {
                name: entry.file_name().to_string_lossy().into_owned().into(),
                path: path.to_string_lossy().into_owned().into(),
                dir,
            }
        })
        .collect();
    entries.sort_by_cached_key(|entry| (!entry.dir, entry.name.to_lowercase()));

    entries
}
