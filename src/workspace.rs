//! Workspace management



use std::{path::PathBuf, sync::RwLock};



#[derive(Clone, Debug)]
pub struct WorkspaceInfo {
    /// The path to this workspace.
    pub path: PathBuf,
    /// Whether this workspace has version control.
    pub has_vc: bool,
}


/// Finds the current workspace.
pub fn find_workspace() -> WorkspaceInfo {
    let mut path = cwd();
    let mut has_vc = false;

    // Find the first ancestor that has version control, and use it as the workspace.
    for ancestor in cwd().ancestors() {
        if ancestor.join(".git").exists()
            // || ancestor.join(".svn").exists()
        {
            path = ancestor.to_path_buf();
            has_vc = true;
        }
    }

    WorkspaceInfo {
        path,
        has_vc,
    }
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
