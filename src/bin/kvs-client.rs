use kvs::KvsClient;
use kvs::Result;
use structopt::StructOpt;

use slog::Drain;
use slog::o;

fn main() -> Result<()> {
    // Setup logging
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!());

    // Store command line arguments in struct
    let opt = kvs::ClientOpt::from_args();

    // Setup KvsClient
    let kvs_client = KvsClient::new(opt.addr, log);

    // Run KvsClient
    kvs_client.run(opt.command)?;

    Ok(())
}
