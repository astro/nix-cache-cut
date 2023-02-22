use std::{
    collections::HashSet,
    fs,
};

use clap::{Arg, ArgAction, Command};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle, HumanBytes};
use walkdir::WalkDir;

mod gcroots;
mod binary_cache;
mod dep_scan;

fn main() {
    // Define command-line arguments
    let matches = Command::new("cachecutter")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Astro <astro@spaceboyz.net>")
        .about("Find dead code in .nix files")
        .arg(
            Arg::new("DRYRUN")
                .action(ArgAction::SetTrue)
                .short('n')
                .long("dry-run")
                .help("Do not actually delete files")
        )
        .arg(
            Arg::new("CACHEDIR")
                .required(true)
                .help("Cache directory")
        )
        .arg(
            Arg::new("GCROOTS")
                .num_args(1..)
                .default_value("/nix/var/nix/gcroots")
                .help("Garbage collector roots")
        )
        .get_matches();

    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {wide_bar} {msg} {spinner}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let progress = MultiProgress::new();
    let progress_gcroots = progress.add(ProgressBar::new(0));
    progress_gcroots.set_prefix("Scanning GCROOTS");
    progress_gcroots.set_style(spinner_style.clone());
    let progress_scanner = progress.add(ProgressBar::new(1));
    progress_scanner.set_prefix("Scanning dependencies");
    progress_scanner.set_style(spinner_style.clone());
    progress_scanner.tick();
    let progress_keep = progress.add(ProgressBar::new(1));
    progress_keep.set_prefix("Keeping archives");
    progress_keep.set_style(spinner_style.clone());
    progress_keep.tick();
    let progress_rm_narinfo = progress.add(ProgressBar::new(1));
    progress_rm_narinfo.set_prefix("Deleting .narinfo files");
    progress_rm_narinfo.set_style(spinner_style.clone());
    progress_rm_narinfo.tick();
    let progress_rm_nar = progress.add(ProgressBar::new(1));
    progress_rm_nar.set_prefix("Deleting .nar files");
    progress_rm_nar.set_style(spinner_style.clone());
    progress_rm_nar.tick();

    // Scan garbage-collector roots
    let mut gcroots = gcroots::GcRoots::new();
    for gcroot in matches.get_many::<String>("GCROOTS").expect("GCROOTS") {
        gcroots.scan(gcroot, &progress_gcroots);
    }

    // Construct cache abstraction
    let mut cache = binary_cache::BinaryCache::new(
        matches.get_one::<String>("CACHEDIR")
            .expect("CACHEDIR")
    );
    // Scan gcroots dependencies
    let mut scanner = dep_scan::DependencyScanner::new();
    gcroots.store_paths.into_iter()
        .for_each(|path| scanner.enqueue(path));
    progress_scanner.reset_eta();
    let scanner_seen = scanner.scan(&mut cache, &progress_scanner);

    // Statistics
    let (mut file_size, mut nar_size) = (0usize, 0usize);
    // Set of files to keep
    progress_keep.set_length(scanner_seen.len() as u64);
    let mut keep_infos = HashSet::with_capacity(scanner_seen.len());
    let mut keep_archives = HashSet::with_capacity(scanner_seen.len());
    let cache_path = cache.path.clone();
    scanner_seen.into_iter()
        .filter_map(|path| {
            let result = cache.get_info_by_store_path(&path)
                .ok();
            progress_keep.inc(1);
            result
        })
        .for_each(|info| {
            keep_infos.insert(info.path);
            keep_archives.insert(cache_path.join(info.fields.get("URL").unwrap()));
            file_size += info.fields.get("FileSize").unwrap().parse::<usize>().unwrap();
            nar_size += info.fields.get("NarSize").unwrap().parse::<usize>().unwrap();
            progress_keep.set_message(
                format!("{} in {} archive files", HumanBytes(nar_size as u64), HumanBytes(file_size as u64))
            );
        });

    let dry_run = matches.get_flag("DRYRUN");
    let mut rm_narinfo_size = 0;
    let mut rm_nar_size = 0;

    progress_rm_narinfo.set_length(0);
    for entry in WalkDir::new(&cache.path)
        .min_depth(1)
        .max_depth(1)
    {
        progress_rm_narinfo.inc_length(1);

        let entry = entry.unwrap();
        if entry.path().to_str().unwrap().ends_with(".narinfo") && ! keep_infos.contains(entry.path()) {
            if let Ok(meta) = fs::metadata(entry.path()) {
                rm_narinfo_size += meta.len();
                progress_rm_narinfo.set_message(format!("{}", HumanBytes(rm_narinfo_size)));
            }

            if ! dry_run {
                match fs::remove_file(entry.path()) {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Cannot remove {}: {}", entry.path().display(), e);
                    }
                }
            }
        }

        progress_rm_narinfo.inc(1);
    }

    progress_rm_nar.set_length(0);
    for entry in WalkDir::new(cache.path.join("nar"))
        .min_depth(1)
        .max_depth(1)
    {
        progress_rm_nar.inc_length(1);

        let entry = entry.unwrap();
        if ! keep_archives.contains(entry.path()) {
            if let Ok(meta) = fs::metadata(entry.path()) {
                rm_nar_size += meta.len();
                progress_rm_nar.set_message(format!("{}", HumanBytes(rm_nar_size)));
            }

            if ! dry_run {
                match fs::remove_file(entry.path()) {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Cannot remove {}: {}", entry.path().display(), e);
                    }
                }
            }
        }

        progress_rm_nar.inc(1);
    }

    progress.clear().unwrap();
}
