// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü and Supercomputing Systems AG
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::PermanentStorageHandler;
use log::*;
use std::fs;
use std::path::PathBuf;

use super::PolkadexDBError as Error;
use super::Result;

use crate::constants::DEFAULT_STORAGE_PATH;

/// handles all disc permanent storage interactions of polkadex databases
pub struct DiskStorageHandler {
    path: PathBuf,
    filename: PathBuf,
}

impl Default for DiskStorageHandler {
    fn default() -> Self {
        let filename = PathBuf::from("some_db.bin");
        DiskStorageHandler::open_default(filename)
    }
}

impl DiskStorageHandler {
    pub fn new(path: PathBuf, filename: PathBuf) -> Self {
        DiskStorageHandler { path, filename }
    }

    pub fn open_default(filename: PathBuf) -> Self {
        let path = PathBuf::from(DEFAULT_STORAGE_PATH);
        DiskStorageHandler::new(path, filename)
    }

    pub fn filepath(&self) -> PathBuf {
        self.path.join(self.filename.to_owned())
    }

    pub fn backup_filepath(&self) -> PathBuf {
        self.filepath().with_extension("bin.1")
    }

    /// checks if the dir exists, and if not, creates a new one
    fn ensure_dir_exists(&self) -> Result<()> {
        fs::create_dir_all(&self.path).map_err(Error::FsError)
    }
}

impl PermanentStorageHandler for DiskStorageHandler {
    fn write_to_storage(&mut self, data: &[u8]) -> Result<()> {
        self.ensure_dir_exists()?;
        // copy existing db to backup file:
        debug!("backup db state");
        if fs::copy(self.filepath(), self.backup_filepath()).is_err() {
            warn!("could not backup previous db state");
        };
        fs::write(&self.filepath(), data).map_err(Error::FsError)
    }

    fn read_from_storage(&self) -> Result<Vec<u8>> {
        fs::read(&self.filepath()).map_err(Error::FsError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn create_disc_storage_handler_works() {
        // given
        let path = PathBuf::from("hello");
        let filename = PathBuf::from("handler.txt");

        // when
        let handler = DiskStorageHandler::new(path.clone(), filename.clone());

        // then
        assert_eq!(handler.filename, filename);
        assert_eq!(handler.path, path);
    }

    #[test]
    fn open_default_disc_storage_handler_works() {
        // given
        let filename = PathBuf::from("handler.txt");

        // when
        let handler = DiskStorageHandler::open_default(filename.clone());

        // then
        assert_eq!(handler.filename, filename);
        assert_eq!(handler.path, PathBuf::from(DEFAULT_STORAGE_PATH));
    }

    #[test]
    fn filepath_join_works() {
        // when
        let handler = DiskStorageHandler::new(PathBuf::from("hello"), PathBuf::from("world.txt"));

        // then
        assert_eq!(handler.filepath(), PathBuf::from("hello/world.txt"));
    }

    #[test]
    fn backup_filepath_join_works() {
        // when
        let handler = DiskStorageHandler::new(PathBuf::from("hello"), PathBuf::from("world.bin"));

        // then
        assert_eq!(
            handler.backup_filepath(),
            PathBuf::from("hello/world.bin.1")
        );
    }

    #[test]
    fn ensure_dir_exists_creates_new_if_not_existing() {
        // given
        let path = PathBuf::from("create");
        let filename = PathBuf::from("new.txt");
        let handler = DiskStorageHandler::new(path.clone(), filename);
        // ensure dir is not already existing
        assert!(!path.is_dir());

        // when
        handler.ensure_dir_exists().unwrap();

        // then
        assert!(fs::read_dir(path.as_path()).is_ok());

        //clean up
        fs::remove_dir_all(path.as_path()).unwrap();
        assert!(!path.is_dir());
    }

    #[test]
    fn ensure_dir_exists_does_not_overwrite_existing() {
        // given
        let path = PathBuf::from("do_not_overwrite");
        let filename = PathBuf::from("already_here.txt");
        let handler = DiskStorageHandler::new(path.clone(), filename.clone());
        handler.ensure_dir_exists().unwrap();
        // ensure dir is existing
        assert!(path.is_dir());
        // create dummy file
        fs::File::create(path.join(filename)).unwrap();

        // when (2nd time ensure dir exists)
        handler.ensure_dir_exists().unwrap();

        // then
        assert!(fs::read_dir(path.as_path()).is_ok());
        assert!(fs::File::open(handler.filepath()).is_ok());

        //clean up
        fs::remove_dir_all(path.as_path()).unwrap();
        assert!(!path.is_dir());
    }

    #[test]
    fn read_from_storage_works() {
        // given
        let path = PathBuf::from("read_from_storage_works");
        let filename = PathBuf::from("some_file.txt");
        let data = "hello_world".as_bytes();
        let disk_storage = DiskStorageHandler::new(path.clone(), filename.clone());
        // create file
        fs::create_dir_all(path.clone()).unwrap();
        let mut file = fs::File::create(path.join(filename)).unwrap();
        file.write_all(data).unwrap();

        // when
        let read_data: Vec<u8> = disk_storage.read_from_storage().unwrap();

        // then
        assert_eq!(data, &read_data);

        //clean up
        fs::remove_dir_all(path.clone()).unwrap();
        assert!(!path.is_dir());
    }

    #[test]
    fn write_to_storage_works() {
        // given
        let path = PathBuf::from("write_to_storage_works");
        let filename = PathBuf::from("write_file.bin");
        let data = "hello_world, nice to meet ya".as_bytes();
        let mut disk_storage = DiskStorageHandler::new(path.clone(), filename.clone());

        // when
        disk_storage.write_to_storage(data).unwrap();

        // then
        let read_data: Vec<u8> = fs::read(path.join(filename)).unwrap();
        assert_eq!(data, &read_data);

        //clean up
        fs::remove_dir_all(path.clone()).unwrap();
        assert!(!path.is_dir());
    }

    #[test]
    fn write_to_storage_backups_and_writes_backup_filename_correctly() {
        // given
        let path = PathBuf::from("write_to_storage_backups_correctly");
        let filename = PathBuf::from("file_to_backup.bin");
        let data_to_backup = "Am I really backuped?".as_bytes();
        let data_second_write = "I hope you are..".as_bytes();
        let mut disk_storage = DiskStorageHandler::new(path.clone(), filename.clone());

        // when
        disk_storage.write_to_storage(data_to_backup).unwrap();
        disk_storage.write_to_storage(data_second_write).unwrap(); // 2nd time should backup

        // then
        let read_data: Vec<u8> = fs::read(path.join(filename)).unwrap();
        assert_eq!(data_second_write, &read_data);
        let read_backup_data: Vec<u8> = fs::read(disk_storage.backup_filepath()).unwrap();
        assert_eq!(data_to_backup, &read_backup_data);

        //clean up
        fs::remove_dir_all(path.clone()).unwrap();
        assert!(!path.is_dir());
    }
}
