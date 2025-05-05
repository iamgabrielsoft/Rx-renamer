use std::sync::Arc;
use clap::ArgMatches;
use regex::Regex;

use crate::app::{ create_app, FROM_FILE_SUBCOMMAND, TO_ASCII_SUBCOMMMAND}; 
use crate::output::Printer; 



pub struct Config {
    pub force: bool,
    pub backup: bool, 
    pub dirs: bool, 
    pub dump: bool, 
    pub run_mode: RunMode, 
    pub replace_mode:ReplaceMode, 
    pub printer: Printer, 
    pub interactive: bool,
}



#[derive(Debug)]
pub enum RunMode {
    Simple(Vec<String>), 
    Recursive{
        paths: Vec<String>,
        max_depth: Option<usize>,
        hidden: bool  
    }, 

    FromFile {
        path: String, 
        undo: bool
    }
}



pub enum ReplaceMode {
    RegExp {
        expression: Regex, 
        replacement: String,
        limit: usize
    },

    ToASCII
}


#[derive(PartialEq, Debug)]
pub enum AppCommand {
    Root,
    FromFile, 
    ToASCII
}


impl AppCommand {
    pub fn from_str(name: &str) -> Result<AppCommand, String> {
        match name {
            "" => Ok(AppCommand::Root), 
            FROM_FILE_SUBCOMMAND => Ok(AppCommand::FromFile), 
            TO_ASCII_SUBCOMMMAND => Ok(AppCommand::ToASCII),
            _  => Err(format!("Non-registred subcommand '{}'", name)), 
        }
    }
}

impl Config {
    pub fn new() -> Result<Arc<Config>, String>{
        let config = match parse_arguements() {
            Ok(config) => config, 
            Err(err) => return Err(err)
        }; 

        Ok(Arc::new(config))
    }
}



struct ArguementParser<'a> {
    matches: &'a ArgMatches<'a>, 
    printer: &'a Printer, 
    command: &'a AppCommand,
}





impl ArguementParser<'_> {
    fn parse_run_mode(&self) -> Result<RunMode, String>{
        if let AppCommand::FromFile = self.command {
            return Ok(RunMode::FromFile { 
                path: String::from(self.matches.value_of("DUMPFILE").unwrap_or_default()), 
                undo: self.matches.is_present("undo")
            }); 
        }


        //let detect runt 
        let input_paths:Vec<String> = self.matches
            .values_of("PATH(S)")
            .unwrap_or_default()
            .map(String::from)
            .collect(); 

        

        if self.matches.is_present("recursive") {
            let max_depth = if self.matches.is_present("max-depth") {
                Some(
                    self.matches
                        .value_of("max-depth")
                        .unwrap_or_default()
                        .parse::<usize>()
                        .unwrap_or_default(), 
                )

            } else {
                None
            }; 

            Ok(RunMode::Recursive { 
                paths: input_paths, 
                max_depth,
                hidden: self.matches.is_present("hidden") 
            })
        
        }else {
            Ok(RunMode::Simple(input_paths))
        }
    }


    fn parse_replace_mode(&self) -> Result<ReplaceMode, String> {
        if let AppCommand::ToASCII = self.command {
            return Ok(ReplaceMode::ToASCII)
        }


        //get validation for the regex statement of the file
        let expression = match Regex::new(self.matches.value_of("EXPRESSION").unwrap_or_default()) {
            Ok(expr) => expr, 
            Err(err) => {
                return Err(format!(
                    "{} Bad Expression provided\n\n {}", 
                    self.printer.colors.error.paint("Error: "), 
                    self.printer.colors.error.paint(err.to_string()), 
                ))
            }
        };


        let replacement = String::from(self.matches.value_of("REPLACEMENT").unwrap_or_default()); 
        let limit = self.matches
            .value_of("replace-limit")
            .unwrap_or_default()
            .parse::<usize>()
            .unwrap_or_default(); 


        Ok(ReplaceMode::RegExp { expression, replacement, limit, })

    }
}


fn parse_arguements() -> Result<Config, String> {
    let app = create_app(); 

    let matches = app.get_matches(); 
    let ( command, matches ) = match matches.subcommand() {
        (name, Some(submatches)) => (AppCommand::from_str(name)?, submatches), 
        (_, None) => (AppCommand::Root, &matches), //acting as a default here 

        
    }; 


    //set dump to default 
    let dump = if matches.is_present("force") {
        !matches.is_present("no-dump")
    
    }else {
        matches.is_present("dump")

    }; 




    let printer = if matches.is_present("silent") {
        Printer::silent() 

    } else {
        match matches.value_of("color").unwrap_or("auto") {
            "always" => Printer::color(), 
            "never" => Printer::no_color(),
            _ => detect_output_color()
        }
    }; 

    
    let arguement_parser = ArguementParser {
        printer: &printer, 
        matches, 
        command: &command
    }; 




    let run_mode = arguement_parser.parse_run_mode()?; 
    let replace_mode = arguement_parser.parse_replace_mode()?; 

    
    Ok(Config {
        force: matches.is_present("force"), 
        backup: matches.is_present("backup"), 
        dirs: matches.is_present("include-dirs"), 
        dump, 
        run_mode, 
        replace_mode, 
        printer,
    })
}



//detect the output color
fn detect_output_color() -> Printer {
    if atty::is(atty::Stream::Stdout) {
        Printer::color()
        //enable color support here
    
    }else {
        Printer::no_color()
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn app_command_from_string() {
        assert_eq!(AppCommand::from_str("").unwrap(), AppCommand::Root); //check for empty string  
        assert_eq!(AppCommand::from_str(FROM_FILE_SUBCOMMAND).unwrap(), AppCommand::FromFile)
    }


    #[test]
    #[should_panic]
    fn app_command_from_string_unknown_error() {
        AppCommand::from_str("testing unknown command").unwrap(); 
    }
}
