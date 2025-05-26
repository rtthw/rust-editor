//! Workspace management



use std::{path::{Path, PathBuf}, sync::RwLock};



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



pub struct Workspace {
    pub info: WorkspaceInfo,
    pub entries: Vec<Entry>,
}

pub fn read_workspace(info: WorkspaceInfo) -> Result<Workspace, std::io::Error> {
    let mut entries = Vec::new();

    for e in std::fs::read_dir(&info.path)? {
        let entry = e?;
        let path = entry.path();
        entries.push(if path.is_dir() {
            Entry::Dir { path }
        } else {
            Entry::File { path }
        });
    }
    entries.sort_by(|a, b| {
        if a.is_dir() == b.is_dir() {
            a.path().file_name().cmp(&b.path().file_name())
        } else if a.is_dir() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(Workspace {
        info,
        entries,
    })
}



#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Entry {
    File {
        path: PathBuf,
    },
    Dir {
        path: PathBuf,
    },
}

impl Entry {
    pub fn is_dir(&self) -> bool {
        matches!(self, Entry::Dir { .. })
    }

    pub fn path(&self) -> &Path {
        match self {
            Entry::File { path } => &path,
            Entry::Dir { path } => &path,
        }
    }
}
