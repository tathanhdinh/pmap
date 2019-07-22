use std::mem::{self, MaybeUninit};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use rayon::prelude::*;

use crate::{
    error::{AppResult, WindowsApiError},
    handle::Handle,
    process::Process,
    system::System,
};

use winapi::um::{
    memoryapi::VirtualQueryEx,
    winnt::{HANDLE, MEMORY_BASIC_INFORMATION, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION},
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

impl Region {
    fn get_region_bases(hdl: &Handle, sys_info: &System) -> AppResult<Box<[usize]>> {
        let mut region_bases = vec![];
        let mut next_base = sys_info.min_app_addr;
        let mut mem_info = MaybeUninit::uninit();
        while next_base < sys_info.max_app_addr {
            if unsafe {
                VirtualQueryEx(
                    hdl.into(),
                    next_base as _,
                    mem_info.as_mut_ptr(),
                    mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            } != mem::size_of::<MEMORY_BASIC_INFORMATION>()
            {
                return last_error!();
            }

            let mem_info = unsafe { mem_info.assume_init() };

            region_bases.push(mem_info.AllocationBase as usize);
            next_base = mem_info.AllocationBase as usize + mem_info.RegionSize;
        }

        Ok(region_bases.into_boxed_slice())
    }

    fn get_regions(region_bases: &[usize], hdl: &Handle) -> AppResult<Box<[Region]>> {
        let hdl = {
            let hdl: HANDLE = hdl.into();
            hdl as usize
        };

        let regions: AppResult<Vec<_>> = region_bases
            .into_par_iter()
            .map(|&region_base| {
                let mut mem_info = MaybeUninit::uninit();

                if unsafe {
                    VirtualQueryEx(
                        hdl as _,
                        region_base as _,
                        mem_info.as_mut_ptr(),
                        mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                    )
                } != mem::size_of::<MEMORY_BASIC_INFORMATION>()
                {
                    return last_error!();
                }

                let mem_info = unsafe { mem_info.assume_init() };
                Ok(Region {
                    base_addr: mem_info.AllocationBase as _,
                    size: mem_info.RegionSize,
                    map_type: FromPrimitive::from_u32(mem_info.Type).unwrap(),
                    com_state: FromPrimitive::from_u32(mem_info.State).unwrap(),
                })
            })
            .collect();

        regions.map(|rs| rs.into_boxed_slice())
    }

    pub fn scan(proc: &Process, sys_info: &System) -> AppResult<Box<[Region]>> {
        let proc_hdl = proc.open_with_flag(PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION)?;

        // sequential: look for all region bases
        let region_bases = Self::get_region_bases(&proc_hdl, sys_info)?;

        // concurent: scan regions
        Self::get_regions(&*region_bases, &proc_hdl)
    }
}
