extern crate getopts;
extern crate memmap;

use getopts::Options;
use memmap::Mmap;
use memmap::MmapMut;
use std::cmp;
use std::env;
use std::fs::OpenOptions;
use std::num::ParseIntError;

struct Partition {
    file: String,
    size: usize,
}

fn parse_number(number: &str) -> Result<usize, ParseIntError> {
    if number.len() > 2 {
        if &number[..2] == "0x" {
            return Ok(usize::from_str_radix(&number[2..], 16))?;
        }
    }
    return Ok(usize::from_str_radix(number, 10))?;
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] file1,size file2,size ...", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("o", "output", "set output file name", "NAME");
    opts.optopt("f", "fill", "fill image with bytes", "BYTE");
    opts.optflag("q", "quiet", "keep silence");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let quiet = matches.opt_present("q");

    let fill =
        parse_number(&matches.opt_str("f").unwrap_or(String::from("0xff"))).unwrap_or(255) as u8;
    let output = matches.opt_str("o").unwrap_or(String::from("output.bin"));
    if matches.free.is_empty() {
        print_usage(&program, opts);
        return;
    }

    let mut total = 0;
    let mut parts: Vec<Partition> = Vec::new();
    for p in &matches.free {
        let param: Vec<&str> = p.split(',').collect();
        if param.len() < 2 {
            println!("no size specify for file {}", &param[0]);
            return;
        }

        let part = Partition {
            file: param[0].to_string(),
            size: parse_number(&param[1]).unwrap_or(0),
        };
        total += part.size;
        parts.push(part);
    }

    let mut offset = 0;
    if !quiet {
        println!(
            "creating output file:{} total size: {:#08X} ({} bytes)",
            &output, total, total
        );
    }
    if let Ok(outfile) = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&output)
    {
        outfile
            .set_len(total as u64)
            .expect("set output file length failed");
        let mut mmap_out = unsafe {
            MmapMut::map_mut(&outfile).expect(&format!("unable to map output file: {}", &output))
        };
        mmap_out.fill(fill);
        mmap_out.flush().expect("flush output file failed!");

        for part in &parts {
            if part.file == "padding" {
                // skip
                offset += part.size;
            } else if let Ok(infile) = OpenOptions::new().read(true).open(&part.file) {
                let mmap_in = unsafe {
                    Mmap::map(&infile).expect(&format!("unable to map input file: {}", &part.file))
                };
                let size = cmp::min(mmap_in.len(), part.size);
                if !quiet {
                    println!(
                        "  > input:{} offset:{:#08X} file size:{:#08X} partition size:{:#08X}",
                        &part.file,
                        offset,
                        mmap_in.len(),
                        part.size
                    );
                }
                mmap_out[offset..offset + size].copy_from_slice(&mmap_in[..size]);
                offset += part.size;
            } else {
                println!("unable to open input file: {}", &part.file);
            }
        }
    } else {
        println!("unable to open output file: {}", &output);
        return;
    }

}
