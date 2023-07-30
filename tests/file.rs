use std::fs;

use bytes::Bytes;
use pdf2pwg::{render, Error, Format, Resolution};

#[async_std::test]
async fn render_file_pwg() -> Result<(), Error> {
    let pdf = fs::read(r"D:\Work\DancesportServices\pdf2pwg\tests\test.pdf").unwrap();
    let rendered = render(
        Bytes::from(pdf),
        Resolution::Dpi600,
        Resolution::Dpi600,
        Format::Pwg,
    )
    .await?;

    fs::write(
        format!(r"D:\Work\DancesportServices\pdf2pwg\target\debug\test.pwg"),
        rendered,
    )
    .unwrap();

    Ok(())
}

#[async_std::test]
async fn render_file_urf() -> Result<(), Error> {
    let pdf = fs::read(r"D:\Work\DancesportServices\pdf2pwg\tests\test.pdf").unwrap();
    let rendered = render(
        Bytes::from(pdf),
        Resolution::Dpi600,
        Resolution::Dpi600,
        Format::Urf,
    )
    .await?;

    fs::write(
        format!(r"D:\Work\DancesportServices\pdf2pwg\target\debug\test.urf"),
        rendered,
    )
    .unwrap();

    Ok(())
}
