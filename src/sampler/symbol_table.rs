use std::path::PathBuf;

use super::*;

#[derive(Debug)]
pub struct SymbolTable {
    address_to_symbol_table: std::collections::HashMap<u64, SymbolInfo>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    function: Option<String>,
    module: Option<PathBuf>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            address_to_symbol_table: std::collections::HashMap::new(),
        }
    }

    pub fn symbolicate(&mut self, backtrace: &Vec<u64>, symbolicator: &Symbolicator) {
        for address in backtrace {
            if !self.address_to_symbol_table.contains_key(address) {
                let symbolicated_frame = symbolicator.symbolicate(*address);
                self.address_to_symbol_table.insert(
                    *address,
                    SymbolInfo {
                        function: symbolicated_frame.function,
                        module: symbolicated_frame.module.map(PathBuf::from),
                    },
                );
            }
        }
    }

    pub fn symbol(&self, address: u64) -> Option<&SymbolInfo> {
        self.address_to_symbol_table.get(&address)
    }
}

impl SymbolInfo {
    pub fn get_function(&self) -> Option<&str> {
        self.function.as_ref().map(String::as_str)
    }

    pub fn get_module_name(&self) -> Option<&str> {
        self.module
            .as_ref()
            .map(|f| f.file_name().map(std::ffi::OsStr::to_str).flatten())
            .flatten()
    }
}
