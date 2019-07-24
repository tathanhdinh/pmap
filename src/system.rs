use std::mem::MaybeUninit;

use once_cell::sync::OnceCell;

use winapi::um::sysinfoapi::GetSystemInfo;

pub(crate) struct System {
    pub min_app_addr: usize,
    pub max_app_addr: usize,
    pub page_size: u32,
    pub alloc_granularity: u32,
}

static SYS_INFO: OnceCell<System> = OnceCell::new();

impl System {
    pub fn global() -> &'static Self {
        if let Some(sys_info) = SYS_INFO.get() {
            return sys_info;
        }

        let sys_info = {
            let mut sys_info = MaybeUninit::uninit();
            unsafe {
                GetSystemInfo(sys_info.as_mut_ptr());
                sys_info.assume_init()
            }
        };

        let _ = SYS_INFO.set(System {
            min_app_addr: sys_info.lpMinimumApplicationAddress as _,
            max_app_addr: sys_info.lpMaximumApplicationAddress as _,
            page_size: sys_info.dwPageSize,
            alloc_granularity: sys_info.dwAllocationGranularity,
        });

        SYS_INFO.get().unwrap()
    }
}
