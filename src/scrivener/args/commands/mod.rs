//! Subcommands and related logic.

use failure::{Error, ResultExt};
use prettytable::{format, Attr, Cell, Row, Table};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use crate::scrivener::notes::Index;

mod errors;

#[derive(Debug, StructOpt)]
/// Command line note application
///
/// Stores the name, path, and tags of a note in a file named
/// scrivener.toml, which is stored in the following locations:  
///
/// Linux: ~/.config/scrivener/scrivener.toml  
///
pub enum Command {
    /// Opens a new file in the user's default text editor.
    ///
    /// Uses $EDITOR if it is set and defaults to vi otherwise
    New {
        /// A unique identifier to associate with the note
        name: String,

        /// The note file's intended location
        ///
        /// Defaults to the current directory if not specified
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,

        /// An optional list of tags to attach to the note
        #[structopt(short, long)]
        tags: Option<Vec<String>>,
    },

    /// Adds an existing plaintext file to the notes index
    Add {
        /// The name to associate with the note
        name: String,

        /// The path to the file to add
        #[structopt(parse(from_os_str))]
        path: PathBuf,

        /// An optional list of tags to attach to the note
        #[structopt(short, long)]
        tags: Option<Vec<String>>,
    },

    /// Edits an existing note
    ///
    /// Defaults to the current directory if not specified
    Edit {
        /// The name of the note to edit
        name: String,
    },

    /// Removes a note from the notes index without deleting the file
    Remove {
        /// The name of the note to remove
        name: String,
    },
    /// Removes a note from the notes index and deletes its file
    Delete {
        /// The name of the note to delete
        name: String,
    },
    /// Lists all notes
    List {
        /// Show each note file's path
        #[structopt(short = "p", long = "paths")]
        show_paths: bool,

        /// Show each note's tags
        #[structopt(short = "t", long = "tags")]
        show_tags: bool,
    }, // /// Searches all notes for notes with a given name or tag
       // TODO: Search {}

       // /// Runs a note if it is marked as executable
       // TODO: Run {}
}

impl Command {
    /// Executes a function that corresponds to the outcome of a
    /// subcommand.
    pub fn execute(&self, index: &mut Index) -> Result<(), Error> {
        match self {
            Command::New { name, path, tags } => create_new_note(index, name, path, tags),
            Command::Add { name, path, tags } => add_note(index, name, path, tags),
            Command::Edit { name } => edit_note(index, name),
            Command::Remove { name } => remove_note(index, name),
            Command::Delete { name } => delete_note(index, name),
            Command::List {
                show_paths,
                show_tags,
            } => list_notes(index, *show_paths, *show_tags),
        }
    }
}

/// Adds an existing file to the `Index`.
///
/// # Errors
///
/// - When a `Note` with the same name as the one being added already
/// exists in the `Index`.
fn add_note(
    index: &mut Index,
    name: &str,
    path: &PathBuf,
    tags: &Option<Vec<String>>,
) -> Result<(), Error> {
    failure::ensure!(!index.contains(name), errors::already_exists(name));

    index.add(name, path, tags)?;

    println!("Note `{}` at {} added successfully.", name, path.display());

    Ok(())
}

/// Creates a file and adds it as a `Note` to the `Index`
///
/// If `None` is given as the path, the path used is the current
/// working directory.
///
/// Prompts a user for input by opening a temportary file with
/// the user's default texteditor.
///
/// For Linux, this is the value of $EDITOR. If $EDITOR is not set,
/// vi is used instead.
///
/// # Panics
///
/// - $EDITOR is set to an invalid command
///
/// # Errors
///
/// - A `Note` with the same name as the one being added exists.
///
/// - The path given is a directory, already has a file, or is
/// otherwise inaccessible.
fn create_new_note(
    index: &mut Index,
    name: &str,
    path: &Option<PathBuf>,
    tags: &Option<Vec<String>>,
) -> Result<(), Error> {
    let path = match path {
        Some(path) => path.clone(),
        None => {
            let mut path = std::env::current_dir()
                .with_context(|_| errors::could_not("access current directory"))?;
            path.push(format!("{}.txt", &name));
            path
        }
    };

    failure::ensure!(!index.contains(name), errors::already_exists(name));

    failure::ensure!(
        !path.is_dir(),
        "{} is a directory, not a file.",
        path.display()
    );
    failure::ensure!(
        !path.exists(),
        "A file at {} already exists.",
        path.display()
    );

    let mut file =
        File::create(&path).with_context(|_| format!("Could not create {}.", path.display()))?;

    let text = scrawl::new().with_context(|_| errors::could_not("open editor"))?;

    file.write_all(&text.as_bytes())
        .with_context(|_| errors::could_not("write to file"))?;

    add_note(index, name, &path, tags)?;

    Ok(())
}

/// Edits an existing note.
///
/// Prompts the user for input by opening a temporary file with
/// the default text editor.
///
/// For Linux, this is the value of $EDITOR. If $EDITOR is not set,
/// vi is used instead.
///
/// # Panics
///
/// - $EDITOR is set to an invalid command
///
/// # Errors
///
/// - There is no note with the `name` that is given.
fn edit_note(index: &mut Index, name: &str) -> Result<(), Error> {
    let path = match index.get(name) {
        Some(note) => note.path(),
        None => failure::bail!(errors::does_not_exist(name)),
    };

    scrawl::edit(path).with_context(|_| errors::could_not_note("open", name, path))?;

    errors::successful(name, "edited");

    Ok(())
}

/// Removes a note from the `Index` WITHOUT deleting the
/// corresponding file.
///
/// # Errors
///
/// - There is no `Note` in the `Index` with the given name.
fn remove_note(index: &mut Index, name: &str) -> Result<(), Error> {
    let status = index.remove(name);

    failure::ensure!(status, errors::does_not_exist(name));

    errors::successful(name, "removed");

    Ok(())
}

/// Removes a note from the `Index` AND deletes the corresponding
/// file.
///
/// # Errors
///
/// - There is no `Note` in the `Index` with the given name.
///
/// - The `Note` cannot be deleted.
fn delete_note(index: &mut Index, name: &str) -> Result<(), Error> {
    let path = match index.get(name) {
        Some(note) => note.path(),
        None => failure::bail!(errors::does_not_exist(name)),
    };

    fs::remove_file(path).with_context(|_| errors::could_not_note("delete", name, path))?;

    remove_note(index, name)?;

    errors::successful(name, "deleted");

    Ok(())
}

/// Lists all `Note`s in the `Index` in a table printed to the screen
/// with or without its relative path and tags.
///
/// If `show_paths` and `show_tags` are both false, then the table
/// will have only one column that shows the `Note`s' names
///
/// If either `show_paths` or `show_tags` are true, the table will
/// have two columns, one for the names and one for either the paths
/// or tags, respectively.
///
/// If both `show_paths` and `show_tags` are true, then the table
/// will have tree columns, with names, paths, and tags.
///
/// If the `Index` is empty, then a helpful message will be shown
/// instead.
fn list_notes(index: &Index, show_paths: bool, show_tags: bool) -> Result<(), Error> {
    // If index has no notes, print a helpful message and return.
    if index.notes().is_empty() {
        println!("There are no notes to list!");
        println!("Create one with 'srcv new <name>'");
        println!("Try 'srcv --help' for more options.");
        return Ok(());
    }

    let mut table = Table::new();

    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    // Add a cell to the title row that says "Notes" in bold.
    let mut title = Row::new(vec![Cell::new("Notes").with_style(Attr::Bold)]);

    // If show_paths is true, add a cell to the title row that says
    // "Paths" in bold.
    if show_paths {
        title.add_cell(Cell::new("Paths").with_style(Attr::Bold));
    }

    // If show_tags is true, add a cell to the title row that says
    // "Tags" in bold.
    if show_tags {
        title.add_cell(Cell::new("Tags").with_style(Attr::Bold));
    }

    table.set_titles(title);

    // For every note in the index
    for note in &mut index.notes().iter() {
        // Initialize a row with the note's name in the first cell.
        let mut row = Row::new(vec![Cell::new(note.name())]);

        // If show_paths is true
        if show_paths {
            // Get the current working directory
            let note_path = note.path();
            let path = abs_to_rel(note_path);

            // Add the path to the row.
            row.add_cell(Cell::new(&path));
        }

        // If show_tags is true
        if show_tags {
            let tags = note.tags();

            // If the note has tags associated with it
            if let Some(tags) = tags {
                // Initialize tag_list as a new String.
                let mut tag_list = String::new();

                // Then, split the tag list into the first element and
                // every other element
                if let Some((first, rest)) = tags.split_first() {
                    // Push the first tag to tag_list
                    tag_list.push_str(first);

                    // For any the remaining tags
                    for tag in rest {
                        // Append it to tag_list after a comma and a newline.
                        tag_list.push_str(&format!(",\n{}", tag));
                    }
                }
                // Add tag_list to the row
                row.add_cell(Cell::new(&tag_list));
            } else {
                // Else, add an empty string to the row
                row.add_cell(Cell::new(&String::new()));
            }
        }

        // Add the row to the table
        table.add_row(row);
    }

    // Print the table
    table.printstd();

    Ok(())
}

/// Determines whether a path is directly inside root
fn is_in_root(path: &Path) -> bool {
    let root = "/";
    path.canonicalize().unwrap().parent().unwrap() == Path::new(root)
}

/// Converts an absolute path pointing to a file to a relative path
/// based on the current working directory unless the current working
/// directory is inaccessible or the path points to a file in root
fn abs_to_rel(path: &Path) -> String {
    // If the current directory is accessible
    if let Ok(current_dir) = std::env::current_dir() {
        // And If path has the current working directory as its
        // parent directory
        if let Ok(rel_path) = path.strip_prefix(&current_dir) {
            // Then strip path's prefix, add "./{}" to it,
            // and return it
            return format!("./{}", rel_path.display());
        } else {
            let parent_dir = "../";

            // Else, set rel_prefix to "../"
            let mut rel_prefix = String::from(parent_dir);

            // Set ancestor to the current directory's parent directory
            let mut ancestor = current_dir.parent().unwrap().to_path_buf();

            let rel_path = loop {
                // If note_path has parent_dir as a prefix,
                if let Ok(rel_path) = path.strip_prefix(&ancestor) {
                    // Then break the loop and return rel_prefix with
                    // rel_path appended to it.
                    break format!("{}{}", rel_prefix, rel_path.display());
                }

                //Else, pop the last component off of parent_dir
                ancestor.pop();

                // And append "../" to rel_prefix
                rel_prefix.push_str(parent_dir);
            };

            // If rel_paths's parent directory is not root
            if !is_in_root(Path::new(&rel_path)) {
                // Then return rel_path
                rel_path
            } else {
                //Else, return path as a string
                path.to_str().unwrap().to_string()
            }
        }
    } else {
        // Else, return path as a string
        path.to_str().unwrap().to_string()
    }
}

//TODO: Improve tests
#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn add_a_note() {
        let mut index = Index::new();
        let file = NamedTempFile::new().unwrap();

        let name = "Test Add";
        let path = file.path().to_path_buf();
        let tags = Some(vec![String::from("one"), String::from("two")]);

        add_note(&mut index, name, &path, &tags).unwrap();

        let mut expected = Index::new();
        expected.add(name, &path, &tags).unwrap();

        assert_eq!(index, expected);
    }

    #[test]
    fn remove_a_note() {
        let mut index = Index::new();
        let file = NamedTempFile::new().unwrap();

        let name = "Test Remove";
        let path = file.path().to_path_buf();

        add_note(&mut index, name, &path, &None).unwrap();

        remove_note(&mut index, name).unwrap();

        assert_eq!(index, Index::new());
    }
}
