use super::*;

pub struct Symbolicator {
    process_handle: HANDLE,
}

pub struct SymbolicatedFrame {
    pub function: Option<String>,
    pub module: Option<String>,
}

impl Symbolicator {
    pub fn new(process_handle: HANDLE, search_path: &[&str]) -> Result<Self, Error> {
        unsafe {
            let mut path_string: Vec<u16> = search_path.join(";").encode_utf16().collect();
            path_string.push(0);
            SymInitializeW(process_handle, PCWSTR::from_raw(path_string.as_ptr()), true)
                .map_err(|e| Error::SymInitializeFailed(e))?
        };
        Ok(Self { process_handle })
    }

    pub fn symbolicate(&self, address: u64) -> SymbolicatedFrame {
        let function = unsafe {
            let mut displacement: u64 = 0;
            let mut symbol_info = SYMBOL_INFO_PACKAGEW::default();
            symbol_info.si.SizeOfStruct = size_of::<SYMBOL_INFOW>() as u32;
            symbol_info.si.MaxNameLen = MAX_SYM_NAME;

            if SymFromAddrW(
                self.process_handle,
                address,
                Some(&mut displacement),
                &mut symbol_info.si,
            )
            .is_ok()
            {
                PCWSTR::from_raw(symbol_info.si.Name.as_ptr())
                    .to_string()
                    .ok()
            } else {
                None
            }
        };

        let module = unsafe {
            let mut module_info = IMAGEHLP_MODULEW64::default();
            module_info.SizeOfStruct = size_of::<IMAGEHLP_MODULEW64>() as u32;

            if SymGetModuleInfoW64(self.process_handle, address, &mut module_info).is_ok() {
                PCWSTR::from_raw(module_info.ModuleName.as_ptr())
                    .to_string()
                    .ok()
            } else {
                None
            }
        };

        SymbolicatedFrame { function, module }
    }
}

impl Drop for Symbolicator {
    fn drop(&mut self) {
        unsafe {
            _ = SymCleanup(self.process_handle);
        }
    }
}
