use std::process::exit;

use color_eyre::eyre::{eyre, Result};
use moxide::{
    process::{enum_proc, Process},
    scanner::{
        BasicScanPattern, BasicScanner, ListScanResult, ScanConfig, ScanResult, ScanType, Scanner, Writer,
    },
};

pub struct App {
    attached_process: Option<Process>,
    config: ScanConfig,
    scanner: BasicScanner,
    writer: Writer,
    result: Option<ListScanResult>,
}

impl App {
    pub fn new() -> App {
        App {
            attached_process: None,
            scanner: BasicScanner {},
            result: None,
            config: ScanConfig::default(),
            writer: Writer::new()
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
            "list" | "l" => self.list_results(),
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
        let pattern = parse_pattern(pattern)?;
        self.config.width = pattern.1;
        let result = self.scanner.new_scan(process, &self.config, &pattern.0);
        let count = result.count();
        self.result = Some(result);
        Ok(format!("Scan complete. {} results found", count).to_owned())
    }
    fn next_scan(&mut self, pattern: Option<&&str>) -> Result<String> {
        let process = self
            .attached_process
            .as_ref()
            .ok_or(eyre!("No process attached"))?;
        let pattern = pattern.ok_or(eyre!("No pattern provided"))?;
        let pattern = parse_pattern(pattern)?.0;
        let result = self.result.as_mut().ok_or(eyre!("Not in a scan"))?;
        self.scanner
            .next_scan(process, &self.config, &pattern, result);
        let count = result.count();
        Ok(format!("Scan complete. {} results left.", count).to_owned())
    }
    fn list_results(&self) -> Result<String> {
        let result = self.result.as_ref().ok_or(eyre!("No scan results"))?;
        let list = result.to_list();
        let output = list
            .iter()
            .map(|r| format!("{:<16x}: {}", r.address, r.value))
            .collect::<Vec<String>>()
            .join("\n");
        Ok(output)
    }
    fn write(&self, value: Option<&&str>, address: Option<&&str>) -> Result<String> {
        let process = self
            .attached_process
            .as_ref()
            .ok_or(eyre!("No process attached"))?;
        let value = ScanType::from_str(value.ok_or(eyre!("No value provided"))?)?;
        if let Some(address) = address {
            let address = usize::from_str_radix(address, 16)?;
            self.writer.write(process, address, &value)?;
            return Ok("Write successful".to_owned());
        } else {
            // Write to all results
            let result = self.result.as_ref().ok_or(eyre!("No scan results"))?;
            result.to_list().iter().try_for_each(|r| {
                self.writer.write(process, r.address, &value).map(|_|())
            })?
        }
        Ok("Write successful".to_owned())
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

fn parse_pattern(pattern: &str) -> Result<(BasicScanPattern, usize)> {
    let pattern = pattern.trim();
    let parts:Vec<_> = pattern.split(':').collect();
    if parts.len() != 2 {
        return Err(eyre!("There are multiple : in the pattern"));
    }
    let operator = parts[0];
    let values = parts[1].split(',');
    let data_types = values.map(ScanType::from_str).collect::<Result<Vec<_>>>()?;
    // Anyway an arg0 must be provided. It determines the width of the pattern
    let arg0 = data_types.get(0).map(|v|v.clone()).ok_or(eyre!("Not enough values provided"))?;
    // arg1 is optional
    let arg1 = data_types.get(1).map(|v|v.clone()).ok_or(eyre!("Not enough values provided"));
    let pattern = match operator {
        "=" => Ok(BasicScanPattern::Exact(arg0)),
        ">=" => Ok(BasicScanPattern::GreaterOrEqualThan(arg0)),
        "<=" => Ok(BasicScanPattern::LessOrEqualThan(arg0)),
        "b" => Ok(BasicScanPattern::Between(arg0, arg1?)),
        "+" => Ok(BasicScanPattern::Increased(arg0)),
        "+=" => Ok(BasicScanPattern::IncreasedBy(arg0)),
        "+>=" => Ok(BasicScanPattern::IncreasedAtLeast(arg0)),
        "+<=" => Ok(BasicScanPattern::IncreasedAtMost(arg0)),
        "-" => Ok(BasicScanPattern::Decreased(arg0)),
        "-=" => Ok(BasicScanPattern::DecreasedBy(arg0)),
        "->=" => Ok(BasicScanPattern::DecreasedAtLeast(arg0)),
        "-<=" => Ok(BasicScanPattern::DecreasedAtMost(arg0)),
        "c" => Ok(BasicScanPattern::Changed(arg0)),
        "u" => Ok(BasicScanPattern::Unchanged(arg0)),
        "?" => Ok(BasicScanPattern::Unknown(arg0)),
        _ => Err(eyre!("Invalid pattern")),
    };
    Ok((pattern?, arg0.width()))
}

fn print_help() -> Result<String> {
    Ok(
        "Commands:
        attach <pid> - Attach to a process
        run <pattern> - Run a scan
        next <pattern> - Run a scan with the previous results
        list - List the results of the last scan
        write <value> [address] - Write a value to an address
        ps - List all processes
        help - Print this help message
        exit - Exit the program"
            .to_owned(),
    )
}
