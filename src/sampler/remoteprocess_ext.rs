pub type Pid = remoteprocess::Pid;
pub type Tid = remoteprocess::Tid;

use core::ffi::*;
use std::path::PathBuf;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Diagnostics::Debug::*;

use super::*;

pub trait ProcessExt {
    fn loaded_modules(&self) -> Vec<ModuleInfo>;
}

impl ProcessExt for remoteprocess::Process {
    fn loaded_modules(&self) -> Vec<ModuleInfo> {
        extern "system" fn callback(
            module_name: PCWSTR,
            base_address: u64,
            size: u32,
            modules_ptr: *const c_void,
        ) -> BOOL {
            if let Ok(module_name) = unsafe { module_name.to_string() } {
                let mut modules = unsafe { Box::from_raw(modules_ptr as *mut Vec<ModuleInfo>) };
                modules.push(ModuleInfo::new(
                    PathBuf::from(module_name),
                    base_address,
                    size,
                ));
                Box::leak(modules);
            }
            BOOL::from(true)
        }

        unsafe {
            let modules = Box::new(Vec::<ModuleInfo>::new());
            let modules_ptr = Box::into_raw(modules);
            _ = EnumerateLoadedModulesW64(
                HANDLE(*self.handle),
                Some(callback),
                Some(modules_ptr as *mut c_void),
            );
            *Box::from_raw(modules_ptr)
        }
    }
}
