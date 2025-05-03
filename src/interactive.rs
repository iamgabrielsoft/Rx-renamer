use std::process::{ Command, Stdio }; 
use std::path::PathBuf; 
use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use tempfile::{tempfile, NamedTempFile};

pub enum Editor {
  VIM, 
  NOTEPAD, 
}


#[derive(Serialize, Deserialize)]
struct RenameOperation {
  old_name: String, 
  new_name: String,
  status: bool
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
    if(cfg(target_os = "windows")) {
      Editor::NOTEPAD
    }
    else {
      Editor::VIM
    }
  }

  pub fn edit_file(&self, file_path: &PathBuf) -> Result<()> {
    match self {
      Editor::VIM => {
        Command::new("vim")
          .arg(file_path)
          .stdin(Stdio::inherit())
          .stdout(Stdio::inherit())
          .stderr(Stdio::inherit())
          .spawn()?
          .wait()
      }

      Editor::NotePad => {
        Command::new("notepad")
          .arg(file_path)
          .stdin(Stdio::inherit())
          .stdout(Stdio::inherit())
          .stderr(Stdio::inherit())
          .spawn()?
          .wait()
      }
    }

    Ok(())
  }
}


impl InterativeMode {
  pub fn new() -> Self {
    InterativeMode {
      editor: Editor::getEditor(),
    }
  }

  pub fn process_operations(&self, operations: Vec<RenameOperation>) -> Result<()> {
    let mut temp_file = NamedTempFile::new(); 
    let mut operation_data = Vec::new();

    for (old, new) in &operations {
      operation_data.push(RenameOperation {
        old_name: old.to_string_lossy().to_string(),
        new_name: new.to_string_lossy().to_string(),
        status: "pending".to_string(),
      });
    }

    serde_json::to_writer_pretty(&temp_file, &operation_data)?;

    self.editor.editFile(temp_file.path())?;


    let modified_data = serde_json::from_reader(temp_file.as_file())
      .into_iter()
      .filter(|op| op.status == "completed")
      .map(|op| {
        (
          PathBuf::from(&op.old_name),
          PathBuf::from(&op.new_name),
        )
      })
      .collect();

    Ok(modified_data)
  }
}