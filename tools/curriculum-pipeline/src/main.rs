use clap::Parser;
use curriculum_pipeline::cli::{Cli, Command};
use curriculum_pipeline::{compile, extract, generate, lint, validate};
use std::fs;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Extract { pdf, out } => {
            let text = extract::extract_text_from_file(&pdf)?;
            fs::write(&out, &text)?;
            eprintln!("Extracted {} bytes from {:?} -> {:?}", text.len(), pdf, out);
        }
        Command::Generate {
            text,
            subject,
            grade,
            chapter_title,
            source_authority,
            out,
            use_claude,
        } => {
            let source = fs::read_to_string(&text)?;
            let args = generate::GenerateArgs {
                text: source,
                subject,
                grade,
                chapter_title,
                source_authority,
                use_claude,
            };
            let subject_file = generate::generate(&args)?;
            let json = serde_json::to_string_pretty(&subject_file)?;
            fs::write(&out, &json)?;
            eprintln!(
                "Generated content for {} -> {:?}",
                subject_file.subject, out
            );
        }
        Command::Validate { file } => {
            let json = fs::read_to_string(&file)?;
            let subject: curriculum_pipeline::schema::SubjectFile = serde_json::from_str(&json)?;
            let v_issues = validate::validate(&subject);
            let l_issues = lint::lint(&subject);
            let mut exit_code = 0;
            for issue in &v_issues {
                eprintln!("ERROR [{}]: {}", issue.concept_id, issue.message);
                exit_code = 1;
            }
            for issue in &l_issues {
                eprintln!(
                    "LINT  [{}] {}: {}",
                    issue.concept_id, issue.field, issue.message
                );
                exit_code = 1;
            }
            if exit_code == 0 {
                eprintln!("Validation passed: no structural issues or lint warnings.");
            }
            std::process::exit(exit_code);
        }
        Command::Compile { content_dir, out } => {
            compile::compile(&content_dir, &out)?;
        }
    }
    Ok(())
}
