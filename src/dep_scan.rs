use std::{collections::HashSet, path::{Path, PathBuf}};

use crate::binary_cache::BinaryCache;

pub struct DependencyScanner {
    pub seen: HashSet<PathBuf>,
}

impl DependencyScanner {
    pub fn new() -> Self {
        DependencyScanner {
            seen: HashSet::new(),
        }
    }

    // TODO: should work a queue, not recurse
    pub fn scan(&mut self, cache: &mut BinaryCache, path: &Path) {
        if self.seen.contains(path) {
            return;
        }
        self.seen.insert(path.into());

        match cache.get_info_by_store_path(path) {
            Ok(info) => {
                dbg!(&info);
                for reference in info.references() {
                    let path = PathBuf::from("/nix/store").join(reference);
                    dbg!(&path);
                    self.scan(cache, &path);
                }
                if let Some(deriver) = info.deriver() {
                    let path = PathBuf::from("/nix/store").join(deriver);
                    dbg!(&path);
                    self.scan(cache, &path);
                }
            }
            Err(e) => {
                eprintln!("{path:?}: {e}");
            }
        }
    }
}
