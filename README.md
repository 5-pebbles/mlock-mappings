# mlock-mappings 🍬

Lock a process's file-backed pages into RAM so it survives having its backing storage overwritten.

Parses `/proc/<pid>/maps`, mmaps the same file regions into its own address space, and mlocks them - pinning the shared page cache pages in physical memory. The target process's page table entries resolve to the same frames, so it won't fault on disk reads that no longer exist. 🍭

## 🍫 Usage

```sh
mlock-mappings 1              # by PID
mlock-mappings /proc/1/maps   # by maps file path
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
