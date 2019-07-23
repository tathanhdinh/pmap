use once_cell::sync::OnceCell;
use paw;
use structopt;

#[macro_use]
mod error;
mod handle;
mod memory;
mod process;
mod system;
mod utils;

use process::ProcessesSnapshot;

#[derive(structopt::StructOpt)]
struct Args {
    #[structopt(short = "l", long = "list", help = "show system processes")]
    snapshot: bool,

    #[structopt(
        short = "p",
        long = "pid",
        help = "show memory map of process with PID"
    )]
    pid: Option<u32>,
}

static SYS_INFO: OnceCell<system::System> = OnceCell::new();

#[paw::main]
fn main(arg: Args) -> error::AppResult<()> {
    if arg.snapshot {
        let snapshot_hdl = ProcessesSnapshot::snapshot_handle()?;
        let processes = ProcessesSnapshot::new(&snapshot_hdl);
        for proc in processes {}
    } else if let Some(pid) = arg.pid {

    }

    Ok(())
}
