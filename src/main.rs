use std::io::{self, Read, Write};
use tsutil::packet::Packet;
use tsutil::psi::{create_pat_packet, create_pmt_packet};

const PAYLOAD_LENGTH: usize = 184;

fn main() -> io::Result<()> {
    let mut cc: u8 = 0;
    let mut pat_cc: u8 = 0;
    let mut readbytes: usize = 0;
    let mut wrotebytes: usize = 0;
    let mut buf: [u8; PAYLOAD_LENGTH] = [0; PAYLOAD_LENGTH];
    let mut count = fill_payload(&mut buf)?;
    readbytes = readbytes + count;
    wrotebytes = wrotebytes + write_stream_metadata(&mut pat_cc)?;

    while count == PAYLOAD_LENGTH {
        if wrotebytes % (188 * 500) < 188 {
            wrotebytes = wrotebytes + write_stream_metadata(&mut pat_cc)?;
        }
        wrotebytes = wrotebytes + write_packet(&mut cc, &buf[..count])?;

        count = fill_payload(&mut buf)?;
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
    wrotebytes = wrotebytes + write_packet(&mut cc, &buf[..count])?;
    writeln!(io::stderr(), "Read {}, Wrote {}", readbytes, wrotebytes)?;
    Ok(())
}

fn fill_payload(buf:&mut [u8]) -> io::Result<usize> {

    let mut count = io::stdin().read(buf)?;
    let mut total_read = count;
    while count < PAYLOAD_LENGTH && count > 0 {
        count = io::stdin().read(&mut buf[total_read..])?;
        total_read = total_read + count;
    }
    Ok(total_read)
}

fn write_stream_metadata(cc: &mut u8) -> io::Result<usize> {
    let pat = create_pat_packet(&[0x1000], *cc);
    let mut wrote = io::stdout().write(&pat[..])?;
    let pmt = create_pmt_packet(0x1000, &[(42, 27)], *cc);
    wrote = wrote + io::stdout().write(&pmt[..])?;
    *cc = (*cc + 1) % 0xF;
    Ok(wrote)
}

fn write_packet(cc: &mut u8, payload: &[u8]) -> io::Result<usize> {
    let pkt = Packet::create_packet_with_payload(false, false, true, 42, 0, 1, *cc, &payload);
    let wrote = io::stdout().write(&pkt[..])?;
    *cc = (*cc + 1) % 0xF;
    Ok(wrote)
}
