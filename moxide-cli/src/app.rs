use std::{process::exit, str::FromStr};

use color_eyre::eyre::{eyre, Context, Result};
use moxide::{
    process::{enum_proc, Process},
    scanner::{self, BasicWriter, Pattern, Region, Scannable, Ty},
};
struct Scanner {
    process: Process,
    result: Vec<Region>,
}

impl Scanner {
    fn scan<T: Scannable>(&mut self, pattern: &Pattern<T>) -> Result<usize> {
        let memory_regions = self.process.memory_regions();
        self.result = memory_regions
            .iter()
            .flat_map(|region| {
                match self
                    .process
                    .read_memory(region.BaseAddress as _, region.RegionSize)
                {
                    Ok(bytes) => Some(pattern.run(region.clone(), bytes)),
                    Err(_) => None, // This happens, for no reason
                }
            })
            .collect();
        Ok(self.result.iter().map(|r| r.count()).sum())
    }
    fn rescan<T: Scannable>(&mut self, pattern: &Pattern<T>) -> Result<usize> {
        self.result = self
            .result
            .iter()
            .flat_map(|region| {
                match self
                    .process
                    .read_memory(region.info.BaseAddress as _, region.info.RegionSize)
                {
                    Ok(bytes) => Some(pattern.rerun(&region, bytes)),
                    Err(_) => None,
                }
            })
            .collect();
        Ok(self.result.iter().map(|r| r.count()).sum())
    }
}

pub struct App {
    attached_process: Option<Process>,
    scanner: Option<Scanner>,
    writer: BasicWriter,
}

impl App {
    pub fn new() -> App {
        App {
            attached_process: None,
            scanner: None,
            writer: BasicWriter::new(),
        }
    }
    pub fn handle_input(&mut self, input: &str) -> Result<String> {
        tracing::debug!("Handling input: {}", input);
        let args = input.split(' ').collect::<Vec<&str>>();
        match args[0] {
            "exit" | "quit" => exit(0),
            "attach" | "a" => self.attach_to_process(args.get(1)),
            "run" | "r" => self.new_scan(args.get(1)),
            "next" | "n" => self.next_scan(args.get(1)),
            "list" | "l" => self.list_results(args.get(1)),
            "write" | "w" => self.write(args.get(1), args.get(2)),
            "ps" => list_processes(),
            "help" | "h" => print_help(),
            _ => Ok("Unknown command".to_owned()),
        }
    }
    fn attach_to_process(&mut self, pid: Option<&&str>) -> Result<String> {
        let pid = pid.ok_or(eyre!("No PID provided"))?.parse::<u32>()?;
        self.cleanup();
        let process = Process::open(pid)?;
        let name = process.name()?;
        self.attached_process = Some(process);
        Ok(format!("Attached to process: {}", name))
    }
    fn cleanup(&mut self) {
        self.attached_process = None;
    }

    fn new_scan(&mut self, pattern: Option<&&str>) -> Result<String> {
        let process = self
            .attached_process
            .as_ref()
            .ok_or(eyre!("No process attached"))?;
        let pattern = pattern.ok_or(eyre!("No pattern provided"))?;
        let pattern = Pattern::from_str(pattern)?;
        self.scanner = Some(Scanner {
            process: Process::open(process.pid())?,
            result: Vec::new(),
        });
        let scanner = self.scanner.as_mut().unwrap();
        let count = scanner.scan(&pattern)?;
        Ok(format!("Scan complete. {} results found", count).to_owned())
    }
    fn next_scan(&mut self, pattern: Option<&&str>) -> Result<String> {
        let pattern = pattern.ok_or(eyre!("No pattern provided"))?;
        let pattern = Pattern::from_str(pattern)?;
        let scanner = self.scanner.as_mut().ok_or(eyre!("No previous scan"))?;
        let count = scanner.rescan(&pattern)?;
        Ok(format!("Scan complete. {} results left.", count).to_owned())
    }
    fn list_results(&self, ty: Option<&&str>) -> Result<String> {
        let ty = ty.ok_or(eyre!("No type provided"))?;
        let ty = Ty::from_str(ty)?.default();
        let scanner = self.scanner.as_ref().ok_or(eyre!("Not scanning"))?;
        let result = &scanner.result;
        let list = result
            .iter()
            .flat_map(|r| r.to_list(&ty))
            .collect::<Vec<_>>();
        let output = list
            .iter()
            .map(|r| format!("{:<16x}: {}", r.0, r.1))
            .collect::<Vec<String>>()
            .join("\n");
        Ok(output)
    }
    fn write(&self, value: Option<&&str>, address: Option<&&str>) -> Result<String> {
        let process = self
            .attached_process
            .as_ref()
            .ok_or(eyre!("No process attached"))?;
        let scanner = self.scanner.as_ref().ok_or(eyre!("No scan results"))?;
        let value = Pattern::from_str(value.ok_or(eyre!("No value provided"))?)?;
        match value {
            Pattern::Exact(v) => {
                if let Some(address) = address {
                    let address = usize::from_str_radix(address, 16)?;
                    self.writer.write(process, address, &v)?;
                    return Ok("Write successful".to_owned());
                } else {
                    // Write to all results
                    scanner.result.iter().try_for_each(|region| {
                        region
                            .to_list(&v)
                            .iter()
                            .try_for_each(|r| self.writer.write(process, r.0, &v).map(|_| ()))
                    })?;
                }
                Ok("Write successful".to_owned())
            }
            _ => Err(eyre!("Cannot write non-exact value")),
        }
    }
}

fn list_processes() -> Result<String> {
    let pids = enum_proc()?;
    let processes = pids
        .iter()
        .filter_map(|pid| Process::open(*pid).ok())
        .filter(|p| p.name().is_ok())
        .collect::<Vec<Process>>();
    let output = processes
        .iter()
        .map(|p| format!("{}:{}", p.pid(), p.name().unwrap()))
        .collect::<Vec<String>>()
        .join("\n");
    Ok(output)
}

fn print_help() -> Result<String> {
    Ok("Commands:
        attach,a <pid> - Attach to a process
        run,r <pattern> - Run a scan
        next,n <pattern> - Run a scan with the previous results
        list,l <type> - List the results of the last scan
        write,w <value> [address] - Write a value to an address
        ps - List all processes
        help,h - Print this help message
        exit - Exit the program"
        .to_owned())
}
