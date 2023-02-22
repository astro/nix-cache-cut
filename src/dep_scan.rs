use std::{collections::{HashSet, VecDeque}, path::PathBuf};

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

    pub fn scan(&mut self, cache: &mut BinaryCache, path: PathBuf) {
        let mut queue = VecDeque::with_capacity(1);
        queue.push_back(path);

        while let Some(path) = queue.pop_front() {
            if self.seen.contains(&path) {
                // skip if scanned before
                continue;
            }
            self.seen.insert(path.clone());

            match cache.get_info_by_store_path(&path) {
                Ok(info) => {
                    dbg!(&info);
                    for reference in info.references() {
                        let path = PathBuf::from("/nix/store").join(reference);
                        queue.push_back(path);
                    }
                    if let Some(deriver) = info.deriver() {
                        let path = PathBuf::from("/nix/store").join(deriver);
                        queue.push_back(path);
                    }
                }
                Err(e) => {
                    eprintln!("Cannot scan cache for {path:?}: {e}");
                }
            }
        }
    }
}
