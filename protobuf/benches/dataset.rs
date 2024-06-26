use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use criterion::{criterion_group, criterion_main, Criterion};
use prost::Message;

use protobuf::benchmarks::{dataset, proto2, proto3, BenchmarkDataset};

fn load_dataset(dataset: &Path) -> Result<BenchmarkDataset, Box<dyn Error>> {
    let mut f = File::open(dataset)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(BenchmarkDataset::decode(buf.as_slice())?)
}

fn benchmark_dataset<M>(criterion: &mut Criterion, name: &str, dataset: &'static Path)
where
    M: prost::Message + Default + 'static,
{
    let mut group = criterion.benchmark_group(&format!("dataset/{}", name));

    group.bench_function("merge", move |b| {
        let dataset = load_dataset(dataset).unwrap();
        let mut message = M::default();
        b.iter(|| {
            for buf in &dataset.payload {
                message.clear();
                message.merge(buf.as_slice()).unwrap();
                criterion::black_box(&message);
            }
        });
    });

    group.bench_function("encode", move |b| {
        let messages = load_dataset(dataset)
            .unwrap()
            .payload
            .iter()
            .map(Vec::as_slice)
            .map(M::decode)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let mut buf = Vec::with_capacity(messages.iter().map(M::encoded_len).sum::<usize>());
        b.iter(|| {
            buf.clear();
            for message in &messages {
                message.encode(&mut buf).unwrap();
            }
            criterion::black_box(&buf);
        });
    });

    group.bench_function("encoded_len", move |b| {
        let messages = load_dataset(dataset)
            .unwrap()
            .payload
            .iter()
            .map(Vec::as_slice)
            .map(M::decode)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        b.iter(|| {
            let encoded_len = messages.iter().map(M::encoded_len).sum::<usize>();
            criterion::black_box(encoded_len)
        });
    });
}

macro_rules! dataset {
    ($name: ident, $ty: ty) => {
        fn $name(criterion: &mut Criterion) {
            benchmark_dataset::<$ty>(criterion, stringify!($name), dataset::$name());
        }
    };
}

dataset!(google_message1_proto2, proto2::GoogleMessage1);
dataset!(google_message1_proto3, proto3::GoogleMessage1);
dataset!(google_message2, proto2::GoogleMessage2);

criterion_group!(
    dataset,
    google_message1_proto2,
    google_message1_proto3,
    google_message2,
);

criterion_main!(dataset);
