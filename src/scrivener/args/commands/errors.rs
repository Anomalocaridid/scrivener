//! Formatting for consistent error messages.

use std::path::PathBuf;

/// Used in the event of a successfull operation.
pub(super) fn successful(name: &str, action: &str) {
    println!("Note `{}` has been {} successfully", name, action)
}

/// Used when an instance of `Note` with a given `name` already exists.
pub(super) fn already_exists(name: &str) -> String {
    format!("A note named `{}` already exists.", name)
}

/// Used when an instance of note does not exist when it should.
pub(super) fn does_not_exist(name: &str) -> String {
    format!("Note `{}` does not exist.", name)
}

/// Used for general cases when an action cannot be completed.
pub(super) fn could_not(action: &str) -> String {
    format!("Could not {}.", action)
}

/// Used when a `Note`'s file cannot be operated on.
pub(super) fn could_not_note(action: &str, name: &str, path: &PathBuf) -> String {
    format!(
        "Could not {} note `{}` at {}.",
        action,
        name,
        path.display()
    )
}
