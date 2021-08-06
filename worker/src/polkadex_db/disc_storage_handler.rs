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
use std::path::PathBuf;
use rocksdb::{DB, Options};
pub struct DiscStorageHandler {
    storage_location: PathBuf,
    storage: DB,
}

impl Default for DiscStorageHandler {
    fn default() -> Self {
        let path = "some_db.bin";
        DiscStorageHandler::open_default(path)
    }
}
impl DiscStorageHandler {
    fn new(path: PathBuf, storage: DB) -> Self {
        DiscStorageHandler { path, storage }
    }

    pub fn open_default(path: PathBuf) -> Self {
        storage = DB::open_default(path);
        DiscStorageHandler::new(path, storage)
    }



    pub fn set_path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }

    pub fn set_filename(mut self, filename: PathBuf) -> Self {
        self.filename = filename;
        self
    }

    pub fn filepath(&self) -> PathBuf {
        self.path.join(self.filename.to_owned())
    }

    /// checks if the dir exists, and if not, creates a new one
    fn ensure_dir_exists(&self) -> Result<()> {
        if !&self.path.is_dir() {
            fs::create_dir_all(&self.path)?
        }
        Ok(())
    }
}

impl DiscStorageHandler for PermanentStorageHandler {
    write_to_storage()
}