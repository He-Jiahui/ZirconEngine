use crate::core::framework::script::{
    ScriptHostFieldDescriptor, ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor,
    ScriptHostParameterDescriptor, ScriptHostPrototypeKind, ScriptHostTypeDescriptor,
    ScriptHostTypeRef, ScriptHostValueKind,
};

use super::ScriptHostInterfaceMarkdownOptions;

pub fn render_script_host_modules_markdown(
    modules: &[ScriptHostModuleDescriptor],
    options: &ScriptHostInterfaceMarkdownOptions,
) -> String {
    let mut output = String::new();
    push_heading(&mut output, heading_level(options, 0), &options.title);
    output.push('\n');

    let mut modules = modules.iter().collect::<Vec<_>>();
    modules.sort_by(|left, right| left.name.cmp(&right.name));

    for module in modules {
        render_module(&mut output, module, options);
    }

    output
}

fn render_module(
    output: &mut String,
    module: &ScriptHostModuleDescriptor,
    options: &ScriptHostInterfaceMarkdownOptions,
) {
    push_heading(
        output,
        heading_level(options, 1),
        &format!("Module `{}`", module.name),
    );
    output.push('\n');
    push_line(output, &format!("- Version: `{}`", module.version));
    push_optional_line(output, "- Documentation: ", module.documentation.as_deref());
    render_capabilities(output, "- Capabilities: ", &module.capabilities, options);
    output.push('\n');

    render_types(output, &module.types, options);
    render_functions(output, &module.functions, options);
}

fn render_types(
    output: &mut String,
    types: &[ScriptHostTypeDescriptor],
    options: &ScriptHostInterfaceMarkdownOptions,
) {
    if types.is_empty() && !options.include_empty_sections {
        return;
    }

    push_heading(output, heading_level(options, 2), "Types");
    output.push('\n');

    let mut types = types.iter().collect::<Vec<_>>();
    types.sort_by(|left, right| left.name.cmp(&right.name));

    for type_descriptor in types {
        render_type(output, type_descriptor, options);
    }
}

fn render_type(
    output: &mut String,
    type_descriptor: &ScriptHostTypeDescriptor,
    options: &ScriptHostInterfaceMarkdownOptions,
) {
    push_heading(
        output,
        heading_level(options, 3),
        &format!("Type `{}`", type_descriptor.name),
    );
    output.push('\n');
    push_line(
        output,
        &format!("- Type ref: {}", render_type_ref(&type_descriptor.type_ref)),
    );
    push_line(
        output,
        &format!(
            "- Prototype: `{}`",
            format_prototype_kind(type_descriptor.prototype_kind)
        ),
    );
    push_line(
        output,
        &format!(
            "- Value construction: `{}`",
            type_descriptor.allow_value_construction
        ),
    );
    push_optional_line(
        output,
        "- Documentation: ",
        type_descriptor.documentation.as_deref(),
    );
    output.push('\n');

    render_fields(output, &type_descriptor.fields, options);
}

fn render_fields(
    output: &mut String,
    fields: &[ScriptHostFieldDescriptor],
    options: &ScriptHostInterfaceMarkdownOptions,
) {
    if fields.is_empty() && !options.include_empty_sections {
        return;
    }

    push_line(output, "Fields:");
    output.push('\n');
    for field in fields {
        push_line(
            output,
            &render_named_type_ref(&field.name, &field.type_ref, field.documentation.as_deref()),
        );
    }
    output.push('\n');
}

fn render_functions(
    output: &mut String,
    functions: &[ScriptHostFunctionDescriptor],
    options: &ScriptHostInterfaceMarkdownOptions,
) {
    if functions.is_empty() && !options.include_empty_sections {
        return;
    }

    push_heading(output, heading_level(options, 2), "Functions");
    output.push('\n');

    let mut functions = functions.iter().collect::<Vec<_>>();
    functions.sort_by(|left, right| left.name.cmp(&right.name));

    for function in functions {
        render_function(output, function, options);
    }
}

fn render_function(
    output: &mut String,
    function: &ScriptHostFunctionDescriptor,
    options: &ScriptHostInterfaceMarkdownOptions,
) {
    push_heading(
        output,
        heading_level(options, 3),
        &format!("Function `{}`", function.name),
    );
    output.push('\n');
    push_line(
        output,
        &format!("- Return: {}", render_type_ref(&function.return_type)),
    );
    render_capabilities(
        output,
        "- Required capabilities: ",
        &function.required_capabilities,
        options,
    );
    push_optional_line(
        output,
        "- Documentation: ",
        function.documentation.as_deref(),
    );
    output.push('\n');

    render_parameters(output, &function.parameters, options);
}

fn render_parameters(
    output: &mut String,
    parameters: &[ScriptHostParameterDescriptor],
    options: &ScriptHostInterfaceMarkdownOptions,
) {
    if parameters.is_empty() && !options.include_empty_sections {
        return;
    }

    push_line(output, "Parameters:");
    output.push('\n');
    for parameter in parameters {
        push_line(
            output,
            &render_named_type_ref(
                &parameter.name,
                &parameter.type_ref,
                parameter.documentation.as_deref(),
            ),
        );
    }
    output.push('\n');
}

fn render_capabilities(
    output: &mut String,
    label: &str,
    capabilities: &[String],
    options: &ScriptHostInterfaceMarkdownOptions,
) {
    if !options.include_capabilities || (capabilities.is_empty() && !options.include_empty_sections)
    {
        return;
    }

    let mut capabilities = capabilities.iter().collect::<Vec<_>>();
    capabilities.sort();
    let capabilities = capabilities
        .into_iter()
        .map(|capability| format!("`{capability}`"))
        .collect::<Vec<_>>()
        .join(", ");
    push_line(output, &format!("{label}{capabilities}"));
}

fn render_named_type_ref(
    name: &str,
    type_ref: &ScriptHostTypeRef,
    documentation: Option<&str>,
) -> String {
    match documentation {
        Some(documentation) => format!(
            "- `{name}`: {} - {documentation}",
            render_type_ref(type_ref)
        ),
        None => format!("- `{name}`: {}", render_type_ref(type_ref)),
    }
}

fn render_type_ref(type_ref: &ScriptHostTypeRef) -> String {
    format!(
        "`{}` (`{}`)",
        type_ref.type_name,
        format_value_kind(type_ref.value_kind)
    )
}

fn format_value_kind(value_kind: ScriptHostValueKind) -> &'static str {
    match value_kind {
        ScriptHostValueKind::Null => "null",
        ScriptHostValueKind::Bool => "bool",
        ScriptHostValueKind::Int => "int",
        ScriptHostValueKind::Float => "float",
        ScriptHostValueKind::String => "string",
        ScriptHostValueKind::Bytes => "bytes",
        ScriptHostValueKind::HostHandle => "host_handle",
    }
}

fn format_prototype_kind(prototype_kind: ScriptHostPrototypeKind) -> &'static str {
    match prototype_kind {
        ScriptHostPrototypeKind::Module => "module",
        ScriptHostPrototypeKind::Class => "class",
        ScriptHostPrototypeKind::Interface => "interface",
        ScriptHostPrototypeKind::Struct => "struct",
        ScriptHostPrototypeKind::Enum => "enum",
        ScriptHostPrototypeKind::Native => "native",
    }
}

fn push_optional_line(output: &mut String, prefix: &str, value: Option<&str>) {
    if let Some(value) = value {
        push_line(output, &format!("{prefix}{value}"));
    }
}

fn heading_level(options: &ScriptHostInterfaceMarkdownOptions, offset: usize) -> usize {
    options
        .heading_level
        .clamp(1, 6)
        .saturating_add(offset)
        .clamp(1, 6)
}

fn push_heading(output: &mut String, level: usize, text: &str) {
    output.push_str(&"#".repeat(level.clamp(1, 6)));
    output.push(' ');
    output.push_str(text);
    output.push('\n');
}

fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}
