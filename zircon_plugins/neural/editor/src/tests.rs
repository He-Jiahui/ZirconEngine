use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::onnx::{convert_graph, OnnxAttribute, OnnxGraph, OnnxNode, OnnxTensor};
use zircon_editor::core::asset::AssetTypeId;
use zircon_editor::core::editor_extension::EditorExtensionRegistry;
use zircon_editor::core::editor_operation::{EditorOperationInvocation, EditorOperationPath};
use zircon_editor::EditorPlugin;
use zircon_plugin_neural_runtime::{NnOpAttrs, NnOpCode, NnTensorKind};

use crate::plugin::NeuralModelImportCommand;
use crate::{
    editor_plugin, package_manifest, plugin_registration, EDITOR_CAPABILITIES, EDITOR_CRATE_NAME,
    NEURAL_AUTHORING_CAPABILITY, PLUGIN_ID,
};

#[test]
fn neural_editor_plugin_mirrors_runtime_and_owns_authoring_capability() {
    let plugin = editor_plugin();
    assert_eq!(
        plugin.declaration().mirrored_runtime_package_id(),
        Some(PLUGIN_ID)
    );
    assert_eq!(plugin.declaration().package_manifest(), package_manifest());

    let registration = plugin_registration();
    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    let expected_capabilities = EDITOR_CAPABILITIES
        .iter()
        .map(|capability| (*capability).to_string())
        .collect::<Vec<_>>();
    assert_eq!(registration.capabilities, expected_capabilities);
    assert!(registration
        .capabilities
        .contains(&NEURAL_AUTHORING_CAPABILITY.to_string()));
    let editor_module = registration
        .package_manifest
        .modules
        .iter()
        .find(|module| module.name == "neural.editor")
        .expect("neural editor registration must attach its editor module");
    assert_eq!(editor_module.crate_name, EDITOR_CRATE_NAME);
    assert_eq!(editor_module.capabilities, expected_capabilities);

    let mut extensions = EditorExtensionRegistry::default();
    plugin
        .register_editor_extensions(&mut extensions)
        .expect("neural authoring extensions must register");
    let import_operation =
        EditorOperationPath::parse("neural.model.import").expect("neural import operation");
    let neural_asset_type = AssetTypeId::parse("neural.model").expect("neural model asset type");
    let importers = extensions.asset_importers();
    assert_eq!(importers.len(), 1);
    assert_eq!(importers[0].id(), "neural.model.onnx");
    assert_eq!(importers[0].operation(), &import_operation);
    assert_eq!(importers[0].source_extensions(), ["onnx"]);
    assert_eq!(importers[0].output_type(), Some(&neural_asset_type));
    assert_eq!(
        importers[0].required_capabilities(),
        [NEURAL_AUTHORING_CAPABILITY]
    );
    assert!(extensions.pending_command(&import_operation).is_some());
    let factory = extensions
        .operation_factory(&import_operation)
        .expect("neural import operation factory");
    let project_root = create_test_project("registration");
    fs::write(
        project_root.join("assets/fixtures/model.onnx"),
        minimal_onnx_model(),
    )
    .unwrap();
    let invocation = EditorOperationInvocation::new(import_operation.clone()).with_arguments(
        serde_json::json!({
            "project_root": project_root.to_string_lossy(),
            "source_path": "assets/fixtures/model.onnx",
            "output_path": "assets/model.znn"
        }),
    );
    assert_eq!(
        factory
            .create(&invocation)
            .expect("valid neural import operation command")
            .command()
            .label(),
        "Import ONNX Model"
    );
    assert!(factory
        .create(&EditorOperationInvocation::new(import_operation.clone()))
        .is_err());
    assert!(extensions
        .menu_items()
        .iter()
        .any(|item| item.operation() == &import_operation));
    assert!(extensions
        .asset_type_contributions()
        .iter()
        .any(|contribution| contribution.asset_type() == &neural_asset_type));
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn neural_import_factory_rejects_paths_outside_project_asset_authority() {
    let plugin = editor_plugin();
    let mut extensions = EditorExtensionRegistry::default();
    plugin
        .register_editor_extensions(&mut extensions)
        .expect("neural authoring extensions must register");
    let operation = EditorOperationPath::parse("neural.model.import").unwrap();
    let factory = extensions.operation_factory(&operation).unwrap();
    let project_root = create_test_project("path-authority");
    fs::write(
        project_root.join("assets/fixtures/model.onnx"),
        minimal_onnx_model(),
    )
    .unwrap();

    for arguments in [
        serde_json::json!({
            "project_root": project_root.to_string_lossy(),
            "source_path": "../outside.onnx",
            "output_path": "assets/model.znn"
        }),
        serde_json::json!({
            "project_root": project_root.to_string_lossy(),
            "source_path": "assets/fixtures/model.onnx",
            "output_path": "../outside.znn"
        }),
        serde_json::json!({
            "project_root": project_root.to_string_lossy(),
            "source_path": "assets/fixtures/model.onnx",
            "output_path": project_root.join("assets/model.znn")
        }),
    ] {
        let invocation =
            EditorOperationInvocation::new(operation.clone()).with_arguments(arguments);
        assert!(factory.create(&invocation).is_err());
    }

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn neural_import_apply_and_revert_restore_or_remove_the_project_output() {
    let project_root = create_test_project("apply-revert");
    let source = project_root.join("assets/fixtures/model.onnx");
    let output = project_root.join("assets/model.znn");
    fs::write(&source, minimal_onnx_model()).unwrap();
    fs::write(&output, b"previous-output").unwrap();

    let mut replace = NeuralModelImportCommand::new(source.clone(), output.clone());
    replace.apply_to_filesystem().unwrap();
    assert_ne!(fs::read(&output).unwrap(), b"previous-output");
    replace.revert_filesystem().unwrap();
    assert_eq!(fs::read(&output).unwrap(), b"previous-output");

    fs::remove_file(&output).unwrap();
    let mut create = NeuralModelImportCommand::new(source.clone(), output.clone());
    create.apply_to_filesystem().unwrap();
    assert!(output.is_file());
    create.revert_filesystem().unwrap();
    assert!(!output.exists());

    fs::write(&source, b"not-an-onnx-model").unwrap();
    fs::write(&output, b"preserved-output").unwrap();
    let mut invalid = NeuralModelImportCommand::new(source, output.clone());
    assert!(invalid.apply_to_filesystem().is_err());
    assert_eq!(fs::read(&output).unwrap(), b"preserved-output");

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn nn_onnx_convert_small_cnn_succeeds() {
    let graph = OnnxGraph {
        inputs: vec!["image".to_string()],
        outputs: vec!["ao".to_string()],
        tensors: BTreeMap::from([
            (
                "image".to_string(),
                OnnxTensor::shape_only("image", [1, 1, 4, 4]),
            ),
            (
                "weights".to_string(),
                OnnxTensor::f32("weights", [1, 1, 3, 3], vec![0.0; 9]),
            ),
            ("ao".to_string(), OnnxTensor::shape_only("ao", [1, 1, 2, 2])),
        ]),
        nodes: vec![OnnxNode {
            name: "conv".to_string(),
            op_type: "Conv".to_string(),
            inputs: vec!["image".to_string(), "weights".to_string()],
            outputs: vec!["ao".to_string()],
            attributes: BTreeMap::from([("group".to_string(), OnnxAttribute::Int(1))]),
        }],
    };

    let model = convert_graph(&graph).expect("small CNN graph should convert");

    assert_eq!(model.ops[0].code, NnOpCode::Conv2d);
    assert!(model
        .tensors
        .iter()
        .any(|tensor| tensor.kind == NnTensorKind::Weight));
    assert_eq!(model.weights.len() % 256, 0);
}

#[test]
fn nn_onnx_convert_unsupported_op_diagnostic() {
    let graph = OnnxGraph {
        inputs: vec!["sequence".to_string()],
        outputs: vec!["state".to_string()],
        tensors: BTreeMap::from([
            (
                "sequence".to_string(),
                OnnxTensor::shape_only("sequence", [1, 1, 1, 4]),
            ),
            (
                "state".to_string(),
                OnnxTensor::shape_only("state", [1, 1, 1, 4]),
            ),
        ]),
        nodes: vec![OnnxNode {
            name: "unsupported_lstm".to_string(),
            op_type: "LSTM".to_string(),
            inputs: vec!["sequence".to_string()],
            outputs: vec!["state".to_string()],
            attributes: BTreeMap::new(),
        }],
    };

    let diagnostics = convert_graph(&graph).expect_err("unsupported LSTM must fail conversion");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].op_type, "LSTM");
    assert!(diagnostics[0]
        .to_json_line()
        .contains("\"op_type\":\"LSTM\""));
}

#[test]
fn nn_onnx_convert_accepts_nchw_float_resize_scales() {
    let graph = OnnxGraph {
        inputs: vec!["source".to_string()],
        outputs: vec!["upscaled".to_string()],
        tensors: BTreeMap::from([
            (
                "source".to_string(),
                OnnxTensor::shape_only("source", [1, 1, 2, 2]),
            ),
            (
                "upscaled".to_string(),
                OnnxTensor::shape_only("upscaled", [1, 1, 4, 4]),
            ),
        ]),
        nodes: vec![OnnxNode {
            name: "resize".to_string(),
            op_type: "Resize".to_string(),
            inputs: vec!["source".to_string()],
            outputs: vec!["upscaled".to_string()],
            attributes: BTreeMap::from([
                (
                    "scales".to_string(),
                    OnnxAttribute::Floats(vec![1.0, 1.0, 2.0, 2.0]),
                ),
                (
                    "mode".to_string(),
                    OnnxAttribute::String("nearest".to_string()),
                ),
                (
                    "nearest_mode".to_string(),
                    OnnxAttribute::String("floor".to_string()),
                ),
                (
                    "coordinate_transformation_mode".to_string(),
                    OnnxAttribute::String("asymmetric".to_string()),
                ),
            ]),
        }],
    };

    let model = convert_graph(&graph).expect("standard NCHW Resize scales should convert");

    assert_eq!(model.ops[0].code, NnOpCode::Upsample2d);
    assert!(matches!(
        &model.ops[0].attrs,
        NnOpAttrs::Upsample2d { scale: [2, 2] }
    ));
}

#[test]
fn nn_onnx_convert_rejects_linear_resize_without_linear_backend() {
    let graph = OnnxGraph {
        inputs: vec!["source".to_string()],
        outputs: vec!["upscaled".to_string()],
        tensors: BTreeMap::from([
            (
                "source".to_string(),
                OnnxTensor::shape_only("source", [1, 1, 2, 2]),
            ),
            (
                "upscaled".to_string(),
                OnnxTensor::shape_only("upscaled", [1, 1, 4, 4]),
            ),
        ]),
        nodes: vec![OnnxNode {
            name: "linear_resize".to_string(),
            op_type: "Resize".to_string(),
            inputs: vec!["source".to_string()],
            outputs: vec!["upscaled".to_string()],
            attributes: BTreeMap::from([(
                "mode".to_string(),
                OnnxAttribute::String("linear".to_string()),
            )]),
        }],
    };

    let diagnostics = convert_graph(&graph).expect_err("linear Resize must not become nearest");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].op_type, "Resize");
    assert_eq!(diagnostics[0].reason, "V1 Resize requires nearest mode");
}

#[test]
fn nn_onnx_convert_rejects_resize_without_explicit_backend_sampling_contract() {
    let make_graph = |attributes| OnnxGraph {
        inputs: vec!["source".to_string()],
        outputs: vec!["upscaled".to_string()],
        tensors: BTreeMap::from([
            (
                "source".to_string(),
                OnnxTensor::shape_only("source", [1, 1, 2, 2]),
            ),
            (
                "upscaled".to_string(),
                OnnxTensor::shape_only("upscaled", [1, 1, 4, 4]),
            ),
        ]),
        nodes: vec![OnnxNode {
            name: "resize".to_string(),
            op_type: "Resize".to_string(),
            inputs: vec!["source".to_string()],
            outputs: vec!["upscaled".to_string()],
            attributes,
        }],
    };
    let valid_sampling = BTreeMap::from([
        (
            "mode".to_string(),
            OnnxAttribute::String("nearest".to_string()),
        ),
        (
            "nearest_mode".to_string(),
            OnnxAttribute::String("floor".to_string()),
        ),
        (
            "coordinate_transformation_mode".to_string(),
            OnnxAttribute::String("asymmetric".to_string()),
        ),
    ]);

    let diagnostics =
        convert_graph(&make_graph(valid_sampling.clone())).expect_err("scales must be explicit");
    assert_eq!(diagnostics[0].reason, "V1 Resize requires explicit scales");

    for (missing_attribute, expected_reason) in [
        ("nearest_mode", "V1 Resize requires nearest_mode=floor"),
        (
            "coordinate_transformation_mode",
            "V1 Resize requires coordinate_transformation_mode=asymmetric",
        ),
    ] {
        let mut attributes = valid_sampling.clone();
        attributes.insert(
            "scales".to_string(),
            OnnxAttribute::Floats(vec![1.0, 1.0, 2.0, 2.0]),
        );
        attributes.remove(missing_attribute);
        let diagnostics = convert_graph(&make_graph(attributes))
            .expect_err("implicit ONNX sampling defaults must fail import");
        assert_eq!(diagnostics[0].reason, expected_reason);
    }

    for (attribute, incompatible, expected_reason) in [
        (
            "nearest_mode",
            "round_prefer_floor",
            "V1 Resize requires nearest_mode=floor",
        ),
        (
            "coordinate_transformation_mode",
            "half_pixel",
            "V1 Resize requires coordinate_transformation_mode=asymmetric",
        ),
    ] {
        let mut attributes = valid_sampling.clone();
        attributes.insert(
            "scales".to_string(),
            OnnxAttribute::Floats(vec![1.0, 1.0, 2.0, 2.0]),
        );
        attributes.insert(
            attribute.to_string(),
            OnnxAttribute::String(incompatible.to_string()),
        );
        let diagnostics = convert_graph(&make_graph(attributes))
            .expect_err("backend-incompatible sampling must fail import");
        assert_eq!(diagnostics[0].reason, expected_reason);
    }

    for invalid_scale in [u32::MAX as f32, 1.000_000_1] {
        let mut attributes = valid_sampling.clone();
        attributes.insert(
            "scales".to_string(),
            OnnxAttribute::Floats(vec![1.0, 1.0, invalid_scale, 2.0]),
        );
        let diagnostics = convert_graph(&make_graph(attributes))
            .expect_err("Resize scales must convert to u32 without rounding or saturation");
        assert_eq!(
            diagnostics[0].reason,
            "V1 Resize requires positive integer spatial scales"
        );
    }
}

#[test]
fn nn_onnx_convert_rejects_backend_incompatible_op_arity() {
    let input_names = (0..6)
        .map(|index| format!("input_{index}"))
        .collect::<Vec<_>>();
    let mut tensors = input_names
        .iter()
        .map(|name| (name.clone(), OnnxTensor::shape_only(name, [1, 1, 2, 2])))
        .collect::<BTreeMap<_, _>>();
    tensors.insert(
        "output".to_string(),
        OnnxTensor::shape_only("output", [1, 1, 2, 2]),
    );
    tensors.insert(
        "extra_output".to_string(),
        OnnxTensor::shape_only("extra_output", [1, 1, 2, 2]),
    );

    for (op_type, expected_inputs) in [
        ("Gemm", 2),
        ("MatMul", 2),
        ("Conv", 2),
        ("Relu", 1),
        ("Sigmoid", 1),
        ("Tanh", 1),
        ("Add", 2),
        ("Mul", 2),
        ("Sub", 2),
        ("Div", 2),
        ("BatchNormalization", 5),
        ("LayerNormalization", 3),
        ("MaxPool", 1),
        ("AveragePool", 1),
        ("Resize", 1),
        ("Reshape", 1),
        ("Flatten", 1),
    ] {
        let inputs = input_names[..=expected_inputs].to_vec();
        let expected_reason = format!(
            "V1 {op_type} requires exactly {expected_inputs} {} and 1 output",
            if expected_inputs == 1 {
                "input"
            } else {
                "inputs"
            }
        );
        let graph = OnnxGraph {
            inputs: inputs.clone(),
            outputs: vec!["output".to_string()],
            tensors: tensors.clone(),
            nodes: vec![OnnxNode {
                name: format!("invalid_{}_arity", op_type.to_ascii_lowercase()),
                op_type: op_type.to_string(),
                inputs,
                outputs: vec!["output".to_string()],
                attributes: BTreeMap::new(),
            }],
        };

        let diagnostics = convert_graph(&graph).expect_err("invalid arity must fail import");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].reason, expected_reason);
    }

    let graph = OnnxGraph {
        inputs: vec![input_names[0].clone()],
        outputs: vec!["output".to_string(), "extra_output".to_string()],
        tensors,
        nodes: vec![OnnxNode {
            name: "relu_two_outputs".to_string(),
            op_type: "Relu".to_string(),
            inputs: vec![input_names[0].clone()],
            outputs: vec!["output".to_string(), "extra_output".to_string()],
            attributes: BTreeMap::new(),
        }],
    };
    let diagnostics = convert_graph(&graph).expect_err("multiple outputs must fail import");
    assert_eq!(
        diagnostics[0].reason,
        "V1 Relu requires exactly 1 input and 1 output"
    );
}

#[test]
fn nn_onnx_convert_rejects_backend_incompatible_attributes() {
    let cases = [
        (
            "BatchNormalization",
            vec![vec![1, 2, 2, 2], vec![2], vec![2], vec![2], vec![2]],
            vec![1, 2, 2, 2],
            BTreeMap::from([("epsilon".to_string(), OnnxAttribute::Float(-1.0))]),
            "V1 BatchNormalization requires a finite non-negative epsilon",
        ),
        (
            "LayerNormalization",
            vec![vec![1, 1, 2, 2], vec![2], vec![2]],
            vec![1, 1, 2, 2],
            BTreeMap::from([("epsilon".to_string(), OnnxAttribute::Float(f32::NAN))]),
            "V1 LayerNormalization requires a finite non-negative epsilon",
        ),
        (
            "AveragePool",
            vec![vec![1, 1, 2, 2]],
            vec![1, 1, 1, 1],
            BTreeMap::from([
                ("kernel_shape".to_string(), OnnxAttribute::Ints(vec![2, 2])),
                ("count_include_pad".to_string(), OnnxAttribute::Int(1)),
            ]),
            "V1 AveragePool requires count_include_pad=0",
        ),
        (
            "AveragePool",
            vec![vec![1, 1, 3, 3]],
            vec![1, 1, 1, 1],
            BTreeMap::from([
                ("kernel_shape".to_string(), OnnxAttribute::Ints(vec![2, 2])),
                ("dilations".to_string(), OnnxAttribute::Ints(vec![2, 2])),
            ]),
            "V1 AveragePool does not support attribute dilations",
        ),
    ];

    for (op_type, input_shapes, output_shape, attributes, expected_reason) in cases {
        let graph = single_node_graph(op_type, input_shapes, output_shape, attributes);
        let diagnostics =
            convert_graph(&graph).expect_err("unsupported attributes must fail import");

        assert_eq!(diagnostics.len(), 1, "{op_type}");
        assert_eq!(diagnostics[0].reason, expected_reason, "{op_type}");
    }
}

#[test]
fn nn_onnx_convert_rejects_backend_incompatible_shapes() {
    let resize_attributes = BTreeMap::from([
        (
            "scales".to_string(),
            OnnxAttribute::Floats(vec![1.0, 1.0, 2.0, 2.0]),
        ),
        (
            "mode".to_string(),
            OnnxAttribute::String("nearest".to_string()),
        ),
        (
            "nearest_mode".to_string(),
            OnnxAttribute::String("floor".to_string()),
        ),
        (
            "coordinate_transformation_mode".to_string(),
            OnnxAttribute::String("asymmetric".to_string()),
        ),
    ]);
    let cases = [
        (
            "Add",
            vec![vec![1, 2], vec![2]],
            vec![1, 2],
            BTreeMap::new(),
        ),
        ("Relu", vec![vec![1, 2]], vec![2, 1], BTreeMap::new()),
        (
            "Gemm",
            vec![vec![2, 3], vec![3, 4]],
            vec![2, 5],
            BTreeMap::new(),
        ),
        (
            "Conv",
            vec![vec![1, 1, 4, 4], vec![1, 1, 3, 3]],
            vec![1, 1, 4, 4],
            BTreeMap::from([("group".to_string(), OnnxAttribute::Int(1))]),
        ),
        (
            "MaxPool",
            vec![vec![1, 1, 4, 4]],
            vec![1, 1, 4, 4],
            BTreeMap::from([("kernel_shape".to_string(), OnnxAttribute::Ints(vec![2, 2]))]),
        ),
        (
            "Resize",
            vec![vec![1, 1, 2, 2]],
            vec![1, 1, 3, 3],
            resize_attributes,
        ),
        (
            "BatchNormalization",
            vec![vec![1, 2, 2, 2], vec![3], vec![3], vec![3], vec![3]],
            vec![1, 2, 2, 2],
            BTreeMap::new(),
        ),
        (
            "LayerNormalization",
            vec![vec![1, 1, 2, 2], vec![3], vec![3]],
            vec![1, 1, 2, 2],
            BTreeMap::new(),
        ),
        ("Reshape", vec![vec![2, 3]], vec![5], BTreeMap::new()),
        (
            "MaxPool",
            vec![vec![1, 1, 1, 1]],
            vec![1, 1, 5, 1],
            BTreeMap::from([
                ("kernel_shape".to_string(), OnnxAttribute::Ints(vec![1, 1])),
                ("pads".to_string(), OnnxAttribute::Ints(vec![2, 0, 2, 0])),
            ]),
        ),
        (
            "AveragePool",
            vec![vec![1, 1, 1, 1]],
            vec![1, 1, 5, 1],
            BTreeMap::from([
                ("kernel_shape".to_string(), OnnxAttribute::Ints(vec![1, 1])),
                ("pads".to_string(), OnnxAttribute::Ints(vec![2, 0, 2, 0])),
            ]),
        ),
    ];

    for (op_type, input_shapes, output_shape, attributes) in cases {
        let graph = single_node_graph(op_type, input_shapes, output_shape, attributes);
        let diagnostics =
            convert_graph(&graph).expect_err("backend-invalid shapes must fail import");

        assert_eq!(diagnostics.len(), 1, "{op_type}");
        assert_eq!(
            diagnostics[0].reason,
            format!("V1 {op_type} tensor shapes are not executable by both backends"),
            "{op_type}"
        );
    }
}

#[test]
fn nn_onnx_convert_rejects_terminal_view_outputs_without_gpu_materialization() {
    for (op_type, input_shape, output_shape, attributes) in [
        ("Reshape", vec![2, 3], vec![3, 2], BTreeMap::new()),
        (
            "Flatten",
            vec![2, 3],
            vec![2, 3],
            BTreeMap::from([("axis".to_string(), OnnxAttribute::Int(1))]),
        ),
    ] {
        let graph = single_node_graph(op_type, vec![input_shape], output_shape, attributes);
        let diagnostics = convert_graph(&graph)
            .expect_err("a terminal view must not leave the GPU output resource unwritten");

        assert_eq!(diagnostics.len(), 1, "{op_type}");
        assert_eq!(
            diagnostics[0].reason,
            format!("V1 {op_type} cannot materialize a graph output on the GPU backend"),
            "{op_type}"
        );
    }
}

#[test]
fn nn_onnx_convert_rejects_ops_without_executable_backends() {
    for op_type in ["Concat", "Slice"] {
        let graph = OnnxGraph {
            inputs: vec!["source".to_string()],
            outputs: vec!["output".to_string()],
            tensors: BTreeMap::from([
                (
                    "source".to_string(),
                    OnnxTensor::shape_only("source", [1, 2]),
                ),
                (
                    "output".to_string(),
                    OnnxTensor::shape_only("output", [1, 2]),
                ),
            ]),
            nodes: vec![OnnxNode {
                name: format!("unsupported_{}", op_type.to_ascii_lowercase()),
                op_type: op_type.to_string(),
                inputs: vec!["source".to_string()],
                outputs: vec!["output".to_string()],
                attributes: BTreeMap::new(),
            }],
        };

        let diagnostics =
            convert_graph(&graph).expect_err("unimplemented runtime op must fail import");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].op_type, op_type);
        assert_eq!(
            diagnostics[0].reason,
            "operator has no executable V1 backend"
        );
    }
}

#[test]
fn nn_onnx_convert_rejects_tensor_id_capacity_overflow() {
    let tensors = (0..=(usize::from(u16::MAX) + 1))
        .map(|index| {
            let name = format!("tensor_{index:05}");
            (name.clone(), OnnxTensor::shape_only(name, [1]))
        })
        .collect();
    let graph = OnnxGraph {
        inputs: Vec::new(),
        outputs: Vec::new(),
        tensors,
        nodes: Vec::new(),
    };

    let diagnostics = convert_graph(&graph).expect_err("tensor ids must never wrap");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].op_type, "TensorIdAllocation");
    assert_eq!(
        diagnostics[0].reason,
        "graph exceeds the V1 tensor id capacity"
    );
}

#[test]
fn nn_onnx_convert_reports_initializer_element_count_overflow() {
    let graph = OnnxGraph {
        inputs: Vec::new(),
        outputs: Vec::new(),
        tensors: BTreeMap::from([(
            "oversized_weight".to_string(),
            OnnxTensor::f32("oversized_weight", [65_536, 65_536], Vec::new()),
        )]),
        nodes: Vec::new(),
    };

    let diagnostics =
        convert_graph(&graph).expect_err("initializer size overflow must be diagnosed");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].op_type, "Tensor");
    assert_eq!(
        diagnostics[0].reason,
        "initializer element count exceeds the V1 tensor capacity"
    );
}

#[test]
fn nn_onnx_convert_rejects_backend_index_capacity_for_every_tensor() {
    let graph = single_node_graph(
        "Gemm",
        vec![vec![65_537, 65_537], vec![65_537, 1]],
        vec![65_537, 1],
        BTreeMap::new(),
    );

    let diagnostics = convert_graph(&graph)
        .expect_err("GPU-indexed input tensors must fit the V1 u32 element address space");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.op_type == "Tensor"
            && diagnostic.reason == "tensor element count exceeds the V1 backend index capacity"
    }));
}

#[test]
fn nn_onnx_convert_invalid_group_keeps_input_shape_in_diagnostic() {
    let graph = OnnxGraph {
        inputs: vec!["image".to_string()],
        outputs: vec!["output".to_string()],
        tensors: BTreeMap::from([
            (
                "image".to_string(),
                OnnxTensor::shape_only("image", [1, 2, 4, 4]),
            ),
            (
                "weights".to_string(),
                OnnxTensor::f32("weights", [2, 2, 1, 1], vec![0.0; 4]),
            ),
            (
                "output".to_string(),
                OnnxTensor::shape_only("output", [1, 2, 4, 4]),
            ),
        ]),
        nodes: vec![OnnxNode {
            name: "bad_group".to_string(),
            op_type: "Conv".to_string(),
            inputs: vec!["image".to_string(), "weights".to_string()],
            outputs: vec!["output".to_string()],
            attributes: BTreeMap::from([("group".to_string(), OnnxAttribute::Int(0))]),
        }],
    };

    let diagnostics = convert_graph(&graph).expect_err("zero Conv group must be rejected");

    assert_eq!(
        diagnostics[0].input_shapes,
        vec![vec![1, 2, 4, 4], vec![2, 2, 1, 1]]
    );
}

fn single_node_graph(
    op_type: &str,
    input_shapes: Vec<Vec<u32>>,
    output_shape: Vec<u32>,
    attributes: BTreeMap<String, OnnxAttribute>,
) -> OnnxGraph {
    let inputs = input_shapes
        .iter()
        .enumerate()
        .map(|(index, _)| format!("input_{index}"))
        .collect::<Vec<_>>();
    let mut tensors = inputs
        .iter()
        .zip(input_shapes)
        .map(|(name, shape)| (name.clone(), OnnxTensor::shape_only(name, shape)))
        .collect::<BTreeMap<_, _>>();
    tensors.insert(
        "output".to_string(),
        OnnxTensor::shape_only("output", output_shape),
    );
    OnnxGraph {
        inputs: inputs.clone(),
        outputs: vec!["output".to_string()],
        tensors,
        nodes: vec![OnnxNode {
            name: format!("{}_contract", op_type.to_ascii_lowercase()),
            op_type: op_type.to_string(),
            inputs,
            outputs: vec!["output".to_string()],
            attributes,
        }],
    }
}

static NEXT_TEST_PROJECT: AtomicU64 = AtomicU64::new(1);

fn create_test_project(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zircon-neural-editor-{label}-{}-{}",
        std::process::id(),
        NEXT_TEST_PROJECT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("assets/fixtures")).unwrap();
    fs::write(
        root.join("zircon-project.toml"),
        concat!(
            "name = \"NeuralImportTest\"\n",
            "format_version = 3\n",
            "project_guid = \"b1f87b6d-4823-4b9a-b902-c0e4bb02b1aa\"\n",
            "default_scene = \"res://scenes/main.scene.toml\"\n",
            "asset_roots = [\"assets\"]\n",
            "library_version = 1\n",
        ),
    )
    .unwrap();
    root
}

fn minimal_onnx_model() -> Vec<u8> {
    let dimension = vec![0x08, 0x01];
    let shape = proto_message(1, &dimension);
    let mut tensor_type = vec![0x08, 0x01];
    tensor_type.extend(proto_message(2, &shape));
    let type_proto = proto_message(1, &tensor_type);
    let mut value_info = proto_message(1, b"input");
    value_info.extend(proto_message(2, &type_proto));
    let graph = proto_message(11, &value_info);
    proto_message(7, &graph)
}

fn proto_message(field: u8, payload: &[u8]) -> Vec<u8> {
    assert!(field < 16 && payload.len() < 128);
    let mut bytes = Vec::with_capacity(payload.len() + 2);
    bytes.push((field << 3) | 2);
    bytes.push(payload.len() as u8);
    bytes.extend_from_slice(payload);
    bytes
}
