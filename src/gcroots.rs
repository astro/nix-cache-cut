use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
};
use indicatif::ProgressBar;
use walkdir::WalkDir;

pub struct GcRoots {
    seen: HashSet<PathBuf>,
    pub store_paths: HashSet<PathBuf>,
}

impl GcRoots {
    pub fn new() -> Self {
        GcRoots {
            seen: HashSet::new(),
            store_paths: HashSet::new(),
        }
    }

    pub fn scan<P: Into<PathBuf>>(&mut self, gcroot: P, progress: &ProgressBar) {
        let Ok(gcroot) = fs::canonicalize(gcroot.into()) else {
            return;
        };

        if self.seen.contains(&gcroot) {
            return;
        }

        progress.inc_length(1);
        for entry in WalkDir::new(gcroot) {
            match entry {
                Ok(entry) => {
                    self.seen.insert(entry.path().into());

                    if entry.path_is_symlink() {
                        let target = fs::read_link(entry.path())
                            .expect("read_link");
                        if let Ok(target) = fs::canonicalize(entry.path().join(target)) {
                            if target.starts_with("/nix/store") {
                                self.add_store_path(target);
                            } else {
                                self.scan(target, progress);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{e}");
                }
            }
        }
        progress.inc(1);
    }

    fn add_store_path(&mut self, mut path: PathBuf) {
        let components = path.components()
            .count();
        for _ in 4..components {
            path.pop();
        }
        self.store_paths.insert(path);
    }
}
