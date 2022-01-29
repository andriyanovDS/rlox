use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use rlox::bytecode::hash_table::HashTable;
use rlox::bytecode::object_string::ObjectString;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut hash_table = HashTable::<ObjectString, ()>::new();

    let strings: Vec<String> = (0..10_000)
        .map(|_| -> String {
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect()
        })
        .collect();

    c.bench_function("insert", |b| b.iter(|| {
        for string in &strings {
            hash_table.insert(ObjectString::from_string(string.clone()), ());
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);