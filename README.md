# Rerun template repository
Template for our private and public repos, containing CI, CoC, etc

When creating a new Rerun repository, use this as a template, then modify it as it makes sense.

This template should be the default for any repository of any kind, including:
* Rust projects
* C++ projects
* Python projects
* Other stuff

This template includes
* License files
* Code of Conduct
* Helpers for checking and linting Rust code
  - `cargo-clippy`
  - `cargo-deny`
  - `rust-toolchain`
  - …
* CI for:
  - Spell checking
  - Link checking
  - C++ checks
  - Python checks
  - Rust checks


## How to use
Start by clicking "Use this template" at https://github.com/rerun-io/rerun_template/ or follow [these instructions](https://docs.github.com/en/free-pro-team@latest/github/creating-cloning-and-archiving-repositories/creating-a-repository-from-a-template).

Then follow these steps:
* Run `scripts/template_update.py init --languages cpp,rust,python` to delete files you don't need (give the languages you need support for)
* Search and replace all instances of `new_repo_name` with the name of the repository.
* Search and replace all instances of `new_project_name` with the name of the project (crate/binary name).
* Search for `TODO` and fill in all those places
* Replace this `README.md` with something better
* Commit!

In the future you can always update this repository with the latest changes from the template by running:
* `scripts/template_update.py update --languages cpp,rust,python`
