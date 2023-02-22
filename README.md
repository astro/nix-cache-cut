# nix-cache-cut

Yet another garbage collector for Nix binary caches.

This one runs with a list of GC roots just like your ordinary
`nix-collect-garbage`. It does not operate time-based to expire old
files like [lheckemann's
cache-gc](https://github.com/lheckemann/cache-gc).


## Usage

```console
$ nix run github:astro/nix-cache-cut -- --help
Trim Nix binary caches according to GC roots

Usage: nix-cache-cut [OPTIONS] <CACHEDIR> [GCROOTS]...

Arguments:
  <CACHEDIR>    Cache directory
  [GCROOTS]...  Garbage collector roots [default: /nix/var/nix/gcroots]

Options:
  -n, --dry-run  Do not actually delete files
  -h, --help     Print help
  -V, --version  Print version
```
