extern crate memmap;
use memmap::MmapMut;
use std::fs::OpenOptions;
use std::error::Error;

fn map_physical_mem_rw(addr: usize, len: usize) -> Result<MmapMut, Box<Error>> {
    let f = OpenOptions::new().read(true).write(true).open("/dev/mem")?;
    let m = unsafe {
        memmap::MmapOptions::new()
            .offset(addr as u64)
            .len(len)
            .map_mut(&f)?
    };
    Ok(m)
}

fn write_mem<T>(addr: usize, val: T)
where
    T: std::fmt::LowerHex,
{
    let sz = std::mem::size_of::<T>();
    let m = match map_physical_mem_rw(addr, sz) {
        Ok(m) => m,
        Err(err) => {
            panic!("Failed to mmap: Err={:?}", err);
        }
    };
    let p = m.as_ptr() as *mut T;
    unsafe {
        std::ptr::write_volatile(p, val);
    }
}

fn parse_hex(s: &String) -> Result<usize, Box<Error>> {
    let s = if s.starts_with("0x") || s.starts_with("0X") {
        &s[2..]
    } else {
        s
    };
    Ok(usize::from_str_radix(s, 16)?)
}

fn print_usage(name: &str) {
    eprintln!("Write physical memory by specified size.");
    eprintln!("Usage: {} size address val", name);
    eprintln!("  where size={{1,2,4,8}}, address and val in hexadecimal.");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        print_usage(&args[0]);
        return;
    }
    let addr = parse_hex(&args[2]).expect("address parse error");
    let val = parse_hex(&args[3]).expect("value parse error");
    match args[1].parse() {
        Ok(1) => write_mem::<u8>(addr, val as u8),
        Ok(2) => write_mem::<u16>(addr, val as u16),
        Ok(4) => write_mem::<u32>(addr, val as u32),
        Ok(8) => write_mem::<u64>(addr, val as u64),
        _ => print_usage(&args[0]),
    }
}