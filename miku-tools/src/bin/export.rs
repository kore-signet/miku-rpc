use miku_rpc::wrappers::{FileImportExport, FileImportExportCard};
use miku_rpc::DeviceBus;
use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> io::Result<()> {
    let out_path = PathBuf::from(
        env::args()
            .nth(1)
            .expect("please specify the file to export!"),
    );
    let name = out_path
        .file_name()
        .and_then(|v| v.to_str())
        .expect("couldn't find file name - is the path you specified valid?");

    let mut input =
        BufReader::new(File::open(&out_path).expect("couldn't open file for exporting!"));

    let mut bus = DeviceBus::new("/dev/hvc0")?;

    let card: FileImportExportCard = bus.wrap()?.expect("a file import/export card is required!");
    card.reset(&mut bus)?;

    let stderr_handle = io::stderr();
    let mut stderr = stderr_handle.lock();

    // this is to ensure we don't go over 4096 bytes - it's not optimal, but cleaner than a solution that stretches every read to the limit.
    let mut read_buffer: [u8; 900] = [0; 900];
    let mut read_total = 0;
    let mut last_printed_percent: usize = 0;

    let file_len = input.get_ref().metadata()?.len() as usize;

    card.begin_export_file(&mut bus, name)?;

    writeln!(stderr, "exporting file {} with size {}", name, file_len)?;

    let start = Instant::now();

    loop {
        let bytes_read = input.read(&mut read_buffer)?;
        if bytes_read == 0 {
            break;
        }

        read_total += bytes_read;
        card.write_export_file(&mut bus, &read_buffer[..bytes_read])?;

        let pct = read_total * 100 / file_len;
        if pct >= last_printed_percent + 5 {
            last_printed_percent = pct;
            write!(stderr, "\x1b[2K\r{}% written...", pct)?;
        }
    }

    card.finish_export_file(&mut bus)?;
    writeln!(stderr, "exported file in {:?}!", start.elapsed())?;

    Ok(())
}
