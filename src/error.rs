//
// Copyright 2021 Hans W. Uhlig. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::fmt::Formatter;

pub type DotVoxResult<T, I> = std::result::Result<T, DotVoxError<I>>;

pub enum DotVoxError<I> {
    NoMainChunk,
    NomError(nom::error::Error<I>),
    IOError(std::io::Error),
}

impl<I> std::fmt::Display for DotVoxError<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DotVoxError::NoMainChunk => write!(f, "NoMainChunk"),
            DotVoxError::NomError(err) => write!(f, "NomError({})", err.code.description()),
            DotVoxError::IOError(err) => write!(f, "{}", err),
        }
    }
}

impl<I> std::fmt::Debug for DotVoxError<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<I> std::error::Error for DotVoxError<I> {}