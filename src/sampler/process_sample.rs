use super::*;

#[derive(Debug)]
pub struct ProcessSample {
    threads: Vec<ThreadSample>,
    symbol_table: SymbolTable,
    modules: Vec<ModuleInfo>,
}

impl ProcessSample {
    pub fn new(
        threads: Vec<ThreadSample>,
        symbol_table: SymbolTable,
        modules: Vec<ModuleInfo>,
    ) -> Self {
        Self {
            threads,
            symbol_table,
            modules,
        }
    }
}

impl std::fmt::Display for ProcessSample {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Process")?;

        writeln!(f)?;
        for thread in &self.threads {
            writeln!(f, "Thread {}", thread.get_thread_id())?;
            for sample_point in thread.sample_tree_dfs_iter() {
                let symbol = self.symbol_table.symbol(sample_point.get_address());

                let function_name = symbol
                    .map(SymbolInfo::get_function)
                    .flatten()
                    .unwrap_or("{unknown}");
                let module_name = symbol
                    .map(SymbolInfo::get_module_name)
                    .flatten()
                    .unwrap_or("{unknown}");

                writeln!(
                    f,
                    "{}{} - {}  (in {})  [{:#x}]",
                    " ".repeat(sample_point.get_level() as usize),
                    sample_point.get_count(),
                    function_name,
                    module_name,
                    sample_point.get_address()
                )?;
            }
        }

        writeln!(f)?;

        writeln!(f, "Modules:")?;
        for module in &self.modules {
            writeln!(
                f,
                "  {:#x} - {:#x}  {:24} {}",
                module.address_range().start,
                module.address_range().end,
                module.name().unwrap_or("{unknown}"),
                module.file_path().unwrap_or("")
            )?;
        }

        Ok(())
    }
}
