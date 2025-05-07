use std::result; 

pub type Result<T> = result::Result<T, Error>; 

//error type here
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum  ErrorKind {
    CreateBackup, 
    CreateFile, 
    CreateSymlink, 
    ExistingPath, 
    JsonParse, 
    ReadFile, 
    Rename, 
    SameFilename, 
    SolveOrder
}


#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind, 
    pub value: Option<String>
}


impl Error  {
    pub fn description(&self) -> &str { 
        match self.kind {
            ErrorKind::CreateBackup => "Cannot create a backup of", 
            ErrorKind::CreateFile => "Cannot create file", 
            ErrorKind::CreateSymlink => "Cannot create symlink", 
            ErrorKind::ExistingPath => "Conflict with existing path", 
            ErrorKind::JsonParse => "Cannot parse JSON  file",
            ErrorKind::ReadFile => "Cannot open/read file",
            ErrorKind::Rename => "Cannot Rename", 
            ErrorKind::SameFilename => "Files will have the same name", 
            ErrorKind::SolveOrder => "Cannot solve sorting problem"
        }
    }
}