use std::io::{Error, Write};

pub fn compress<W>(
    bitmap: &[u8],
    pixel_width: usize,
    bits_per_pixel: usize,
    compressed: &mut W,
) -> Result<(), Error>
where
    W: Write,
{
    let bytes_per_line = (pixel_width * bits_per_pixel + 7) / 8;
    let mut lines = bitmap.chunks(bytes_per_line);

    if let Some(mut first) = lines.next() {
        let mut count = 1;
        while let Some(next) = lines.next() {
            if first != next {
                flush_lines(count, first, compressed)?;
                first = next;
                count = 0;
            }
            count += 1;
        }
        flush_lines(count, first, compressed)?;
    }

    Ok(())
}

fn flush_lines<W>(mut count: usize, line: &[u8], compressed: &mut W) -> Result<(), Error>
where
    W: Write,
{
    while count > 0 {
        let chunk = count.min(256);
        compressed.write_all(&[(chunk - 1) as u8])?;
        compress_line(line, compressed)?;
        count -= chunk;
    }
    Ok(())
}

fn compress_line<W>(line: &[u8], compressed: &mut W) -> Result<(), Error>
where
    W: Write,
{
    let mut groups = line.group_by(|current, next| current == next);
    let mut index = 0;
    let mut differring_len = None;

    while let Some(group) = groups.next() {
        if group.len() > 1 {
            flush_different(line, &mut index, &mut differring_len, compressed)?;
            flush_rle(&mut index, group, compressed)?;
        } else if let Some(differring_len) = &mut differring_len {
            *differring_len += 1;
        } else {
            differring_len = Some(1)
        }
    }
    // flush possible remainder
    flush_different(line, &mut index, &mut differring_len, compressed)?;

    Ok(())
}

fn flush_rle<W>(index: &mut usize, group: &[u8], compressed: &mut W) -> Result<(), Error>
where
    W: Write,
{
    for chunk in group.chunks(128) {
        compressed.write_all(&[(chunk.len() - 1) as u8])?;
        compressed.write_all(&chunk[..1])?;
    }
    *index += group.len();

    Ok(())
}

fn flush_different<W>(
    line: &[u8],
    index: &mut usize,
    differring_len: &mut Option<usize>,
    compressed: &mut W,
) -> Result<(), Error>
where
    W: Write,
{
    if let Some(differring_len) = differring_len.take() {
        for chunk in line[*index..*index + differring_len].chunks(128) {
            compressed.write_all(&[(257 - chunk.len()) as u8])?;
            compressed.write_all(chunk)?;
        }
        *index += differring_len;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwg_1bit_example_lines() {
        let line1 = [0b10001111_u8, 0b01111000, 0b11110111];
        let line2 = [0b01110110_u8, 0b01110111, 0b01100111];
        let line3 = [0b01110111_u8, 0b01110111, 0b01110111];
        let line4 = [0b10001110_u8, 0b00111000, 0b11100011];
        let line5 = [0xff_u8, 0xff_u8, 0xff_u8];

        let mut output = Vec::with_capacity(8);
        let _ = compress_line(&line1, &mut output);
        assert_eq!(&[0xfe_u8, 0x8f, 0x78, 0xf7], output.as_slice());

        let mut output = Vec::with_capacity(8);
        let _ = compress_line(&line2, &mut output);
        assert_eq!(&[0xfe_u8, 0x76, 0x77, 0x67], output.as_slice());

        let mut output = Vec::with_capacity(8);
        let _ = compress_line(&line3, &mut output);
        assert_eq!(&[0x02_u8, 0x77], output.as_slice());

        let mut output = Vec::with_capacity(8);
        let _ = compress_line(&line4, &mut output);
        assert_eq!(&[0xfe_u8, 0x8e, 0x38, 0xe3], output.as_slice());

        let mut output = Vec::with_capacity(8);
        let _ = compress_line(&line5, &mut output);
        assert_eq!(&[0x02_u8, 0xff], output.as_slice());
    }

    #[test]
    fn pwg_1bit_example_bitmap() {
        let bitmap = [
            0b10001111_u8,
            0b01111000,
            0b11110111,
            0b01110110_u8,
            0b01110111,
            0b01100111,
            0b01110111_u8,
            0b01110111,
            0b01110111,
            0b01110111_u8,
            0b01110111,
            0b01110111,
            0b01110111_u8,
            0b01110111,
            0b01110111,
            0b01110111_u8,
            0b01110111,
            0b01110111,
            0b10001110_u8,
            0b00111000,
            0b11100011,
            0xff_u8,
            0xff_u8,
            0xff_u8,
        ];

        let expected = [
            0x0_u8, 0xfe, 0x8f, 0x78, 0xf7, 0x0_u8, 0xfe, 0x76, 0x77, 0x67, 0x3_u8, 0x02, 0x77,
            0x0_u8, 0xfe, 0x8e, 0x38, 0xe3, 0x0_u8, 0x02, 0xff,
        ];

        let mut output = Vec::with_capacity(32);
        let _ = compress(&bitmap, 23, 1, &mut output);
        assert_eq!(&expected, output.as_slice());
    }
}
