use pyo3::prelude::*;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::runtime::UiCommand;

#[pyclass(unsendable)]
#[derive(Clone)]
struct PythonBridge {
    sender: Arc<Mutex<Sender<UiCommand>>>,
}

#[pymethods]
impl PythonBridge {
    fn create_node(&self, node_type: String, styles: HashMap<String, String>, text: Option<String>) -> PyResult<()> {
        let cmd = UiCommand::CreateNode { node_type, styles, text };
        self.sender.lock().unwrap().send(cmd)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Bridge error: {}", e)))
    }

}

pub struct PythonRuntime {
    bridge: PythonBridge,
}

impl PythonRuntime {
    pub fn new(sender: Sender<UiCommand>) -> Self {
        Self {
            bridge: PythonBridge {
                sender: Arc::new(Mutex::new(sender)),
            }
        }
    }

    pub fn execute_script(&self, script: &str) -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let bridge_cell = Py::new(py, self.bridge.clone())?;
            let locals = pyo3::types::PyDict::new(py);
            locals.set_item("NativeUI", bridge_cell)?;

            py.run(script, None, Some(locals))?;
            Ok(())
        })
    }
}
