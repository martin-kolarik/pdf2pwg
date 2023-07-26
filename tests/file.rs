use std::{fs, sync::Arc};

use pdf2pwg::{render, Output, Resolution};
use pdfium_render::prelude::PdfiumError;

#[async_std::test]
async fn render_file_pwg() -> Result<(), PdfiumError> {
    let pdf = fs::read(r"D:\Work\DancesportServices\pdf2pwg\tests\test.pdf").unwrap();
    let pages = render(
        Arc::new(pdf),
        Resolution::Dpi600,
        Resolution::Dpi600,
        Output::Pwg,
    )
    .await?;

    for (index, page) in pages.iter().enumerate() {
        fs::write(
            format!(r"D:\Work\DancesportServices\pdf2pwg\target\debug\page-{index}.pwg"),
            &page,
        )
        .unwrap();
    }

    Ok(())
}

#[async_std::test]
async fn render_file_urf() -> Result<(), PdfiumError> {
    let pdf = fs::read(r"D:\Work\DancesportServices\pdf2pwg\tests\test.pdf").unwrap();
    let pages = render(
        Arc::new(pdf),
        Resolution::Dpi600,
        Resolution::Dpi600,
        Output::Urf,
    )
    .await?;

    for (index, page) in pages.iter().enumerate() {
        fs::write(
            format!(r"D:\Work\DancesportServices\pdf2pwg\target\debug\page-{index}.urf"),
            &page,
        )
        .unwrap();
    }

    Ok(())
}
