use crate::build::validate_resume;
use crate::cli::ValidateArgs;
use crate::error::AppError;
use crate::json_out::print_json;
use crate::model::MachineResult;
use crate::resume_io::load_resume;

pub fn handle_validate(args: ValidateArgs) -> Result<(), AppError> {
    let resume = load_resume(&args.input)?;
    validate_resume(&resume)?;
    if args.json_output {
        print_json(&MachineResult::success(
            serde_json::json!({
                "input_file": args.input,
                "schema_version": resume.schema_version,
                "status": "valid"
            }),
            Vec::new(),
        ))?;
    } else {
        println!("{} is valid", args.input.display());
    }
    Ok(())
}
