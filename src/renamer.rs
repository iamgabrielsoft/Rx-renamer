
use any_ascii::any_ascii;
use crate::config::{Config, ReplaceMode, RunMode};
use crate::dumpfile::{ Operation, Operations, self};
use crate::error::{ Result, Error, ErrorKind};
use crate::fileutils::{create_backup, get_paths };
use crate::interactive::InterativeMode;
use crate::solver;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;


pub type RenameMap = HashMap<PathBuf, PathBuf>; 


pub struct Renamer {
    config: Arc<Config>, 
    pub interactive: Option<InterativeMode>,
}

impl Renamer {
    pub fn new(config: &Arc<Config>) -> Result<Renamer>{
        Ok(Renamer {
            config: config.clone(), 
            interactive: None, //initialize interactive mode as None
        })
    }

    /** process user input accordingly */
    pub fn process (&self) -> Result<Operations>{
        let operations = match self.config.run_mode {
            RunMode::Simple(_) | RunMode::Recursive { .. } => {
                //get user input path

                let input_paths = get_paths(&self.config.run_mode);
                let rename_map = self.get_rename_map(&input_paths)?;

        
                //solve renaming option  ordering to avoid conflict; 
                solver::solve_rename_order(&rename_map)? 
            }

            RunMode::FromFile { ref path, undo} => {
                //read operation from file
                let operations = dumpfile::read_from_file(&PathBuf::from(path))?; 

                if undo {
                    solver::revert_operations(&operations)? 

                }else {
                    operations
                }

            }
        };


        if self.config.dump {
            dumpfile::dump_to_file(&operations)?; 
        }

        Ok(operations)
    }


    /** Batch rename of files and folders */
    pub fn batch_rename(&self, operations: Operations) -> Result<()> {
        for operation in operations {
            self.rename(&operation)?; 
        }

        Ok(())
    }


    /** Replace file name matches the given config */
    fn replace_match(&self, path: &Path) -> Result<PathBuf> {
        let file_name = path.file_name().ok_or_else(|| Error {
            kind: ErrorKind::ReadFile, 
            value: Some("No file name found".to_string()),
        })?;

        let target_name = match &self.config.replace_mode {
            ReplaceMode::RegExp { 
                expression, 
                replacement, 
                limit,
            } => {

                let file_name_str = file_name.to_str().ok_or_else(|| Error {
                    kind: ErrorKind::ReadFile, 
                    value: Some("File name is not a valid UTF-8 string".to_string()),
                })?;

                expression.replacen(file_name_str, *limit, replacement.as_str()).into_owned()
            }
            ReplaceMode::ToASCII => {
                let file_name_str = file_name.to_str().ok_or_else(|| Error {
                    kind: ErrorKind::ReadFile,
                    value: Some("File name is not a valid UTF-8 string".to_string()),
                })?;
                any_ascii(file_name_str).to_string()
            }   
        };

        Ok(match path.parent() {
            None => PathBuf::from(target_name),
            Some(parent) => parent.join(Path::new(&target_name)),
        })
    }


    /** Rename file if it exist using this medthod */
    fn rename(&self, operation: &Operation) -> Result<()> {
        let printer = &self.config.printer; 
        let colors = &printer.colors; 

        if self.config.force {
            //create backup before renaming with force
            if self.config.backup {
                match create_backup(&operation.source) {
                    Ok(backup) => printer.print(&format!(
                        "{} Backup created - {}",
                        colors.info.paint("Info: "),
                        colors.source.paint(format!(
                            "{} -> {}",
                            operation.source.display(),
                            backup.display()
                        ))
                    )),
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            //rename paths in the filesystem
            if let Err(err) = fs::rename(&operation.source, &operation.target) {
                return Err(Error {
                    kind: ErrorKind::Rename, 
                    value: Some(format!(
                        "{} -> {}\n{}", 
                        operation.source.display(), 
                        operation.target.display(), 
                        err
                    )),
                }); 
            
            }else {
                printer.print_operation(&operation.source, &operation.target); 
            }

        }else {
            printer.print_operation(&operation.source, &operation.target)
        }

        Ok(())

     
        //Rename paths in the filesystem
    }

    fn get_rename_map(&self, paths: &[PathBuf]) -> Result<RenameMap> {
        let printer = &self.config.printer; 
        let colors = &printer.colors; 

        let mut rename_map = RenameMap::new(); 
        let mut error_string = String::new(); 

        for path in paths {
            let target = self.replace_match(path)?; 

            if target != *path {
                if let Some(old_path) = rename_map.insert(target.clone(), path.clone()) {
                    //target cannot be duplicated be any reason
                    error_string.push_str(
                        &colors.error.paint(format!(
                            "\n{0}->{2}\n{1}->{2}\n",
                            old_path.display(), 
                            path.display(), 
                            target.display()
                        ))
                        .to_string()
                    ); 
                }
            }
            
        }

        if error_string.is_empty() {
            Ok(rename_map)
        
        }else {
            Err(Error{
                kind: ErrorKind::SameFilename, 
                value: Some(error_string)
            })
        }
    }

    pub fn new_with_interactive_mode(config: &Arc<Config>) -> Self {
        Renamer {
            config: config.clone(),
            interactive: Some(InterativeMode::new()),
        }

    }

}