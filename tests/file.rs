use std::{
    fs,
    sync::{Arc, LazyLock, Mutex},
};

use artwrap::{block_on, with_main_async};
use pdf2pwg::{render, Error, Format, Orientation, Resolution};

static LOCK: LazyLock<Arc<Mutex<u8>>> = LazyLock::new(|| Arc::new(Mutex::new(0)));

#[test]
fn render_file_pwg() -> Result<(), Error> {
    block_on(async {
        //let _unused = LOCK.lock().unwrap();

        let pdf = fs::read(r"D:\Work\DancesportServices\pdf2pwg\tests\test.pdf").unwrap();
        let rendered = render(
            Arc::new(pdf),
            Format::Pwg,
            Orientation::Portrait,
            Resolution::Dpi600,
            Resolution::Dpi600,
        )
        .await?;

        fs::write(
            format!(r"D:\Work\DancesportServices\pdf2pwg\target\debug\test.pwg"),
            rendered,
        )
        .unwrap();

        Ok(())
    })
}

#[test]
fn render_file_urf() -> Result<(), Error> {
    block_on(async {
        //let _unused = LOCK.lock().unwrap();

        let pdf = fs::read(r"D:\Work\DancesportServices\pdf2pwg\tests\test.pdf").unwrap();
        let rendered = render(
            Arc::new(pdf),
            Format::Urf,
            Orientation::Portrait,
            Resolution::Dpi600,
            Resolution::Dpi600,
        )
        .await?;

        fs::write(
            format!(r"D:\Work\DancesportServices\pdf2pwg\target\debug\test.urf"),
            rendered,
        )
        .unwrap();

        Ok(())
    })
}
