use criterion::{criterion_group, criterion_main, Criterion};
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use rand::distributions::Alphanumeric;
use tempfile::TempDir;
use kvs::{KvStore, SledKvsEngine, KvsEngine};

/// Create a stirng with a random byte size between 0 and 100000
pub fn get_random_string(rng: &mut ThreadRng) -> String {
    // Generate random byte size
    let size = rng.gen_range(1..100000);

    // Create string with random byte size
    rng.sample_iter(&Alphanumeric).take(size).map(char::from).collect()
}

pub fn kvs_benchmark(c: &mut Criterion) {
    // Create temporary directory and create a new database on it
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path()).expect("unable to create KvStore at the given path");

    // Create a random number generator and a keys array that will hold 100 keys
    let mut rng = thread_rng();
    let mut keys = Vec::new();

    c.bench_function("kvs_write", |b| b.iter(|| {
        // Generate random string key and value
        let key = get_random_string(&mut rng);
        let value = get_random_string(&mut rng);

        // Store key in array
        keys.push(key.clone());

        // Set key-value pair in the store
        store.set(key, value).expect("failed to set value");
    }));

    c.bench_function("kvs_read", |b| b.iter(|| {
        // Retrieve random key from keys array
        let key = &keys[rng.gen_range(0..keys.len())];

        // Get key-value pair from the store
        store.get(key.clone()).expect("failed to get value");
    }));
}

pub fn sled_benchmark(c: &mut Criterion) {
    // Create temporary directory and create a new database on it
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = SledKvsEngine::open(temp_dir.path()).expect("unable to create Sled store at the given path");

    // Create a random number generator and a keys array that will hold 100 keys
    let mut rng = thread_rng();
    let mut keys = Vec::new();

    c.bench_function("sled_write", |b| b.iter(|| {
        // Generate random string key and value
        let key = get_random_string(&mut rng);
        let value = get_random_string(&mut rng);

        // Store key in array
        keys.push(key.clone());

        // Set key-value pair in the store
        store.set(key, value).expect("failed to set value");
    }));

    c.bench_function("sled_read", |b| b.iter(|| {
        // Retrieve random key from keys array
        let key = &keys[rng.gen_range(0..keys.len())];

        // Get key-value pair from the store
        store.get(key.clone()).expect("failed to get value");
    }));
}

criterion_group!(benches, kvs_benchmark, sled_benchmark);
criterion_main!(benches);