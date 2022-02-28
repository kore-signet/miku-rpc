use miku_rpc::wrappers::{FileImportExport, FileImportExportCard};
use miku_rpc::DeviceBus;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> io::Result<()> {
    let out_path = if let Some(path) = env::args().nth(1) {
        PathBuf::from(&path)
    } else {
        println!("please specify a file to write to");
        return Ok(());
    };

    let mut path_buffer = String::new();

    let out_path = if out_path.exists() {
        let stdin = io::stdin();

        loop {
            path_buffer.clear();
            println!("file already exists! [A]bort, [O]verwrite or [R]ename?");
            stdin.read_line(&mut path_buffer)?;
            match path_buffer.trim() {
                "A" => return Ok(()),
                "O" => break out_path,
                "R" => {
                    println!("ok! what do you want the new file renamed to?");
                    stdin.read_line(&mut path_buffer)?;
                    break PathBuf::from(&path_buffer.trim());
                }
                _ => continue,
            }
        }
    } else {
        out_path
    };

    let mut out = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&out_path)?;

    let mut bus = DeviceBus::new("/dev/hvc0")?;

    let card: FileImportExportCard = bus.wrap()?.expect("a file import/export card is required!");
    card.reset(&mut bus)?;

    let stderr_handle = io::stderr();
    let mut stderr = stderr_handle.lock();

    if card.request_import_file(&mut bus)? {
        let info = loop {
            if let Some(data) = card.begin_import_file(&mut bus)? {
                break data;
            }
        };

        writeln!(
            stderr,
            "importing file {} with size {}",
            info.name, info.size
        )?;

        let mut offset: usize = 0;
        let mut last_printed_percent: usize = 0;

        let start = Instant::now();
        while let Some(ref bytes) = card.read_import_file(&mut bus)? {
            out.write_all(bytes)?;

            offset += bytes.len();
            let pct = offset * 100 / info.size as usize;
            if pct >= last_printed_percent + 5 {
                last_printed_percent = pct;
                write!(stderr, "\x1b[2K\r{}% read...", pct)?;
            }
        }

        out.sync_all()?;

        writeln!(stderr, "imported file in {:?}", start.elapsed())?;
    }

    Ok(())
}
