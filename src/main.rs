use std::collections::HashSet;

use clap::{Arg, ArgAction, Command};
use walkdir::WalkDir;

mod gcroots;
mod binary_cache;
mod dep_scan;

fn main() {
    let matches = Command::new("cachecutter")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Astro <astro@spaceboyz.net>")
        .about("Find dead code in .nix files")
        .arg(
            Arg::new("QUIET")
                .action(ArgAction::SetTrue)
                .short('q')
                .long("quiet")
                .help("No progress output"),
        )
        .arg(
            Arg::new("CACHEDIR")
                // .short('d')
                .required(true)
                // .long("cache-directory")
                .help("Cache directory"),
        )
        .arg(
            Arg::new("GCROOTS")
                .num_args(1..)
                .default_value("/nix/var/nix/gcroots")
                .help("Garbage collector roots"),
        )
        .get_matches();

    let quiet = matches.get_flag("QUIET");
    let mut gcroots = gcroots::GcRoots::new();
    for gcroot in matches.get_many::<String>("GCROOTS").expect("GCROOTS") {
        gcroots.scan(gcroot);
    }

    let mut cache = binary_cache::BinaryCache::new(
        matches.get_one::<String>("CACHEDIR")
            .expect("CACHEDIR")
    );
    let mut scanner = dep_scan::DependencyScanner::new();
    for path in gcroots.store_paths.into_iter() {
        scanner.scan(&mut cache, &path);
    }

    let (mut file_size, mut nar_size) = (0usize, 0usize);
    let mut keep_infos = HashSet::with_capacity(scanner.seen.len());
    let mut keep_archives = HashSet::with_capacity(scanner.seen.len());
    for path in &scanner.seen {
        if let Ok(info) = cache.get_info_by_store_path(path) {
            keep_infos.insert(info.path);
            keep_archives.insert(cache.path.join(info.fields.get("URL").unwrap()));
            file_size += info.fields.get("FileSize").unwrap().parse::<usize>().unwrap();
            nar_size += info.fields.get("NarSize").unwrap().parse::<usize>().unwrap();
        }
    }
    dbg!(file_size);
    dbg!(nar_size);

    // dbg!(keep_infos);
    // dbg!(keep_archives);

    for entry in WalkDir::new(&cache.path)
        .min_depth(1)
        .max_depth(1)
    {
        let entry = entry.unwrap();
        if entry.path().to_str().unwrap().ends_with(".narinfo") {
            if ! keep_infos.contains(entry.path()) {
                println!("rm {}", entry.path().display());
            }
        }
    }

    for entry in WalkDir::new(cache.path.join("nar"))
        .min_depth(1)
        .max_depth(1)
    {
        let entry = entry.unwrap();
        if ! keep_archives.contains(entry.path()) {
            println!("rm {}", entry.path().display());
        }
    }
}
