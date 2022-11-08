use crate::{connection::Connection, mem_dump::main_dump};
use anyhow::Result;
use crate::mem_dump::parse_hex;



pub async fn command(msg: &mut str, conn: &mut Connection) -> Result<()> {
    
    let words = msg.split_whitespace().collect::<Vec<&str>>();
    let com = words[0];

    let mut res = com;
    let mut res2 = String::new();

    if com == "mem" && words[1] == "dump" { 
        let addr = parse_hex(&words[2]).expect("address parse error");
        let len: usize = if words.len() >= 4 {
            parse_hex(&words[3]).unwrap_or(1)
        } else {
            1
        };
        res2 = main_dump(1, addr, len);
    }
    res = res2.as_str();
    //mem dump 0x2378423 64
    //mem write 0x2378423 01 02 FF AA CC

    //s malloc
    //s g_some_var



    
    conn.send_message(&res).await
}