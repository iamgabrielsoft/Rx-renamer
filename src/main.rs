use config::Config;
use interactive::RenameOperation;
use renamer::Renamer;
use error::Result;

extern crate ansi_term;
extern crate any_ascii;
extern crate atty;
extern crate chrono;
extern crate difference;
extern crate path_abs;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate walkdir;



#[macro_use]
extern crate clap;
extern crate serde_derive;



mod dumpfile;
mod config;
mod app;
mod error; 
mod output;
mod fileutils;
mod renamer;
mod solver;
mod interactive;



fn main() -> Result<()> { 
    let config = Config::new().map_err(|e| error::Error {
        kind: error::ErrorKind::ReadFile,
        value: Some(e),
    })?;

    let renamer = if config.interactive {
        Renamer::new_with_interactive_mode(&config)
    }else {
        Renamer::new(&config)?
    };


    let operations = renamer.process()?;

    if config.interactive {
        if let Some(interactive) = &renamer.interactive {
            let rename_operations = operations
                .into_iter()
                .map(|op| RenameOperation {
                    old_name: op.source.to_string_lossy().to_string(), 
                    new_name: op.target.to_string_lossy().to_string(),
                    status: false,
                })
                .collect();

            let modified_operation = interactive.process_operations(rename_operations);

            interactive.apply_rename_operations(modified_operation);
        }
    }
    else {
        if config.force {
            renamer.batch_rename(operations)?;
        }
        else {
            for op in operations {
                config.printer.print_operation(&op.source, &op.target);
            }
        }
    }

    println!("File(s) renamed successfully!");
    Ok(())
} 