extern crate clap;
use std::{io::{ErrorKind}, path::PathBuf};
use clap::{Arg, App};
use anyhow::{Result};
use std::fs::{read_dir, rename, create_dir};
use infer::MatcherType;

type PathVector = Vec<PathBuf>;

#[derive(Debug)]
struct PathFolders {
  video: PathVector,
  image: PathVector,
  audio: PathVector,
  document: PathVector,
  compressed: PathVector,
  book: PathVector,
  font: PathVector,
  pdf: PathVector,
  other: PathVector,
}

impl PathFolders {
  fn into_iteratable(self) -> [(String, PathVector); 9] {
    [
      (String::from("video"), self.video),
      (String::from("image"), self.image),
      (String::from("other"), self.other),
      (String::from("pdf"), self.pdf),
      (String::from("font"), self.font),
      (String::from("book"), self.book),
      (String::from("compressed"), self.compressed),
      (String::from("document"), self.document),
      (String::from("audio"), self.audio),
    ]
  }
}

macro_rules! empty_path_vector {
  () => {
    vec![]
  }
}

macro_rules! folders {
  ($var_name: ident) => {
    let mut $var_name = PathFolders {
      video: empty_path_vector!(),
      image: empty_path_vector!(),
      audio: empty_path_vector!(),
      document: empty_path_vector!(),
      compressed: empty_path_vector!(),
      book: empty_path_vector!(),
      font: empty_path_vector!(),
      pdf: empty_path_vector!(),
      other: empty_path_vector!(),
    };
  };
}

macro_rules! moveFolderItemsToSubDirectory {
  ($containing_folder: ident, $folder_items: ident) => {
    for item in $folder_items {
      // println!("From {:?}, TO {:?}", item, $containing_folder.to_owned() + "/" + item.file_name().unwrap().to_str().unwrap());
      match rename(item, $containing_folder.to_owned() + "/" + item.file_name().unwrap().to_str().unwrap()) {
        Ok(_x) => {},
        Err(_y) => {
          println!("Moving file failed: {:?}", item);
        },
      }
    }
  };
}

fn main() -> Result<()> {
  let matches = App::new("File Organizer")
                          .version("1.0")
                          .author("Codelock Holmes<nasik2ms@gmail.com>")
                          .about("Organizes the current directory or the provided directory into subfolders based on the content type!")
                          .arg(Arg::with_name("PATH")
                               .help("Sets the input file to use")
                               .default_value(&"./")
                               .index(1))
                          .get_matches();

    // Calling .unwrap() is safe here because "PATH" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    let directory = matches.value_of("PATH").unwrap();

    folders!(folders);

    for item in read_dir(directory)? {
      let item = item?;
      let path = item.path();
      if path.is_file() {
        let input = infer::get_from_path(&path);

        match input.as_ref() {
          Ok(x) => 
            match x {
              Some(a) => {
                let mime_type = a;
                match mime_type.matcher_type() {
                  MatcherType::IMAGE => folders.image.push(path),
                  MatcherType::VIDEO => folders.video.push(path),
                  MatcherType::AUDIO => folders.audio.push(path),
                  MatcherType::DOC => folders.document.push(path),
                  MatcherType::ARCHIVE if mime_type.mime_type() == mime::APPLICATION_PDF => folders.pdf.push(path),
                  MatcherType::ARCHIVE => folders.compressed.push(path),
                  MatcherType::BOOK => folders.book.push(path),
                  MatcherType::CUSTOM => folders.other.push(path),
                  MatcherType::FONT => folders.font.push(path),
                  _ => folders.other.push(path)
                }
              },
              None => {
                folders.other.push(path);
              }
            },
          Err(_why) => println!("Cannot parse file!"),
        }
      }
    }

    for (folder_name, folder_items) in &folders.into_iteratable() {
      // Create folder
      let conatining_folder: &str = &(directory.to_owned() + "/" + folder_name);
      match create_dir(std::path::Path::new(conatining_folder)) {
        Ok(_x) => {
          moveFolderItemsToSubDirectory!(conatining_folder, folder_items);
        },
        Err(x) => {
          match x.kind() {
            ErrorKind::AlreadyExists => {
              moveFolderItemsToSubDirectory!(conatining_folder, folder_items);
            },
            _ => {}
          }
        }
      }
    }

    Ok(())
}
