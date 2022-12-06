//extern crate memmap;
use anyhow::Result;
use memmap::Mmap;
use std::{fs::File, num::ParseIntError, ops::Add, str::Bytes, string::ParseError, u8, vec};

use core::ptr::read_volatile;

fn map_physical_mem(addr: usize, len: usize) -> Result<Mmap> {
    let m = unsafe {
        memmap::MmapOptions::new()
            .offset(addr as u64)
            .len(len)
            .map(&File::open("/dev/mem")?)?
    };
    Ok(m)
}

// fn dump_mem_u8(addr: usize, len: usize) {
//     let m = match map_physical_mem(addr, len) {
//         Ok(m) => m,
//         Err(err) => {
//             panic!("Failed to mmap: Err={:?}", err);
//         }
//     };
//     (0..len).for_each(|x| println!("{:016x}: {:02x}", addr + x, m[x]));
// }

fn dump_mem0<T>(addr: usize, len: usize) -> String
where
    T: std::fmt::LowerHex,
{
    let sz = std::mem::size_of::<T>();
    let m = match map_physical_mem(addr, len * sz) {
        Ok(m) => m,
        Err(err) => {
            return format!("Failed to mmap: Err={:?}", err);
        }
    };
    let mut s = String::new();
    let p = m.as_ptr() as *const T;
    (0..len).for_each(|x| unsafe {
        let ss = format!(
            "{:016x}: {:02$x}",
            addr + sz * x,
            std::ptr::read_volatile(p.offset(x as isize)),
            sz * 2
        );

        s += &ss;
    });
    s
}

fn dump_mem<T>(addr: usize, len: usize, len_row: usize) -> String
where
    T: std::fmt::LowerHex,
{
    let sz = std::mem::size_of::<T>();
    let v = vec_mem::<T>(addr, len);

    let mut res = String::new();
    let mut col_pos = 0;
    for i in 0..len {
        if col_pos >= len_row {
            col_pos = 0;
            res += "\n";
        }
        let byte = format!("{:#01$x}, ", v[i], sz * 2 + 2);

        col_pos += 1;
        res += &byte;
    }
    res
}

pub fn dump_mem_u8(addr: usize, len: usize, len_row: usize) -> String {
    let sz = 1;
    let v = u8_vec_mem(addr, len);

    let mut s = String::new();
    let mut s_char = String::new();
    let mut col_pos = 0;
    for i in 0..len {
        if col_pos >= len_row {
            col_pos = 0;
            s += &format!(" // {s_char} \n");
            s_char.clear();
        }
        let byte = format!("{:#01$x}, ", v[i], sz * 2 + 2);
        s += &byte;

        let char = v[i] as char;
        s_char.push(char);

        col_pos += 1;
    }

    return s;
}

pub fn u8_vec_mem(addr: usize, len: usize) -> Vec<u8> {
    //let sz = std::mem::size_of::<T>();
    let mut v = Vec::new();
    let p = addr as *const u8;

    for i in 0..len {
        unsafe {
            let m = std::ptr::read_volatile(p.offset(i as isize));
            v.push(m);
        }
    }
    v
}

pub fn vec_mem<T>(addr: usize, len: usize) -> Vec<T>
where
    T: std::fmt::LowerHex,
{
    //let sz = std::mem::size_of::<T>();
    let mut v = Vec::new();
    if addr == 0 {
        return v;
    }
    let p = addr as *const T;

    for i in 0..len {
        unsafe {
            let m = std::ptr::read_volatile(p.offset(i as isize));
            v.push(m);
        }
    }
    v
}

pub fn parse_hex(s: &str) -> Result<usize, ParseIntError> {
    if s.starts_with("0x") || s.starts_with("0X") {
        usize::from_str_radix(&s[2..], 16)
    } else {
        usize::from_str_radix(&s, 10)
    }
}

fn print_usage(name: &str) {
    eprintln!("Dump physical memory by specified size.");
    eprintln!("Usage: {} size address [len]", name);
    eprintln!("  where size={{1,2,4,8}}, address and len in hexadecimal.");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        print_usage(&args[0]);
        return;
    }
    let addr = parse_hex(&args[2]).expect("address parse error");
    let len: usize = if args.len() >= 4 {
        parse_hex(&args[3]).unwrap_or(1)
    } else {
        1
    };
    match args[1].parse() {
        Ok(1) => dump_mem::<u8>(addr, len, 8),
        Ok(2) => dump_mem::<u16>(addr, len, 8),
        Ok(4) => dump_mem::<u32>(addr, len, 8),
        Ok(8) => dump_mem::<u64>(addr, len, 8),
        _ => {
            print_usage(&args[0]);
            "".to_string()
        }
    };
}

pub fn main_dump(bytes: usize, addr: usize, len: usize, len_row: usize) -> String {
    match bytes {
        1 => dump_mem_u8(addr, len, len_row),
        2 => dump_mem::<u16>(addr, len, len_row),
        4 => dump_mem::<u32>(addr, len, len_row),
        8 => dump_mem::<u64>(addr, len, len_row),
        _ => "".to_string(),
    }
}
