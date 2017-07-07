use gtk::*;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs::{self};
use std::fs::metadata;
use std::ops::Deref;
use std::path::{Path, PathBuf};

fn path_sorter(a: &PathBuf, b: &PathBuf) -> Ordering {
    if let Some(a_os_str) = a.deref().file_name() {
        if let Some(b_os_str) = b.deref().file_name() {
            return a_os_str.cmp(&b_os_str);
        }
    }
    Ordering::Equal
}

fn sort_string_paths(paths: &HashSet<String>) -> Vec<PathBuf> {
    let mut paths_vec = Vec::new();
    for path_str in paths.iter() {
        paths_vec.push(PathBuf::from(path_str));
    }
    paths_vec.sort_by(path_sorter);
    paths_vec
}

fn update_project_buttons(ui: &::utils::UI, prefs: &::utils::Prefs) {
    if let Some(path_str) = ::utils::get_selected_path(ui) {
        let is_project = prefs.projects.contains(&path_str);
        let path = Path::new(&path_str);
        ui.rename_button.set_sensitive(!metadata(path).map(|m| m.is_dir()).unwrap_or(false));
        ui.remove_button.set_sensitive(!metadata(path).map(|m| m.is_dir()).unwrap_or(false) || is_project);
    } else {
        ui.rename_button.set_sensitive(false);
        ui.remove_button.set_sensitive(false);
    }
}

fn add_node(ui: &::utils::UI, node: &Path, parent: Option<&TreeIter>) {
    if let Some(full_path_str) = node.to_str() {
        if let Some(leaf_os_str) = node.file_name() {
            if let Some(leaf_str) = leaf_os_str.to_str() {
                if !leaf_str.starts_with(".") {
                    let iter = ui.tree_store.append(parent);
                    ui.tree_store.set(&iter, &[0, 1], &[&String::from(leaf_str), &String::from(full_path_str)]);

                    if metadata(node).map(|m| m.is_dir()).unwrap_or(false) {
                        match fs::read_dir(node) {
                            Ok(child_iter) => {
                                let mut child_vec = Vec::new();
                                for child in child_iter {
                                    if let Ok(dir_entry) = child {
                                        child_vec.push(dir_entry.path());
                                    }
                                }
                                child_vec.sort_by(path_sorter);
                                for child in child_vec.iter() {
                                    add_node(ui, child.deref(), Some(&iter));
                                }
                            },
                            Err(e) => println!("Error updating tree: {}", e)
                        }
                    }
                }
            }
        }
    }
}

fn expand_nodes(ui: &::utils::UI, prefs: &::utils::Prefs, parent: Option<&TreeIter>) {
    if let Some(mut iter) = ui.tree_store.iter_children(parent) {
        loop {
            if let Some(path_str) = ::utils::iter_to_str(ui, &iter) {
                if let Some(selection_str) = prefs.selection.clone() {
                    if Path::new(&path_str) == Path::new(&selection_str) {
                        if let Some(path) = ui.tree_store.get_path(&iter) {
                            ui.tree.set_cursor(&path, None, false);
                        }
                    }
                }

                if prefs.expansions.contains(&path_str) {
                    if let Some(path) = ui.tree_store.get_path(&iter) {
                        ui.tree.expand_row(&path, false);
                        expand_nodes(ui, prefs, Some(&iter));
                    }
                }
            }

            if !ui.tree_store.iter_next(&mut iter) {
                break;
            }
        }
    }
}

pub fn update_project_tree(ui: &::utils::UI, prefs: &::utils::Prefs) {
    ui.tree_store.clear();

    for path in sort_string_paths(&prefs.projects).iter() {
        add_node(ui, path, None);
    }

    expand_nodes(ui, prefs, None);

    update_project_buttons(ui, prefs);
}
