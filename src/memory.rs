use std::{
    marker::PhantomData,
    mem::{self, MaybeUninit},
};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use rayon::prelude::*;

use crate::{error::AppResult, handle::Handle, process::Process, system::System};

use winapi::um::{
    memoryapi::VirtualQueryEx,
    winnt::{MEMORY_BASIC_INFORMATION, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION},
};

#[derive(FromPrimitive)]
pub(crate) enum MemoryCommitState {
    Commit = 0x1000,
    Free = 0x10000,
    Reserve = 0x2000,
}

#[derive(FromPrimitive)]
pub(crate) enum MemoryMappingType {
    Image = 0x1000000,
    Mapped = 0x40000,
    Private = 0x20000,
}

pub(crate) struct Region {
    pub base_addr: usize,
    pub size: usize,
    pub map_type: MemoryMappingType,
    pub com_state: MemoryCommitState,
}

pub(crate) struct ProcessMemoryMapping<'a> {
    proc_hdl: Handle,
    current_region_base: usize,
    proc_min_addr: usize,
    proc_max_addr: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> ProcessMemoryMapping<'a> {
    pub fn new(proc: &'a Process, sys_info: &System) -> AppResult<Self> {
        let proc_hdl = proc.open_with_flag(PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION)?;

        Ok(Self {
            proc_hdl,
            current_region_base: sys_info.min_app_addr,
            proc_min_addr: sys_info.min_app_addr,
            proc_max_addr: sys_info.max_app_addr,
            _phantom: PhantomData,
        })
    }
}

impl Iterator for ProcessMemoryMapping<'_> {
    type Item = Region;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_region_base >= self.proc_max_addr {
            return None;
        }

        let mut mem_info = MaybeUninit::uninit();
        if unsafe {
            VirtualQueryEx(
                (&self.proc_hdl).into(),
                self.current_region_base as _,
                mem_info.as_mut_ptr(),
                mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        } != mem::size_of::<MEMORY_BASIC_INFORMATION>()
        {
            return None;
        }

        let mem_info = unsafe { mem_info.assume_init() };

        self.current_region_base = mem_info.AllocationBase as usize + mem_info.RegionSize;
        Some(Region {
            base_addr: mem_info.AllocationBase as _,
            size: mem_info.RegionSize,
            map_type: FromPrimitive::from_u32(mem_info.Type).unwrap(),
            com_state: FromPrimitive::from_u32(mem_info.State).unwrap(),
        })
    }
}
