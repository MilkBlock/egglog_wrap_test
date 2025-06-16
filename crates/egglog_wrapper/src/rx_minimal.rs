use crate::{collect_string_type_defs, wrap::*};
use egglog::{EGraph, SerializeConfig, ast::Command};
use std::{path::PathBuf, sync::Mutex};

pub struct RxMinimal {
    egraph: Mutex<EGraph>,
}

/// Rx with miminal feature (only new function is supported)
impl RxMinimal {
    pub fn new_with_string_type_defs(type_defs: String) -> Self {
        Self {
            egraph: Mutex::new({
                let mut e = EGraph::default();
                log::info!("{}", type_defs);
                e.parse_and_run_program(None, type_defs.as_ref()).unwrap();
                e
            }),
        }
    }
    pub fn new_with_type_defs(commands: Vec<Command>) -> Self {
        Self {
            egraph: Mutex::new({
                let mut e = EGraph::default();
                e.run_program(commands).unwrap();
                e
            }),
        }
    }
    pub fn new() -> Self {
        Self::new_with_string_type_defs(collect_string_type_defs())
    }
    pub fn interpret(&self, s: String) {
        let mut egraph = self.egraph.lock().unwrap();
        egraph.parse_and_run_program(None, s.as_str()).unwrap();
    }
    pub fn to_dot(&self, file_name: PathBuf) {
        let egraph = self.egraph.lock().unwrap();
        let serialized = egraph.serialize(SerializeConfig::default());
        let dot_path = file_name.with_extension("dot");
        serialized
            .to_dot_file(dot_path.clone())
            .unwrap_or_else(|_| panic!("Failed to write dot file to {dot_path:?}"));
    }
}

unsafe impl Send for RxMinimal {}
unsafe impl Sync for RxMinimal {}
// MARK: Receiver
impl Rx for RxMinimal {
    fn receive(&self, received: RxCommand) {
        log::info!("{:?}", received);
        match received {
            RxCommand::StringCommand { string_command } => {
                self.interpret(string_command);
            }
            RxCommand::NativeCommand { native_command } => todo!(),
        }
    }

    fn on_new(&self, node: &(impl crate::wrap::EgglogNode + 'static)) {
        self.interpret(node.to_egglog());
    }

    fn on_set(&self, _node: &mut (impl crate::wrap::EgglogNode + 'static)) {
        panic!("set is unsupported for rx_minimal")
    }

    fn on_func_set<'a, F: EgglogFunc>(
        &self,
        input: <F::Input as EgglogFuncInputs>::Ref<'a>,
        output: <F::Output as crate::wrap::EgglogFuncOutput>::Ref<'a>,
    ) {
        let input_nodes = input.as_nodes();
        let mut input_syms = input_nodes.iter().map(|x| x.cur_sym());
        let output = output.as_node().cur_sym();
        self.receive(RxCommand::StringCommand {
            string_command: format!(
                "(set ({} {}) {} )",
                F::FUNC_NAME,
                input_syms.map(|x| x.as_str()).collect::<String>(),
                output
            ),
        });
    }
}
