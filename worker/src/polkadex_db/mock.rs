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
use super::Result;

pub struct PermanentStorageMock {
    pub contained_data: Vec<u8>,
}
impl Default for PermanentStorageMock {
    fn default() -> Self {
        PermanentStorageMock {
            contained_data: vec![],
        }
    }
}

impl PermanentStorageHandler for PermanentStorageMock {
    fn write_to_storage(&self, _data: &[u8]) -> Result<()> {
        Ok(())
    }
    fn read_from_storage(&self) -> Result<Vec<u8>> {
        Ok(self.contained_data.clone())
    }
}
