use std::io::{self, Write};

use paw;
use structopt;
use tabwriter::TabWriter;

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

#[paw::main]
fn main(arg: Args) -> error::AppResult<()> {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut stdout = TabWriter::new(stdout);

    if arg.snapshot {
        let snapshot_hdl = ProcessesSnapshot::snapshot_handle()?;
        let executing_procs = ProcessesSnapshot::new(&snapshot_hdl)?;

        let mut proc_pretty_table = vec![];
        proc_pretty_table.push(format!(
            "PID\tPPID\tImage base\tImage size\tEntry point\tThreads\tPath"
        ));
        for executing_proc in executing_procs {
            proc_pretty_table.push(format!(
                "{}\t{}\t0x{:x}\t0x{:x}\t0x{:x}\t{}\t{}",
                executing_proc.proc.id,
                executing_proc.pid,
                executing_proc.proc.base_addr,
                executing_proc.proc.img_size,
                executing_proc.proc.entry_point,
                executing_proc.threads,
                executing_proc.proc.img_filepath
            ));
        }
        let proc_pretty_table = proc_pretty_table.join("\n");
        stdout.write_all(proc_pretty_table.as_bytes())?;
        stdout.flush()?;
    } else if let Some(pid) = arg.pid {
        let sys_info = system::System::global();
        let proc = process::Process::new(pid)?;
        let proc_mem_map = memory::ProcessMemoryMapping::new(&proc, sys_info)?;

        let mut mem_pretty_table = vec![];
        mem_pretty_table.push(format!("Address\tSize\tState\tType"));
        for region in proc_mem_map {
            mem_pretty_table.push(format!(
                "0x{:x}\t0x{:x}\t{}\t{}",
                region.base_addr, region.size, region.com_state, region.map_type
            ));
        }
        let mem_pretty_table = mem_pretty_table.join("\n");
        stdout.write_all(mem_pretty_table.as_bytes())?;
        stdout.flush()?;
    }

    Ok(())
}
