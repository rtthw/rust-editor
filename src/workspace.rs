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



pub fn read_workspace(info: WorkspaceInfo) -> Result<Workspace, std::io::Error> {
    let entries = read_entries(&info.path, 0, 1)?;

    Ok(Workspace {
        info,
        entries,
    })
}

fn read_entries(path: &Path, level: usize, limit: usize) -> Result<Vec<Entry>, std::io::Error> {
    let mut entries = vec![];
    for e in std::fs::read_dir(path)? {
        let entry = e?;
        let path = entry.path();
        entries.push(if path.is_dir() {
            let children = if level < limit {
                read_entries(&path, level + 1, limit)?
            } else {
                vec![Entry::ContinuationMarker { parent: path.clone(), level }]
            };
            Entry::Dir { path, children, level }
        } else {
            Entry::File { path, level }
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

    Ok(entries)
}



pub struct Workspace {
    pub info: WorkspaceInfo,
    pub entries: Vec<Entry>,
}

impl Workspace {
    pub fn entries(&self) -> impl Iterator<Item = EntryView> {
        self.entries.iter().flat_map(|entry| {
            if let Entry::Dir { children, .. } = &entry {
                let mut entries = Vec::with_capacity(children.len() + 1);
                entries.push(EntryView {
                    name: entry.name(),
                    path: entry.path(),
                    level: entry.level(),
                });
                entries.extend(children.iter().map(|e| EntryView {
                    name: e.name(),
                    path: e.path(),
                    level: e.level(),
                }));

                entries.into_iter()
            } else {
                vec![EntryView {
                    name: entry.name(),
                    path: entry.path(),
                    level: entry.level(),
                }].into_iter()
            }
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Entry {
    File {
        path: PathBuf,
        level: usize,
    },
    Dir {
        path: PathBuf,
        children: Vec<Entry>,
        level: usize,
    },
    ContinuationMarker {
        parent: PathBuf,
        level: usize,
    },
}

impl Entry {
    pub fn is_dir(&self) -> bool {
        matches!(self, Entry::Dir { .. })
    }

    pub fn path(&self) -> &Path {
        match self {
            Entry::File { path, .. } => &path,
            Entry::Dir { path, .. } => &path,
            Entry::ContinuationMarker { parent, .. } => &parent,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Entry::ContinuationMarker { .. } => "...",
            other => {
                other.path().file_name().and_then(|os_str| os_str.to_str()).unwrap()
            }
        }
    }

    pub fn level(&self) -> usize {
        match self {
            Entry::File { level, .. } => *level,
            Entry::Dir { level, .. } => *level,
            Entry::ContinuationMarker { level, .. } => *level,
        }
    }
}

pub struct EntryView<'a> {
    pub name: &'a str,
    pub path: &'a Path,
    pub level: usize,
}
