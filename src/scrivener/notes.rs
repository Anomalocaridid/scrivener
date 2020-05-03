//! Note and Index

use failure::{Error, ResultExt};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// Data that points to and uniquely identifies a plaintext file
#[derive(Deserialize, Serialize, Default, Debug, Eq)]
pub struct Note {
    /// A unique identifier that is used to refer to the note
    name: String,

    /// An absolute path pointing to the corresponding file
    path: PathBuf,

    /// A list of strings to enable categorization of notes
    ///
    /// TODO: make tags searchable
    tags: Option<Vec<String>>,
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Ord for Note {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Note {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Note {
    /// Creates a `Note`, provided that the path given is a valid file
    ///
    /// # Errors
    ///
    /// This function will return an error in the following cases,
    /// although it is not limited to them:
    ///
    /// - The file at `path` does not exist.
    /// - `path` points to a directory.
    pub fn new(name: &str, path: &PathBuf, tags: &Option<Vec<String>>) -> Result<Note, Error> {
        let path = fs::canonicalize(&path)
            .with_context(|_| format!("Could not read file `{:?}`.", path))?;

        Ok(Note {
            name: name.to_string(),
            path,
            tags: tags.clone(),
        })
    }

    /// Returns the `Note`'s name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the `Note`'s path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the `Note`'s tags.
    ///
    /// Returns None if there are none and Some(Vec<String>) otherwise.
    pub fn tags(&self) -> &Option<Vec<String>> {
        &self.tags
    }

    /// A helper function to create an instance of `Note` intended to
    /// help search functions search using only the `name`.
    fn dummy(name: &str) -> Note {
        Note {
            name: name.to_string(),
            path: PathBuf::new(),
            tags: None,
        }
    }
}

/// An index of `Note`s in alphabetical order by name.
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Index {
    notes: BTreeSet<Note>,
}

impl Index {
    /// Creates an instance of `Note` and adds it to `self`.
    pub fn add(
        &mut self,
        name: &str,
        path: &PathBuf,
        tags: &Option<Vec<String>>,
    ) -> Result<(), Error> {
        self.notes.insert(Note::new(name, path, tags)?);
        Ok(())
    }

    /// Removes a `Note` from `self`.
    pub fn remove(&mut self, name: &str) -> bool {
        self.notes.remove(&Note::dummy(name))
    }

    /// Returns `true` if an Index contains a note with the given `name`
    /// and false otherwise.
    pub fn contains(&self, name: &str) -> bool {
        self.notes.contains(&Note::dummy(name))
    }

    /// Creates an instance of `Index` using data stored in the config
    /// file, scrivener.toml.
    pub fn load(filename: &str) -> Result<Index, Error> {
        let index =
            confy::load(filename).with_context(|_| format!("could not read {}.toml", filename))?;
        Ok(index)
    }

    /// Updates scrivener.toml using an instance of `Index`
    pub fn store(&self, filename: &str) -> Result<(), Error> {
        confy::store(filename, self)
            .with_context(|_| format!("could not write to {}.toml", filename))?;
        Ok(())
    }

    /// Returns a reference to a Note with a given `name`
    pub fn get(&self, name: &str) -> Option<&Note> {
        self.notes.get(&Note::dummy(name))
    }

    /// Returns a reference to an `Index`'s notes
    pub fn notes(&self) -> &BTreeSet<Note> {
        &self.notes
    }

    /// Creates an empty instance of Index.
    #[allow(dead_code)]
    pub(super) fn new() -> Index {
        Index {
            notes: BTreeSet::new(),
        }
    }
}

impl PartialEq for Index {
    fn eq(&self, other: &Self) -> bool {
        self.notes == other.notes
    }
}

//TODO: Improve tests
#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    // Currently broken when testing for Windows on Linux.
    #[test]
    fn create_note() {
        let file = NamedTempFile::new().unwrap();

        let name = "test";
        let path = file.path().to_path_buf();

        let note = Note::new(name, &path, &None).unwrap();

        let expected = Note {
            name: name.to_string(),
            path,
            tags: None,
        };

        assert_eq!(note, expected);
        assert_eq!(note.path, expected.path);
        assert_eq!(note.tags, expected.tags);
    }

    // Currently broken when testing for Windows on Linux.
    #[test]
    fn create_note_with_tags() {
        let file = NamedTempFile::new().unwrap();

        let name = "test";
        let path = file.path().to_path_buf();
        let tags = Some(vec![
            "one".to_string(),
            "two".to_string(),
            "three".to_string(),
        ]);

        let note = Note::new(name, &path, &tags).unwrap();

        let expected = Note {
            name: name.to_string(),
            path,
            tags,
        };

        assert_eq!(note, expected);
        assert_eq!(note.path, expected.path);
        assert_eq!(note.tags, expected.tags);
    }

    #[test]
    fn add_note_to_index() {
        let file = NamedTempFile::new().unwrap();

        let name = "Test Add";
        let path = file.path().to_path_buf();
        let tags = Some(vec![String::from("one"), String::from("two")]);

        let mut index = Index {
            notes: BTreeSet::new(),
        };

        index.add(name, &path, &tags).unwrap();

        let mut expected = Index::new();
        expected.add(name, &path, &tags).unwrap();

        assert_eq!(index, expected);
    }

    #[test]
    fn remove_note_from_index() {
        let file = NamedTempFile::new().unwrap();

        let name = "Test Remove";
        let path = file.path().to_path_buf();

        let mut index = Index::new();
        index.add(name, &path, &None).unwrap();

        assert!(index.remove(name));

        assert_eq!(index, Index::new());
    }

    #[test]
    fn index_contains_note() {
        let file = NamedTempFile::new().unwrap();

        let name = "Test Contains";
        let path = file.path().to_path_buf();

        let mut index = Index::new();
        index.add(name, &path, &None).unwrap();

        assert!(index.contains(name));
    }
}
