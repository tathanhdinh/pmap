use winapi::um::sysinfoapi::GetSystemInfo;

use std::mem::MaybeUninit;

pub(crate) struct System {
    pub min_app_addr: usize,
    pub max_app_addr: usize,
    pub page_size: u32,
    pub alloc_granularity: u32,
}

impl System {
    pub fn new() -> Self {
        let sys_info = {
            let mut sys_info = MaybeUninit::uninit();
            unsafe {
                GetSystemInfo(sys_info.as_mut_ptr());
                sys_info.assume_init()
            }
        };

        System {
            min_app_addr: sys_info.lpMinimumApplicationAddress as _,
            max_app_addr: sys_info.lpMaximumApplicationAddress as _,
            page_size: sys_info.dwPageSize,
            alloc_granularity: sys_info.dwAllocationGranularity,
        }
    }
}
