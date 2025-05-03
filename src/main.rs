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



fn main(){
    let config = match config::Config::new() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{}", err); 
            std::process::exit(1)
        }
    };


    if !config.force {
        let info = &config.printer.colors.info; 
        config.printer.print(&format!("{}", info.paint("Testing the DRY-RUN")))
    }


    //configure renamer

    if !config.force{
        let info = &config.printer.colors.info; 
        config.printer.print(&format!("{}", info.paint("DRY")))
    }


    //CONFIGURE RENAMER 

    let renamer = match Renamer::new(&config) {
        Ok(renamer) => renamer, 
        Err(err) => {
            config.printer.print_error(&err); 
            std::process::exit(1)
        }
    }; 

    //Generate operations
    let operations = match renamer.process() {
        Ok(operation) => operation, 
        Err(err) => {
            config.printer.print_error(&err); 
            std::process::exit(1)
        }
    }; 


    //Batch rename operations
    if let Err(err) = renamer.batch_rename(operations) {
        config.printer.print_error(&err); 
        std::process::exit(1); 
    };
} 