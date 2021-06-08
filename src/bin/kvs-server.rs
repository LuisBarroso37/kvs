use kvs::{Engine, KvsEngine, KvsError, Result};
use structopt::StructOpt;
use std::env::current_dir;
use std::fs;
use slog::{Drain, o, info, warn};
use std::io::Write;

fn get_current_engine(logger: &slog::Logger) -> Result<Option<Engine>> {
    // Check if config file exists and if it does not, return None
    let config_file = current_dir()?.join(".config");

    if !config_file.exists() {
        return Ok(None);
    }

    match fs::read_to_string(".config")?.parse() {
        Ok(engine) => Ok(Some(engine)),
        Err(e) => {
            warn!(logger, "The contents of the config file are invalid: {}", e);
            Ok(None)
        }
    }
}

fn main() -> Result<()> {
    // Setup logging
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!());

    // Store command line arguments in struct
    let opt = kvs::ServerOpt::from_args();

    // Check if choosen engine is different from the one previously saved in config file
    if let Some(current_engine) = get_current_engine(&log)? {
        if opt.engine != current_engine {
            return Err(KvsError::InvalidEngine(current_engine.to_string()));
        }
    }

    // Open engine config file and create it if it does not exist
    let mut config_file = fs::File::create(".config")?;

    // Write choosen engine to config file
    write!(&mut config_file, "{}", opt.engine)?;

    // Choose engine based on command line argument
    let engine: Box<dyn KvsEngine> = match opt.engine {
        Engine::Kvs => Box::new(kvs::KvStore::open("./logs")?),
        Engine::Sled => Box::new(kvs::SledKvsEngine::open("./logs")?)
    };

    // Setup KvsServer
    info!(log, "Using engine {}", opt.engine);
    let mut kvs_server = kvs::KvsServer::new(opt.addr, engine, log);

    kvs_server.run()?;

    Ok(())
}