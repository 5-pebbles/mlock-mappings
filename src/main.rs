use std::fs;
use std::io;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    about = "Lock file-backed pages from a process's address space into RAM",
    after_help = "\
EXAMPLES:
  mlock-mappings 1              Lock init's file-backed pages
  mlock-mappings /proc/1/maps   Same, with explicit path"
)]
struct Args {
    /// PID or path to a /proc/<pid>/maps file
    #[arg(value_name = "PID|PATH")]
    maps: String,
}

struct Mapping {
    path: String,
    offset: u64,
    len: usize,
}

struct LockedRegion {
    addr: *mut libc::c_void,
    len: usize,
}

impl Drop for LockedRegion {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.addr, self.len);
        }
    }
}

fn parse_mapping(line: &str) -> Option<Mapping> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 6 {
        return None;
    }

    let path = fields[5..].join(" ");
    if !path.starts_with('/') {
        return None;
    }

    let (start, end) = fields[0].split_once('-')?;
    let start = usize::from_str_radix(start, 16).ok()?;
    let end = usize::from_str_radix(end, 16).ok()?;
    let offset = u64::from_str_radix(fields[2], 16).ok()?;

    Some(Mapping {
        path,
        offset,
        len: end - start,
    })
}

fn lock_region(mapping: &Mapping) -> Result<LockedRegion, String> {
    let file = fs::File::open(&mapping.path).map_err(|e| format!("{}: {e}", mapping.path))?;

    let addr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            mapping.len,
            libc::PROT_READ,
            libc::MAP_PRIVATE,
            file.as_raw_fd(),
            mapping.offset as libc::off_t,
        )
    };
    if addr == libc::MAP_FAILED {
        return Err(format!(
            "mmap {}: {}",
            mapping.path,
            io::Error::last_os_error()
        ));
    }

    if unsafe { libc::mlock(addr, mapping.len) } != 0 {
        unsafe {
            libc::munmap(addr, mapping.len);
        }
        return Err(format!(
            "mlock {}: {}",
            mapping.path,
            io::Error::last_os_error()
        ));
    }

    Ok(LockedRegion {
        addr,
        len: mapping.len,
    })
}

fn main() {
    let args = Args::parse();

    let maps_path = match args.maps.parse::<u32>() {
        Ok(pid) => PathBuf::from(format!("/proc/{pid}/maps")),
        Err(_) => PathBuf::from(&args.maps),
    };

    let content = fs::read_to_string(&maps_path).unwrap_or_else(|e| {
        eprintln!("failed to read {}: {e}", maps_path.display());
        std::process::exit(1);
    });

    let mappings: Vec<Mapping> = content.lines().filter_map(parse_mapping).collect();

    let mut locked: Vec<LockedRegion> = Vec::new();
    let mut total: usize = 0;

    for mapping in &mappings {
        match lock_region(mapping) {
            Ok(region) => {
                eprintln!(
                    "{:>10}  {}+{:#x}",
                    mapping.len, mapping.path, mapping.offset
                );
                total += mapping.len;
                locked.push(region);
            }
            Err(e) => eprintln!("FAILED: {e}"),
        }
    }

    // After target regions — MCL_FUTURE would cause mmap failures above.
    if unsafe { libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE) } != 0 {
        eprintln!("warning: mlockall: {}", io::Error::last_os_error());
    }

    eprintln!("\n{} regions locked, {total} bytes", locked.len());
    eprintln!("kill this process to release");

    loop {
        unsafe {
            libc::pause();
        }
    }
}
