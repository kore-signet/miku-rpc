use miku_rpc::wrappers::{FileImportExport, FileImportExportCard};
use miku_rpc::DeviceBus;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};
use std::time::Instant;

fn main() -> io::Result<()> {
    let out_path = if let Some(path) = env::args().nth(1) {
        path
    } else {
        println!("please specify a file to write to");
        return Ok(());
    };

    let out_f = OpenOptions::new()
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

        out_f.set_len(info.size)?;
        let mut out = BufWriter::new(out_f);

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

        out.flush()?;

        writeln!(stderr, "imported file in {:?}", start.elapsed())?;
    }

    writeln!(stderr, "imported")?;
    Ok(())
}
