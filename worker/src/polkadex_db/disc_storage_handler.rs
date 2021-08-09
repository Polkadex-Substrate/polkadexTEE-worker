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

const DEFAULT_STORAGE_PATH: &str = "polkadex_storage";
/// handles all disc permanent storage interactions of polkadex databases
pub struct DiscStorageHandler {
    path: PathBuf,
    filename: PathBuf,
}

impl Default for DiscStorageHandler {
    fn default() -> Self {
        let filename = PathBuf::from("some_db.bin");
        DiscStorageHandler::open_default(filename)
    }
}

impl DiscStorageHandler {
    pub fn new(path: PathBuf, filename: PathBuf) -> Self {
        DiscStorageHandler { path, filename }
    }

    pub fn open_default(filename: PathBuf) -> Self {
        let path = PathBuf::from(DEFAULT_STORAGE_PATH);
        DiscStorageHandler::new(path, filename)
    }

    pub fn filepath(&self) -> PathBuf {
        self.path.join(self.filename.to_owned())
    }

    /// checks if the dir exists, and if not, creates a new one
    fn ensure_dir_exists(&self) -> Result<()> {
        fs::create_dir_all(&self.path).map_err(Error::FsError)
    }
}

impl PermanentStorageHandler for DiscStorageHandler {
    fn write_to_storage(&self, data: &[u8]) -> Result<()> {
        self.ensure_dir_exists()?;
        // copy existing db to backup file:
        debug!("backup db state");
        if fs::copy(self.path.clone(), self.path.with_extension("bin.1")).is_err() {
            warn!("could not backup previous db state");
        };
        fs::write(&self.filepath(), data).map_err(Error::FsError)
    }

    fn read_from_storage(&self) -> Result<Vec<u8>> {
        fs::read(&self.filepath()).map_err(Error::FsError)
    }
}
