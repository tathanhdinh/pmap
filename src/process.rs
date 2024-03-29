use std::{
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr,
};

use winapi::{
    shared::minwindef::{DWORD, FALSE, HMODULE, MAX_PATH, TRUE},
    um::{
        handleapi::INVALID_HANDLE_VALUE,
        processthreadsapi::OpenProcess,
        psapi::{
            EnumProcessModulesEx, GetModuleFileNameExW, GetModuleInformation, LIST_MODULES_ALL,
            MODULEINFO,
        },
        tlhelp32::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
        winnt::{HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
};

use crate::{
    error::{AppError, AppResult},
    handle::Handle,
    utils,
};

pub(crate) struct Process {
    pub id: u32,
    pub img_filepath: String,
    pub base_addr: usize,
    pub entry_point: usize,
    pub img_size: usize,
}

struct LoadedModule {
    pub base_addr: usize,
    pub entry_point: usize,
    pub img_size: usize,
}

impl Process {
    fn get_image_file_path(hdl: &Handle) -> AppResult<String> {
        let mut img_name = MaybeUninit::<[u16; MAX_PATH]>::uninit();
        let img_name_len = unsafe {
            GetModuleFileNameExW(
                hdl.into(),
                ptr::null_mut(),
                img_name.as_mut_ptr() as _,
                MAX_PATH as _,
            )
        };
        if img_name_len == 0 {
            return last_error!();
        }
        let img_name = unsafe { img_name.assume_init() };
        Ok(utils::string_from_wide(&img_name[0..img_name_len as _]))
    }

    fn get_self_as_loaded_module(hdl: &Handle, self_filepath: &str) -> AppResult<LoadedModule> {
        let mut mods = MaybeUninit::<[HMODULE; 1024]>::uninit();
        let mut mc = MaybeUninit::uninit();
        let modules = {
            if unsafe {
                EnumProcessModulesEx(
                    hdl.into(),
                    mods.as_mut_ptr() as _,
                    1024,
                    mc.as_mut_ptr() as _,
                    LIST_MODULES_ALL,
                )
            } != TRUE
            {
                return last_error!();
            }

            unsafe { mods.assume_init() }
        };

        let mc = unsafe { mc.assume_init() };

        let modules = &modules[0..(mc / mem::size_of::<HMODULE>() as DWORD) as _];

        let mut base_name = [0u16; MAX_PATH];
        let mut module_info = MaybeUninit::uninit();
        // iterate over loaded modules, hopefully find the process image
        for &m in modules {
            let base_name_len = unsafe {
                GetModuleFileNameExW(hdl.into(), m, &mut base_name as *mut _ as _, MAX_PATH as _)
            };

            if base_name_len == 0 {
                continue;
            }

            let m_filepath = utils::string_from_wide(&base_name[0..base_name_len as _]);
            if m_filepath != self_filepath {
                continue;
            }

            if unsafe {
                GetModuleInformation(
                    hdl.into(),
                    m,
                    module_info.as_mut_ptr(),
                    mem::size_of::<MODULEINFO>() as _,
                )
            } != TRUE
            {
                return last_error!();
            }

            // ok, found
            let module_info = unsafe { module_info.assume_init() };
            return Ok(LoadedModule {
                base_addr: module_info.lpBaseOfDll as _,
                img_size: module_info.SizeOfImage as _,
                entry_point: module_info.EntryPoint as _,
            });
        }

        // not found, what to do?
        last_error!()
    }

    pub fn new(id: u32) -> AppResult<Self> {
        let hdl = Handle::new(unsafe {
            OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, id)
        })?;

        // get image file path
        let img_filepath = Self::get_image_file_path(&hdl)?;

        // get base address, entry point, etc.
        let self_mod = Self::get_self_as_loaded_module(&hdl, img_filepath.as_str())?;

        Ok(Process {
            id,
            img_filepath,
            base_addr: self_mod.base_addr,
            img_size: self_mod.img_size,
            entry_point: self_mod.entry_point,
        })
    }

    pub fn open_with_flag(&self, flag: u32) -> AppResult<Handle> {
        Handle::new(unsafe { OpenProcess(flag, FALSE, self.id) })
    }
}

pub(crate) struct ExecutingProcess<'a> {
    pub proc: Process,
    pub pid: u32,
    pub threads: u32,
    _phantom: PhantomData<&'a ()>,
}

pub(crate) struct ProcessesSnapshot<'a> {
    snapshot_hdl: HANDLE,
    current_proc_entry: Option<PROCESSENTRY32W>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> ProcessesSnapshot<'a> {
    pub fn snapshot_handle() -> AppResult<Handle> {
        let hdl = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if hdl == INVALID_HANDLE_VALUE {
            last_error!()
        } else {
            Ok(Handle::new(hdl)?)
        }
    }

    pub fn new(snapshot_hdl: &'a Handle) -> AppResult<Self> {
        Ok(ProcessesSnapshot {
            snapshot_hdl: snapshot_hdl.into(),
            current_proc_entry: None,
            _phantom: PhantomData,
        })
    }
}

impl<'a> Iterator for ProcessesSnapshot<'a> {
    type Item = ExecutingProcess<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut proc_entry = unsafe { MaybeUninit::<PROCESSENTRY32W>::zeroed().assume_init() };
        proc_entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

        loop {
            let has_next = if self.current_proc_entry.is_none() {
                unsafe { Process32FirstW(self.snapshot_hdl, &mut proc_entry) }
            } else {
                unsafe { Process32NextW(self.snapshot_hdl, &mut proc_entry) }
            };

            if has_next != TRUE {
                return None;
            }

            self.current_proc_entry = Some(proc_entry);

            if let Ok(proc) = Process::new(proc_entry.th32ProcessID) {
                return Some(ExecutingProcess {
                    proc,
                    pid: proc_entry.th32ParentProcessID,
                    threads: proc_entry.cntThreads,
                    _phantom: PhantomData,
                });
            }
        }
    }
}
