
use config::Config;
use interactive::RenameOperation;
use renamer::Renamer;

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



fn main() { 
    let config = match Config::new() {
        Ok(config) => config, 
        Err(err) => {
            eprintln!("{}", err); 
            std::process::exit(1);
        }
    };

    let renamer = if config.interactive {
        Renamer::new_with_interactive_mode(&config)
    }else {
        match Renamer::new(&config) {
            Ok(renamer) => renamer, 
            Err(err) => {
                config.printer.print_error(&err);
                std::process::exit(1);
            }
        }
    };


    let operations = match renamer.process() {
        Ok(operations) => operations, 
        Err(err) => {
            config.printer.print_error(&err);
            std::process::exit(1);
        }
    };

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

            match interactive.process_operations(rename_operations) {
                Ok(modified_operations) => {
                    if let Err(err) = interactive.apply_rename_operations(modified_operations) {
                        let custom_error = error::Error {
                            kind: error::ErrorKind::Rename, 
                            value: Some(err.to_string()),
                        };

                        config.printer.print_error(&custom_error);
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    let custom_error = error::Error {
                        kind: error::ErrorKind::Rename, 
                        value: Some(err.to_string()),
                    };
                    config.printer.print_error(&custom_error);
                    std::process::exit(1); 
                }
            }
        }
    }
    else {
        if config.force {
            if let Err(err) = renamer.batch_rename(operations) {
                config.printer.print_error(&err);
                std::process::exit(1);
            }
        }
        else {
            for op in operations {
                config.printer.print_operation(&op.source, &op.target);
            }
        }
    }

    println!("File(s) renamed successfully!");

} 