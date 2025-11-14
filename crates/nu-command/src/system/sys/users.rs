use nu_engine::command_prelude::*;

#[derive(Clone)]
pub struct SysUsers;

impl Command for SysUsers {
    fn name(&self) -> &str {
        "sys users"
    }

    fn signature(&self) -> Signature {
        Signature::build("sys users")
            .category(Category::System)
            .input_output_types(vec![(Type::Nothing, Type::table())])
    }

    fn description(&self) -> &str {
        "View information about the users on the system."
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        Ok(users(call.head).into_pipeline_data())
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            description: "Show info about the system users",
            example: "sys users",
            result: None,
        }]
    }
}

fn users(span: Span) -> Value {
    Value::list(Vec::new(), span)
}
