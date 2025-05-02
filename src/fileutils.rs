

//get the path of the run command made by the user

use crate::config::RunMode;

use crate::error::*; 
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use path_abs::PathAbs;
use walkdir::{ DirEntry, WalkDir}; 


pub type PathList = Vec<PathBuf>; 



pub fn get_paths(mode: &RunMode) -> PathList {
    match mode {
        RunMode::Recursive { 
            paths, 
            max_depth, 
            hidden } => {
            
                //detect of the file is hidden or not
            // let mut path_list = PathList::new(); //list of paths available

            let is_hidden = |f:&DirEntry| -> bool {
                if !hidden && f.depth() > 0 {
                  f.file_name().to_str().map(|s| !s.starts_with(".")).unwrap_or(false)
                }  else {
                  true // or hidden
                }
            };

            let mut path_list = PathList::new(); 

            for path in paths {
                let walkdir = match max_depth {
                    Some(max_depth) => WalkDir::new(path).max_depth(*max_depth), 
                    None => WalkDir::new(path)
                }; 


                let mut walk_list: PathList = walkdir
                        .into_iter()
                        .filter_entry(is_hidden)
                        .filter_map(|e| e.ok())
                        .map(|p| p.path().to_path_buf())
                        .collect(); 
                        //before accessing collect it needs to be converted to a buf then collected as a vector

                        path_list.append(&mut walk_list); 
            }

            path_list

        },

        RunMode::Simple(path_list) => path_list.iter().map(PathBuf::from).collect(), 
        _ => PathList::new()
    }
}




pub fn get_unique_filename(path: &Path, suffix: &str) -> PathBuf {
    let base_name = format!("{} {}", path.file_name().unwrap().to_string_lossy(), suffix); 
    let mut unique_name = path.to_path_buf(); //convert this to an owned path
    unique_name.set_file_name(&base_name); 
    

    let mut index = 0; 
    
    while unique_name.symlink_metadata().is_ok() {
        index += 1; 
        unique_name.set_file_name(format!("{}.{}", base_name, index))
    }


    unique_name
}



//create file backup 
pub fn create_backup(path: &Path) -> Result<PathBuf> {
    //create a backup 
    let backup = get_unique_filename(path, ".rx");
    match fs::copy(path, &backup) {
        Ok(_) => Ok(backup),
        Err(_) => Err(Error {
            kind: ErrorKind::CreateBackup,
            value: Some(path.to_string_lossy().to_string()),
        }),
    }
}





pub fn is_same_file(source: &Path, target:&Path) -> bool {
    {
        let source_metadata = fs::File::open(&source).unwrap().metadata().unwrap(); 
        let target_metadata = fs::File::open(&target).unwrap().metadata().unwrap(); 
        let low_source = source.to_string_lossy().to_string().to_lowercase(); 
        let low_target = target.to_string_lossy().to_string().to_lowercase(); 

        return low_source == low_target && 
            source_metadata.file_type() == target_metadata.file_type() && 
            source_metadata.len() == target_metadata.len()
            && source_metadata.modified().unwrap() == target_metadata.modified().unwrap()
    }
}





pub fn create_symlink(source: &Path, symlink_file: &Path) -> Result<()> {
     #[cfg(windows)]
    match ::std::os::windows::fs::symlink_file(source, symlink_file) {
        Ok(_) => Ok(()), 
        Err(_) => Err(Error {
            kind: ErrorKind::CreateSymlink, 
            value: Some(symlink_file.to_string_lossy().to_string()), 
        })
    };


    #[cfg(unix)]
    //match the symlink file path for unix system 
    match ::std::os::unix::fs::symlink(source, symlink_file) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error{
            kind: ErrorKind::CreateSymlink, 
            value: Some(symlink_file.to_string_lossy().to_string()), 
        })
    }
}


/* cleanup the paths created  */
pub fn cleanup_paths(paths: &mut PathList, keep_dirs: bool) {
    paths.retain(|path| {
        if path.symlink_metadata().is_err() {
            return false; 
        }

        if path.is_dir(){
            keep_dirs && path.file_name().is_some()
        }else {
            true 
        }
    }); 


    let abs_path_map: HashMap<PathAbs, PathBuf> = paths.drain(..)
        .map(|f| (PathAbs::new(&f).unwrap(), f)).collect(); 

    paths.append(&mut abs_path_map.values().cloned().collect()); 
}



//for test 
#[cfg(test)]
mod test {
    use super::*; 
    use std::{fs, io::Write}; 


    #[test]
    fn get_file_list() {
        let mock_files: Vec<String> = vec![
            "test_file_1.txt".to_string(), 
            "test_file_2.txt".to_string(), 
            "test_file_3.txt".to_string()
        ]; 

        let mode = RunMode::Simple(mock_files); 
        let files = get_paths(&mode);
        assert!(files.contains(&PathBuf::from("test_file.1.txt"))); 
        assert!(files.contains(&PathBuf::from("test_file_2.txt"))); 
        assert!(files.contains(&PathBuf::from("test_file_3.txt")));  
    }



    
    #[test]
    fn test_same_file() {
        let tempdir = tempfile::tempdir().expect("Experience error creating temp directory"); 
        println!("Runing test in {:?}", tempdir); 

        let temp_pasth = tempdir.path().to_str().unwrap(); 
        let mock_files:PathList = vec![
            [temp_pasth, "test_file"].iter().collect(), 
            [temp_pasth, "test_FILE"].iter().collect(), 
            [temp_pasth, "test_file"].iter().collect(),
        ]; 

        let other_file = PathBuf::from(format!("{}/other_file", temp_pasth)); 

        for file in &mock_files {
            //iterate through the mock_files
            fs::File::create(&file).expect("error creating mock file...")
                .write_all(b"Hello gabrielsoft")
                .expect("Error writin in the mock file..."); 
        }


        fs::File::create(&other_file)
            .expect("Error when creating mock file...")
            .write_all(b"Hello dude")
            .expect("Error writing mock file"); 

        //tageting macos here
        #[cfg(any(windows, target_os="macos"))]
        {
            assert!(is_same_file(&mock_files[0], &mock_files[0])); 

        }

        //if its not macos -> windows, linux,  android etc
        #[cfg(not(any(windows, target_os="macos")))]
        {

        }
    }



    //perform cleanu[]
    #[test]
    fn cleanup() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

   
        let mock_dirs: PathList = vec![
            [temp_path, "mock_dir_1"].iter().collect(),
            [temp_path, "mock_dir_1", "mock_dir_2"].iter().collect(),
        ];


        #[rustfmt::skip]
        let mock_files: PathList = vec![
            [temp_path, "test_file.txt"].iter().collect(),
            [&mock_dirs[0], &PathBuf::from("test_file.txt")].iter().collect(),
            [&mock_dirs[1], &PathBuf::from("test_file.txt")].iter().collect(),
        ];

        // Create directory tree, files and symlinks in the filesystem
        for mock_dir in &mock_dirs {
            fs::create_dir(&mock_dir).expect("Error creating mock directory...");
        }


        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }


        let symlink: PathBuf = [temp_path, "test_link"].iter().collect();
        let broken_symlink: PathBuf = [temp_path, "test_broken_link"].iter().collect();
        create_symlink(&mock_files[0], &symlink).expect("Error creating symlink.");
        create_symlink(&PathBuf::from("broken_link"), &broken_symlink)
            .expect("Error creating broken symlink.");

        // Create mock_paths from files, symlink, directories, false files and duplicated files
        // Existing files

        let mut mock_paths = PathList::new();

        mock_paths.append(&mut mock_files.clone());
        // Symlinks
        mock_paths.push(symlink.clone());


        mock_paths.push(broken_symlink.clone());


        // Directories
        mock_paths.append(&mut mock_dirs.clone());


        // False files
        #[rustfmt::skip]
        let false_files: PathList = vec![
            [temp_path, "false_file.txt"].iter().collect(),
            [&mock_dirs[0], &PathBuf::from("false_file.txt")].iter().collect(),
            [&mock_dirs[1], &PathBuf::from("false_file.txt")].iter().collect(),
        ];


        mock_paths.append(&mut mock_files.clone());
        // Quadruplicate existing files
        mock_paths.append(&mut mock_files.clone());
        mock_paths.append(&mut mock_files.clone());
        mock_paths.append(&mut mock_files.clone());



        cleanup_paths(&mut mock_paths, false);

        // Must contain these the files
        let mut listed_files = PathList::new();
        listed_files.append(&mut mock_files.clone());
        listed_files.push(symlink.clone());
        listed_files.push(broken_symlink.clone());


        
        for file in &listed_files {
            assert!(mock_paths.contains(file));
            // Only once
            assert_eq!(mock_paths.iter().filter(|f| f == &file).count(), 1);
        }

        // Must NOT contain these files/directories
        #[rustfmt::skip]
        let mut non_listed_files = PathList::new();
        

        non_listed_files.append(&mut mock_dirs.clone());
        non_listed_files.append(&mut false_files.clone());

        for file in &non_listed_files {
            assert!(!mock_paths.contains(file));
        }
    
    }

}