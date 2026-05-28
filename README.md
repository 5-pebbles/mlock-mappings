# mlock-mappings 🍬

Lock a process's file-backed pages into RAM so it survives having its backing storage overwritten.

Parses `/proc/<pid>/maps` and opens each file-backed region through `/proc/<pid>/map_files/` — which resolves to the actual backing file the kernel is using, not whatever the path on disk points to now. This means upgraded-in-place binaries are handled correctly. Each region is mmapped and mlocked, pinning the shared page cache pages in physical memory. 🍭

## 🍫 Usage

```sh
mlock-mappings 1   # lock all of init's file-backed pages
```

The process stays alive holding the locks. Kill it when you're done. 🍩

## 🍰 Building for aarch64 (static, musl)

```sh
rustup target add aarch64-unknown-linux-musl
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc \
  cargo build --release --target aarch64-unknown-linux-musl
```

## 🍪 License

MIT
