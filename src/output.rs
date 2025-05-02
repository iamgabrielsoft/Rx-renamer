use ansi_term::Colour::*;
use ansi_term::Style;
use difference::{Changeset, Difference};
use crate::error::*;
use std::path::Path;




#[derive(PartialEq)]
enum PrinterMode {
    Silent, 
    NoColor,
    Color
}


pub struct Printer {
    pub colors: Colors, 
    mode: PrinterMode
}


pub struct Colors {
    pub info: Style, 
    pub warn: Style, 
    pub error: Style, 
    pub source: Style, 
    pub target: Style, 
    pub highlight: Style
}



impl Printer {
    pub fn color() -> Printer {
        let colors =  Colors {
            info: Style::default().bold(), 
            warn: Style::from(Yellow), 
            error: Style::from(Red), 
            source: Style::from(Fixed(8)), 
            target: Style::from(Green), 
            highlight: Style::from(Red).bold(),
        }; 


        Self {
            colors, 
            mode: PrinterMode::Color
        }
    }


    pub fn no_color() -> Printer {
        let colors =  Colors {
            info: Style::default(), 
            warn: Style::default(), 
            error: Style::default(), 
            source: Style::default(), 
            target: Style::default(), 
            highlight: Style::default()
        }; 

        Printer {
            colors, 
            mode: PrinterMode::NoColor
        }
    }

    pub fn silent() -> Printer {
        let colors = Colors {
            info: Style::default(), 
            warn: Style::default(), 
            error: Style::default(), 
            source: Style::default(), 
            target: Style::default(), 
            highlight: Style::default()
        }; 


        Printer {
            colors, 
            mode: PrinterMode::Silent
        }
    }


    pub fn print(&self, message: &str) {
        match self.mode {
            PrinterMode::Color | PrinterMode::NoColor => {
                println!("{}", message); 
            }

            PrinterMode::Silent => {}
        }
    }


    pub fn eprint(&self, message:&str) {
        match  self.mode {
            PrinterMode::Color | PrinterMode::NoColor => {
                eprintln!("{}", message); 
            }

            PrinterMode::Silent => {}
        }
    }


    pub fn print_error(&self, error: &Error) {
        let error_value = error.value.to_owned().unwrap_or_else(|| String::from("")); 


        self.eprint(&format!(
            "{}{}{}", 
            self.colors.error.paint("Error:"), 
            error.description(), 
            self.colors.error.paint(error_value)
        )); 
    }


    pub fn print_operation(&self, source: &Path, target: &Path) {
        if self.mode == PrinterMode::Silent {
            return ;
        }

        let mut _source_parent = source.parent().unwrap().to_string_lossy(); 
        let mut _source_name = source.file_name().unwrap().to_string_lossy().to_string(); 
        let _target_parent = target.parent().unwrap().to_string_lossy().to_string(); 
        let mut _target_name = target.file_name().unwrap().to_string_lossy().to_string();


        if self.mode == PrinterMode::Color {
            _target_name = self.string_diff (
                &_source_name, 
                &_target_name, 
                self.colors.target, 
                self.colors.highlight, 

            )
        }


        _source_name = self.colors.source.paint(&_source_name).to_string(); 


    }


    
    #[warn(unused_variables)]
    fn string_diff(&self, original: &str, changed: &str, _base_color: Style,  diff_color: Style) -> String {
        let mut colored_string = String::new(); 
        let changedset = Changeset::new(original, changed, "");


        //ieterate through the difference of two strings
        for difference in changedset.diffs {
            match difference {
                Difference::Same(_string) => {
                    colored_string = format!("{}", colored_string); 
                }

                Difference::Add(string) => {
                    colored_string = format!("{}{}", colored_string, diff_color.paint(string))
                }

                Difference::Rem(_) => continue, 
            }
        }

        colored_string
    }
}