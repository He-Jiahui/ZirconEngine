use std::env;
use std::fs;
use std::process::ExitCode;

use zircon_plugin_neural_editor::onnx::{convert_graph, read_onnx_graph};

fn main() -> ExitCode {
    let mut arguments = env::args().skip(1);
    let Some(input_path) = arguments.next() else {
        eprintln!("usage: zr_onnx_convert <input.onnx> <output.znn>");
        return ExitCode::from(2);
    };
    let Some(output_path) = arguments.next() else {
        eprintln!("usage: zr_onnx_convert <input.onnx> <output.znn>");
        return ExitCode::from(2);
    };
    if arguments.next().is_some() {
        eprintln!("usage: zr_onnx_convert <input.onnx> <output.znn>");
        return ExitCode::from(2);
    }

    let result = fs::read(&input_path)
        .map_err(|error| format!("failed to read ONNX input: {error}"))
        .and_then(|bytes| read_onnx_graph(&bytes).map_err(|error| error.to_string()))
        .and_then(|graph| match convert_graph(&graph) {
            Ok(model) => Ok(model),
            Err(diagnostics) => {
                for diagnostic in diagnostics {
                    eprintln!("{}", diagnostic.to_json_line());
                }
                Err("ONNX conversion failed".to_string())
            }
        })
        .and_then(|model| model.to_znn_bytes().map_err(|error| error.to_string()))
        .and_then(|bytes| fs::write(&output_path, bytes).map_err(|error| error.to_string()));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
