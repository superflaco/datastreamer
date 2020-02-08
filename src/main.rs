use clap::clap_app;
use std::fs::File;
use std::io::{self, Read, Write};
use tsutil::packet::Packet;
use tsutil::psi::{create_pat_packet, create_pmt_packet};

const PAYLOAD_LENGTH: usize = 184;

fn main() -> io::Result<()> {
    let cli_matches = clap_app!(blah => (about: "Packages up arbitrary data as MPEGTS stream")
    (@arg OUTPUT: -o --output +takes_value "MPEGTS output destination")
    (@arg INPUT: -i --input +takes_value "MPEGTS input source"))
    .get_matches();
    let mut tmpfile;
    let mut stdin;
    let input_file: &mut dyn Read = if let Some(input_val) = cli_matches.value_of("INPUT") {
        let fopen_result = File::open(input_val);
        match fopen_result {
            Ok(open_file) => {
                tmpfile = open_file;
                &mut tmpfile
            }
            Err(ferr) => {
                writeln!(io::stderr(), "{}", ferr)?;
                stdin = io::stdin();
                &mut stdin
            }
        }
    } else {
        stdin = io::stdin();
        &mut stdin
    };
    let mut outtmpfile;
    let mut stdout;
    let output_file: &mut dyn Write = if let Some(output_val) = cli_matches.value_of("OUTPUT") {
        let fopen_result = File::create(output_val);
        match fopen_result {
            Ok(open_file) => {
                outtmpfile = open_file;
                &mut outtmpfile
            }
            Err(ferr) => {
                writeln!(io::stderr(), "{}", ferr)?;
                stdout = io::stdout();
                &mut stdout
            }
        }
    } else {
        stdout = io::stdout();
        &mut stdout
    };
    let mut cc: u8 = 0;
    //let mut pat_cc: u8 = 0;
    let mut readbytes: usize = 0;
    let mut wrotebytes: usize = 0;
    let mut buf: [u8; PAYLOAD_LENGTH] = [0; PAYLOAD_LENGTH];
    let mut count = fill_payload(input_file, &mut buf)?;
    readbytes = readbytes + count;
    //wrotebytes = wrotebytes + write_stream_metadata(output_file, &mut pat_cc)?;

    while count == PAYLOAD_LENGTH {
        /*if wrotebytes % (188 * 500) < 188 {
            wrotebytes = wrotebytes + write_stream_metadata(output_file, &mut pat_cc)?;
        }*/
        wrotebytes = wrotebytes + write_packet(output_file, &mut cc, &buf)?;

        count = fill_payload(input_file, &mut buf)?;
        readbytes = readbytes + count;
        if count < PAYLOAD_LENGTH {
            writeln!(
                io::stderr(),
                "just read less than payload {} at {}",
                count,
                readbytes
            )?;
        }
    }
    wrotebytes = wrotebytes + write_packet(output_file, &mut cc, &buf[..count])?;
    writeln!(io::stderr(), "Read {}, Wrote {}", readbytes, wrotebytes)?;
    Ok(())
}

fn fill_payload(input_file: &mut dyn Read, buf: &mut [u8]) -> io::Result<usize> {
    let mut count = input_file.read(buf)?;
    let mut total_read = count;
    while count < PAYLOAD_LENGTH && count > 0 {
        count = input_file.read(&mut buf[total_read..])?;
        total_read = total_read + count;
    }
    Ok(total_read)
}

#[allow(dead_code)]
fn write_stream_metadata(output_file: &mut dyn Write, cc: &mut u8) -> io::Result<usize> {
    let pat = create_pat_packet(&[0x1000], *cc);
    let mut wrote = io::stdout().write(&pat[..])?;
    let pmt = create_pmt_packet(0x1000, &[(42, 27)], *cc);
    wrote = wrote + output_file.write(&pmt[..])?;
    *cc = (*cc + 1) % 0xF;
    Ok(wrote)
}

fn write_packet(output_file: &mut dyn Write, cc: &mut u8, payload: &[u8]) -> io::Result<usize> {
    let pkt = Packet::create_packet_with_payload(false, false, true, 42, 0, 1, *cc, &payload);
    let wrote = output_file.write(&pkt[..])?;
    *cc = (*cc + 1) % 0xF;
    Ok(wrote)
}
