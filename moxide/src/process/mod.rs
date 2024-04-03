use color_eyre::eyre::{Result, WrapErr};
use winapi::um::winnt;
use std::io;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::HMODULE;
use winapi::shared::minwindef::{DWORD, FALSE};

pub struct Process {
    pid: u32,
    handle: NonNull<c_void>,
}

impl Process {
    pub fn open(pid: u32) -> Result<Self> {
        // SAFETY: the call doesn't have dangerous side-effects.
        NonNull::new(unsafe {
            winapi::um::processthreadsapi::OpenProcess(
                winnt::PROCESS_VM_READ
                    | winnt::PROCESS_QUERY_INFORMATION
                    | winnt::PROCESS_VM_WRITE
                    | winnt::PROCESS_VM_OPERATION,
                FALSE,
                pid,
            )
        })
        .map(|handle| Self { pid, handle })
        .ok_or_else(io::Error::last_os_error)
        .wrap_err(format!("Fail to open process {pid}"))
    }
    pub fn name(&self) -> Result<String> {
        let mut module = MaybeUninit::<HMODULE>::uninit();
        let mut size = 0;
        // SAFETY: the pointer is valid and the size is correct.
        if unsafe {
            winapi::um::psapi::EnumProcessModules(
                self.handle.as_ptr(),
                module.as_mut_ptr(),
                mem::size_of::<HMODULE>() as u32,
                &mut size,
            )
        } == FALSE
        {
            return Err(io::Error::last_os_error()).wrap_err("EnumProcessModules failed");
        }

        // SAFETY: the call succeeded, so module is initialized.
        let module = unsafe { module.assume_init() };
        let mut buffer = Vec::<u16>::with_capacity(64);
        // SAFETY: the handle, module and buffer are all valid.
        let length = unsafe {
            winapi::um::psapi::GetModuleBaseNameW(
                self.handle.as_ptr(),
                module,
                buffer.as_mut_ptr(),
                buffer.capacity() as u32,
            )
        };
        if length == 0 {
            return Err(io::Error::last_os_error()).wrap_err("GetModuleBaseNameW failed");
        }

        // SAFETY: the call succeeded and length represents bytes.
        unsafe { buffer.set_len(length as usize) };
        Ok(String::from_utf16(&buffer).unwrap())
    }
    pub fn pid(&self) -> u32 {
        self.pid
    }
    pub fn read_memory(&self, addr: usize, n: usize) -> Result<Vec<u8>> {
        let mut buffer = Vec::<u8>::with_capacity(n);
        let mut read = 0;

        // SAFETY: the buffer points to valid memory, and the buffer size is correctly set.
        if unsafe {
            winapi::um::memoryapi::ReadProcessMemory(
                self.handle.as_ptr(),
                addr as *const _,
                buffer.as_mut_ptr().cast(),
                buffer.capacity(),
                &mut read,
            )
        } == FALSE
        {
            Err(io::Error::last_os_error()).wrap_err("ReadProcessMemory failed")
        } else {
            // SAFETY: the call succeeded and `read` contains the amount of bytes written.
            unsafe { buffer.set_len(read as usize) };
            Ok(buffer)
        }
    }
    pub fn write_memory(&self, addr: usize, data: &[u8]) -> Result<usize> {
        let mut written = 0;
        // SAFETY: the input value buffer points to valid memory.
        if unsafe {
            winapi::um::memoryapi::WriteProcessMemory(
                self.handle.as_ptr(),
                addr as *mut _,
                data.as_ptr().cast(),
                data.len(),
                &mut written,
            )
        } == FALSE
        {
            Err(io::Error::last_os_error()).wrap_err("WriteProcessMemory failed")
        } else {
            Ok(written)
        }
    }
    pub fn memory_regions(&self) -> Vec<winapi::um::winnt::MEMORY_BASIC_INFORMATION> {
        let mut base = 0;
        let mut regions = Vec::new();
        let mut info = MaybeUninit::uninit();

        loop {
            // SAFETY: the info structure points to valid memory.
            let written = unsafe {
                winapi::um::memoryapi::VirtualQueryEx(
                    self.handle.as_ptr(),
                    base as *const _,
                    info.as_mut_ptr(),
                    mem::size_of::<winapi::um::winnt::MEMORY_BASIC_INFORMATION>(),
                )
            };
            if written == 0 {
                break regions;
            }
            // SAFETY: a non-zero amount was written to the structure
            let info = unsafe { info.assume_init() };
            base = info.BaseAddress as usize + info.RegionSize;
            regions.push(info);
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        assert!(unsafe { winapi::um::handleapi::CloseHandle(self.handle.as_mut()) } != FALSE);
    }
}

pub fn enum_proc() -> Result<Vec<u32>> {
    // On my linux server, cat /proc/sys/kernel/pid_max yields 4194304. But even a long running linux server seldom holds more than 1024 processes. This should be good.
    let mut pids = Vec::<DWORD>::with_capacity(1024);
    let mut size = 0;
    // SAFETY: the pointer is valid and the size matches the capacity.
    if unsafe {
        winapi::um::psapi::EnumProcesses(
            pids.as_mut_ptr(),
            (pids.capacity() * mem::size_of::<DWORD>()) as u32,
            &mut size,
        )
    } == FALSE
    {
        return Err(io::Error::last_os_error()).wrap_err("EnumProcesses failed");
    }

    let count = size as usize / mem::size_of::<DWORD>();
    // SAFETY: the call succeeded and count equals the right amount of items.
    unsafe { pids.set_len(count) };
    pids.sort();
    Ok(pids)
}

pub fn get_process_by_name(name: &str) -> Option<Process> {
    enum_proc()
        .unwrap()
        .into_iter()
        .filter_map(|pid| Process::open(pid).ok())
        .find(|p| p.name().ok().as_deref() == Some(name))
}
