use std::{fs, sync::Arc};

use macro_rules_attribute::apply;
use pdf2pwg::{render, Error, Format, Resolution};
use smol_macros::test;

#[apply(test!)]
async fn render_file_pwg() -> Result<(), Error> {
    let pdf = fs::read(r"D:\Work\DancesportServices\pdf2pwg\tests\test.pdf").unwrap();
    let rendered = render(
        Arc::new(pdf),
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

#[apply(test!)]
async fn render_file_urf() -> Result<(), Error> {
    let pdf = fs::read(r"D:\Work\DancesportServices\pdf2pwg\tests\test.pdf").unwrap();
    let rendered = render(
        Arc::new(pdf),
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
