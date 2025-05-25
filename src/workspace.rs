//! Workspace management



use std::{path::PathBuf, sync::RwLock};



/// Finds the current workspace folder.
///
/// This function starts searching the filesystem upward from the CWD, and returns the first
/// directory that contains an indicator of a workspace (for now, just a `.git` subdirectory).
///
/// If no workspace was found, returns (CWD, true).
/// Otherwise (workspace, false) is returned
pub fn find_workspace() -> (PathBuf, bool) {
    let current_dir = cwd();
    for ancestor in current_dir.ancestors() {
        if ancestor.join(".git").exists()
            // || ancestor.join(".svn").exists()
        {
            return (ancestor.to_owned(), false);
        }
    }

    (current_dir, true)
}

static CWD: RwLock<Option<PathBuf>> = RwLock::new(None);

pub fn cwd() -> PathBuf {
    if let Some(path) = &*CWD.read().unwrap() {
        return path.clone();
    }

    let mut cwd = std::env::current_dir()
        .expect("couldn't determine current working directory");

    let pwd = std::env::var_os("PWD");

    if let Some(pwd) = pwd.map(PathBuf::from) {
        if pwd.canonicalize().ok().as_ref() == Some(&cwd) {
            cwd = pwd;
        }
    }
    let mut dst = CWD.write().unwrap();
    *dst = Some(cwd.clone());

    cwd
}
