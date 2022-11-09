use crate::{connection::Connection, mem_dump::main_dump};
use anyhow::Result;
use tracing_subscriber::fmt::format;
use crate::mem_dump::parse_hex;



pub async fn command(msg: &mut str, conn: &mut Connection) -> Result<()> {
    
    let words = msg.split_whitespace().collect::<Vec<&str>>();
    let com = words[0];

    let mut res = String::new();

    if words.len() < 2 {
    } else if com == "mem" {
        if words[1] == "dump" { 
            let addr = parse_hex(&words[2]).expect("address parse error");
            let len: usize = if words.len() >= 4 {
                parse_hex(&words[3]).unwrap_or(1)
            } else {
                1
            };

            let x = 16;
            let addr2 = &x as *const i32 as usize;
            res += &format!("addr2 = {addr2:?}; \n");
            
            res += &format!("addr = {addr:?}; \n");
            res += &main_dump(1, addr, len, 8);

        } else if words[1] == "write" {
            let addr = parse_hex(&words[2]).expect("address parse error");
            let mut cnt = 0;
            for i in 3..words.len() {
                let val = match parse_hex(&words[i]) {
                    Ok(v) if v <= u8::MAX as usize => v as u8,
                    Ok(v) => {res += &format!("error: i:{} v:{v} > {} \n", i-2, u8::MAX); continue;},
                    Err(e) => {res += &format!("{e} \n"); continue;},
                };
                write_mem::<u8>(addr+cnt, val);
                cnt += 1;
                
            }
            res += &format!("writed {cnt} bytes to address {:#01$x}, ", addr, (usize::BITS/4+2) as usize);
        }
    } else if com == "s" {

        let foo = dl_sym(words[1]);
        res += &format!("symbol {} at {foo:?}", words[1]);

    } else if com == "r" {

        if words.len() == 2 { // //r 0x76cadc9 # Резолвим функцию по адресу
            res += &match parse_hex(words[1]){
                Ok(addr) => {
                    let info = unsafe {
                        let mut info: Dl_info = std::mem::zeroed(); // intrinsics::init();
                        if dladdr( addr as *mut _, &mut info) == 0 || info.dli_sname.is_null() {
                            None
                        } else {
                            Some((CStr::from_ptr(info.dli_sname).to_str().ok().unwrap_or_default(),
                             CStr::from_ptr(info.dli_fname).to_str().ok().unwrap_or_default()
                            ))
                        }
                    };
                    if let Some((symname, dll_file)) = info {
                        format!("Address {addr:#x} located at {dll_file} within the program {symname}")
                    } else {
                        format!("Address not resolved - {} - {addr:#x}", words[1])
                    }
                }
                Err(e) => format!("{e} - {}", words[1]),
            }

    
        } else if words[1].starts_with("u") { //r u32 g_some_var
            res += &cell_rw(&words);
            
        }

    } else if com == "w" { // w u32 g_some_var = 0x100500
        res += &cell_rw(&words);

    }

    if res.is_empty() {
        res = format!("bad command - {msg}");
    }

    
    conn.send_message(&res).await
}

fn bites(sym_bits: &str) -> usize {
    match sym_bits{
        "u8" =>  1,
        "u16" => 2,
        "u32" => 4,
        _ => (usize::BITS/8) as usize,
    }
}

fn cell_rw(words: &Vec<&str>) -> String {
    if words.len() < 3 {
        return format!("");
    }
    let mut symbol = words[2];
    let addr = match parse_hex(symbol) {
        Ok(addr) => {symbol = ""; addr as *const u8},
        Err(_) => dl_sym(symbol),
    };
    if addr.is_null() {
        return format!("Symbol not resolved - {symbol} - {addr:?}");
    }
    
    if words[0] == "r" {
        let bites = bites(words[1]);
        let val = main_dump(bites, addr as usize, 1, 1);
        return format!("{symbol}({addr:?})={val}");
    } else if words[0] == "w" {
        let val = *words.last().unwrap();
        let val = match parse_hex(val){
            Ok(v) => v,
            Err(e) => return format!("{e} - {val}"),
        };
        let addr = addr as usize;
        match words[1] {
            "u8"  => write_mem::<u8>(addr, val as u8),
            "u16" => write_mem::<u16>(addr, val as u16),
            "u32" => write_mem::<u32>(addr, val as u32),
            "u64" if usize::BITS == 64 => 
                     write_mem::<u64>(addr, val as u64),
            u => return format!("Bit depth not resolved - {u}"),
        }
    
        return format!("Writen: {symbol}({addr:?})={val}");
    } else {
        return format!("Bad command - {}", words[0]);
    }
}

fn write_mem<T>(addr: usize, val: T) {
    let p = addr as *mut T;
    unsafe {
        std::ptr::write_volatile(p, val);
    }
}



use std::os::raw::{c_void, c_char, c_int};
use std::ffi::CStr;

extern {
    fn dladdr(addr: *const c_void, info: *mut Dl_info) -> c_int;
}

#[repr(C)]
struct Dl_info {
    dli_fname: *const c_char,
    dli_fbase: *mut c_void,
    dli_sname: *const c_char,
    dli_saddr: *mut c_void,
}

#[link(name="dl")]
extern {
    //fn dlopen(filename: *const u8, flags: isize) -> *const u8;
    fn dlsym(handle: *const u8, symbol: *const u8) -> *const u8;
}
fn dl_sym(symbol: &str) -> *const u8 {
    let symbol0 = format!("{symbol}\0");
    let symbol = symbol0.as_ptr();
    //let libdl = unsafe { dlopen(b"libdl\0" as *const _, 2) };
    
    let foo: *const u8 = unsafe {
        let libdl = std::mem::zeroed();
        std::mem::transmute( dlsym(libdl, symbol as *const _) )
    };
    foo
}

