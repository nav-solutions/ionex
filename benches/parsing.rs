extern crate criterion;

use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

use ionex::prelude::IONEX;

fn ionex_parsing(path: &str) {
    let _ = IONEX::from_gzip_file(path).unwrap();
}

fn benchmark(c: &mut Criterion) {
    let mut parsing_grp = c.benchmark_group("parsing");

    parsing_grp.measurement_time(Duration::from_secs(20));

    parsing_grp.bench_function("IONEX/V1", |b| {
        b.iter(|| {
            ionex_parsing("data/IONEX/V1/CKMG0020.22I.gz");
        })
    });

    parsing_grp.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
