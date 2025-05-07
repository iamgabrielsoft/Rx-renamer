use std::fs;
use std::io::Write;
use std::process::{ Command, Stdio }; 
use std::path::PathBuf; 
use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use tempfile::NamedTempFile;

pub enum Editor {
  VIM, 
  NOTEPAD, 
}


#[derive(Serialize, Deserialize)]
pub struct RenameOperation {
  pub old_name: String,  
  pub new_name: String,
  pub status: bool
}

pub struct InterativeMode {
  editor: Editor,
}

impl Editor {
  pub fn from_str(s: &str) -> Option<Self> {
    match s.to_lowercase().as_str() {
      "vim" => Some(Self::VIM), 
      "notepad" => Some(Self::NOTEPAD),
      _ => None,  
    }
  }

  pub fn get_editor() -> Self {
    #[cfg(target_os = "windows")] {
      Editor::NOTEPAD
    }
    #[cfg(not(target_os = "windows"))] {
      Editor::VIM
    }
  }

  pub fn edit_file(&self, file_path: &PathBuf) -> Result<()> {
    let status = match self {
      Editor::VIM => {
        Command::new("vim")
          .arg(file_path)
          .stdin(Stdio::inherit())
          .stdout(Stdio::inherit())
          .stderr(Stdio::inherit())
          .spawn()?
          .wait()?
      }

      Editor::NOTEPAD => {
        Command::new("notepad")
          .arg(file_path)
          .stdin(Stdio::inherit())
          .stdout(Stdio::inherit())
          .stderr(Stdio::inherit())
          .spawn()?
          .wait()?
      }
    };

    if status.success() {
      Ok(())
    }else {
      anyhow::bail!("Failed to open editor {:?}", status)
    }

   
  }
}


impl InterativeMode {
  pub fn new() -> Self {
    InterativeMode {
      editor: Editor::get_editor(),
    }
  }

  pub fn process_operations(&self, operations: Vec<RenameOperation>) -> Result<Vec<RenameOperation>> {
    let mut temp_file = NamedTempFile::new()?; 
    let file_path = temp_file.path().to_path_buf();

    let json = serde_json::to_string_pretty(&operations)?;
    temp_file.write_all(json.as_bytes())?;
    temp_file.flush()?;

    self.editor.edit_file(&file_path)?;

    let modified_json = fs::read_to_string(file_path)?; 
    let modified_operations = serde_json::from_str(&modified_json)?;

    Ok(modified_operations)
  }


  pub fn apply_rename_operations(&self, operations: Result<Vec<RenameOperation>>) -> Result<()> {
      for op in operations? {
        if op.status {
          fs::rename(&op.old_name, op.new_name)?;
        }
      }

    Ok(())
  }
}