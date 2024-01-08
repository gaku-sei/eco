use std::{fs::File, io::Read};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use eco_cbz::Image;
use zip::{read::ZipFile, ZipArchive};

static INDEX: usize = 1;

fn try_from_bytes(bytes: &[u8]) {
    Image::try_from_bytes(bytes).unwrap();
}

fn try_from_zip_file(file: ZipFile) {
    Image::try_from_zip_file(file).unwrap();
}

fn try_into_bytes(bytes: &[u8]) {
    let img = Image::try_from_bytes(bytes).unwrap();
    img.try_into_bytes().unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let test_file_path = std::env::current_dir().unwrap().join("../test.cbz");
    if !test_file_path.exists() {
        panic!("a test.cbz file must be present at the root of this project");
    }

    let mut zip = ZipArchive::new(File::open(test_file_path).unwrap()).unwrap();
    let bytes = {
        let mut file = zip.by_index(INDEX).unwrap();
        let mut bytes = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut bytes).unwrap();
        bytes
    };

    c.bench_function(&format!("try_from_bytes {INDEX}"), |b| {
        b.iter(|| {
            try_from_bytes(black_box(&bytes));
        })
    });

    c.bench_function(&format!("try_from_zip_file {INDEX}"), |b| {
        b.iter(|| {
            let file = zip.by_index(INDEX).unwrap();
            try_from_zip_file(black_box(file));
        })
    });

    c.bench_function(&format!("try_into_bytes {INDEX}"), |b| {
        b.iter(|| {
            try_into_bytes(black_box(&bytes));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
