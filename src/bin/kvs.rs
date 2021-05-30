use kvs::Result;
use structopt::StructOpt;
use std::env::current_dir;

fn main() -> Result<()> {
    let mut kvs_store = kvs::KvStore::open(current_dir()?)?;
    let opt = kvs::Opt::from_args();
    kvs::run(opt.command, &mut kvs_store)?;

    Ok(())
}
