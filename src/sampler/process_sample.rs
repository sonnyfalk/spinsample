use super::*;

#[derive(Debug)]
pub struct ProcessSample {
    process_info: ProcessInfo,
    threads: Vec<ThreadSample>,
    symbol_table: SymbolTable,
}

impl ProcessSample {
    pub fn new(
        process_info: ProcessInfo,
        threads: Vec<ThreadSample>,
        symbol_table: SymbolTable,
    ) -> Self {
        Self {
            process_info,
            threads,
            symbol_table,
        }
    }
}

impl std::fmt::Display for ProcessSample {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "Process: {} - {}",
            self.process_info.pid,
            self.process_info.path.to_string_lossy()
        )?;

        let user_cpu_time = self.process_info.user_cpu_time;
        let kernel_cpu_time = self.process_info.kernel_cpu_time;
        let total_cpu_time = user_cpu_time + kernel_cpu_time;
        writeln!(
            f,
            "  CPU Time: {:.3}s (user: {:.3}s, kernel: {:.3}s)",
            total_cpu_time.as_secs_f64(),
            user_cpu_time.as_secs_f64(),
            kernel_cpu_time.as_secs_f64()
        )?;

        writeln!(f)?;
        for thread in &self.threads {
            let user_cpu_time = thread.get_user_cpu_time();
            let kernel_cpu_time = thread.get_kernel_cpu_time();
            let total_cpu_time = user_cpu_time + kernel_cpu_time;

            writeln!(
                f,
                "Thread {}    CPU Time: {:.3}s (user: {:.3}s, kernel: {:.3}s)",
                thread.get_thread_id(),
                total_cpu_time.as_secs_f64(),
                user_cpu_time.as_secs_f64(),
                kernel_cpu_time.as_secs_f64()
            )?;
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
                    " {}{} - {}  (in {})  [{:#x}]",
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
        for module in &self.process_info.modules {
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
