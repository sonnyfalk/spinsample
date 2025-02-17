#[derive(Debug)]
pub struct SymbolTable {
    address_to_symbol_table: std::collections::HashMap<u64, SymbolInfo>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    function: Option<String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            address_to_symbol_table: std::collections::HashMap::new(),
        }
    }

    pub fn symbolicate(
        &mut self,
        backtrace: &Vec<u64>,
        symbolicator: &remoteprocess::Symbolicator,
    ) {
        for address in backtrace {
            if !self.address_to_symbol_table.contains_key(address) {
                _ = symbolicator.symbolicate(*address, false, &mut |sf| {
                    self.address_to_symbol_table.insert(
                        *address,
                        SymbolInfo {
                            function: sf.function.clone(),
                        },
                    );
                });
            }
        }
    }

    pub fn symbol(&self, address: u64) -> Option<&SymbolInfo> {
        self.address_to_symbol_table.get(&address)
    }
}

impl SymbolInfo {
    pub fn get_function(&self) -> Option<&String> {
        self.function.as_ref()
    }
}
