use super::*;

#[derive(Debug)]
pub struct ProcessSample {
    threads: Vec<ThreadSample>,
    symbol_table: SymbolTable,
}

impl ProcessSample {
    pub fn new(threads: Vec<ThreadSample>, symbol_table: SymbolTable) -> Self {
        Self {
            threads,
            symbol_table: symbol_table,
        }
    }
}

impl std::fmt::Display for ProcessSample {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Process")?;
        for thread in &self.threads {
            writeln!(f, "Thread {}", thread.get_thread_id())?;
            for node in thread.get_root_node().get_children() {
                self.write_node(node, f)?;
            }
        }
        Ok(())
    }
}

impl ProcessSample {
    fn write_node(&self, node: &SampleNode, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let function_name = self
            .symbol_table
            .symbol(node.get_address())
            .map(SymbolInfo::get_function)
            .flatten()
            .map(String::as_str)
            .unwrap_or("{unknown}");

        writeln!(
            f,
            "{}{} - {} at {}",
            " ".repeat(node.get_level() as usize),
            node.get_count(),
            function_name,
            node.get_address()
        )?;
        for node in node.get_children() {
            self.write_node(node, f)?;
        }
        Ok(())
    }
}
