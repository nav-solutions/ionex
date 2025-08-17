extern crate criterion;
use criterion::{criterion_group, criterion_main, Criterion};

use std::io::{BufWriter, Write};

use ionex::prelude::IONEX;

#[derive(Debug)]
pub struct Utf8Buffer {
    pub inner: Vec<u8>,
}

impl Write for Utf8Buffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for b in buf {
            self.inner.push(*b);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.clear();
        Ok(())
    }
}

impl Utf8Buffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }
}

fn ionex_formatting<W: Write>(ionex: &IONEX, w: &mut BufWriter<W>) {
    ionex.format(w).unwrap();
}

fn benchmark(c: &mut Criterion) {
    let mut formatting_grp = c.benchmark_group("formatting");

    let mut buffer = BufWriter::new(Utf8Buffer::new(4096));

    let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap();

    formatting_grp.bench_function("OBS/V2", |b| {
        b.iter(|| {
            ionex_formatting(&ionex, &mut buffer);
        })
    });

    formatting_grp.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
