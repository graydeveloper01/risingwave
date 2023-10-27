// Copyright 2023 RisingWave Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::borrow::Cow;

use itertools::Itertools;
use risingwave_pb::expr::ExprNode;
use risingwave_pb::plan_common::column_desc::GeneratedOrDefaultColumn;
use risingwave_pb::plan_common::{PbColumnCatalog, PbColumnDesc};

use super::row_id_column_desc;
use crate::catalog::{offset_column_desc, Field, ROW_ID_COLUMN_ID};
use crate::types::DataType;

/// Column ID is the unique identifier of a column in a table. Different from table ID, column ID is
/// not globally unique.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColumnId(i32);

impl std::fmt::Debug for ColumnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl ColumnId {
    pub const fn new(column_id: i32) -> Self {
        Self(column_id)
    }

    /// Sometimes the id field is filled later, we use this value for better debugging.
    pub const fn placeholder() -> Self {
        Self(i32::MAX - 1)
    }
}

impl ColumnId {
    pub const fn get_id(&self) -> i32 {
        self.0
    }

    /// Returns the subsequent column id.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn apply_delta_if_not_row_id(&mut self, delta: i32) {
        if self.0 != ROW_ID_COLUMN_ID.get_id() {
            self.0 += delta;
        }
    }
}

impl From<i32> for ColumnId {
    fn from(column_id: i32) -> Self {
        Self::new(column_id)
    }
}
impl From<&i32> for ColumnId {
    fn from(column_id: &i32) -> Self {
        Self::new(*column_id)
    }
}

impl From<ColumnId> for i32 {
    fn from(id: ColumnId) -> i32 {
        id.0
    }
}

impl From<&ColumnId> for i32 {
    fn from(id: &ColumnId) -> i32 {
        id.0
    }
}

impl std::fmt::Display for ColumnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ColumnDesc {
    pub data_type: DataType,
    pub column_id: ColumnId,
    pub name: String,
    pub generated_or_default_column: Option<GeneratedOrDefaultColumn>,
    pub description: Option<String>,
}

impl ColumnDesc {
    pub fn unnamed(column_id: ColumnId, data_type: DataType) -> ColumnDesc {
        ColumnDesc {
            data_type,
            column_id,
            name: String::new(),
            generated_or_default_column: None,
            description: None,
        }
    }

    /// Convert to proto
    pub fn to_protobuf(&self) -> PbColumnDesc {
        PbColumnDesc {
            column_type: Some(self.data_type.to_protobuf()),
            column_id: self.column_id.get_id(),
            name: self.name.clone(),
            generated_or_default_column: self.generated_or_default_column.clone(),
            description: self.description.clone(),
        }
    }

    pub fn new_atomic(data_type: DataType, name: &str, column_id: i32) -> Self {
        Self {
            data_type,
            column_id: ColumnId::new(column_id),
            name: name.to_string(),
            generated_or_default_column: None,
            description: None,
        }
    }

    pub fn from_field_with_column_id(field: &Field, id: i32) -> Self {
        Self {
            data_type: field.data_type.clone(),
            column_id: ColumnId::new(id),
            name: field.name.clone(),
            description: None,
            generated_or_default_column: None,
        }
    }

    pub fn from_field_without_column_id(field: &Field) -> Self {
        Self::from_field_with_column_id(field, 0)
    }

    pub fn is_generated(&self) -> bool {
        matches!(
            self.generated_or_default_column,
            Some(GeneratedOrDefaultColumn::GeneratedColumn(_))
        )
    }

    pub fn is_default(&self) -> bool {
        matches!(
            self.generated_or_default_column,
            Some(GeneratedOrDefaultColumn::DefaultColumn(_))
        )
    }
}

impl From<PbColumnDesc> for ColumnDesc {
    fn from(prost: PbColumnDesc) -> Self {
        Self {
            data_type: DataType::from(prost.column_type.as_ref().unwrap()),
            column_id: ColumnId::new(prost.column_id),
            name: prost.name,
            generated_or_default_column: prost.generated_or_default_column,
            description: prost.description.clone(),
        }
    }
}

impl From<&PbColumnDesc> for ColumnDesc {
    fn from(prost: &PbColumnDesc) -> Self {
        prost.clone().into()
    }
}

impl From<&ColumnDesc> for PbColumnDesc {
    fn from(c: &ColumnDesc) -> Self {
        Self {
            column_type: c.data_type.to_protobuf().into(),
            column_id: c.column_id.into(),
            name: c.name.clone(),
            generated_or_default_column: c.generated_or_default_column.clone(),
            description: c.description.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnCatalog {
    pub column_desc: ColumnDesc,
    pub is_hidden: bool,
}

impl ColumnCatalog {
    /// Get the column catalog's is hidden.
    pub fn is_hidden(&self) -> bool {
        self.is_hidden
    }

    /// If the column is a generated column
    pub fn is_generated(&self) -> bool {
        self.column_desc.is_generated()
    }

    /// If the column is a generated column
    pub fn generated_expr(&self) -> Option<&ExprNode> {
        if let Some(GeneratedOrDefaultColumn::GeneratedColumn(desc)) =
            &self.column_desc.generated_or_default_column
        {
            Some(desc.expr.as_ref().unwrap())
        } else {
            None
        }
    }

    /// If the column is a column with default expr
    pub fn is_default(&self) -> bool {
        self.column_desc.is_default()
    }

    /// Get a reference to the column desc's data type.
    pub fn data_type(&self) -> &DataType {
        &self.column_desc.data_type
    }

    /// Get the column desc's column id.
    pub fn column_id(&self) -> ColumnId {
        self.column_desc.column_id
    }

    /// Get a reference to the column desc's name.
    pub fn name(&self) -> &str {
        self.column_desc.name.as_ref()
    }

    /// Convert column catalog to proto
    pub fn to_protobuf(&self) -> PbColumnCatalog {
        PbColumnCatalog {
            column_desc: Some(self.column_desc.to_protobuf()),
            is_hidden: self.is_hidden,
        }
    }

    /// Creates a row ID column (for implicit primary key).
    pub fn row_id_column() -> Self {
        Self {
            column_desc: row_id_column_desc(),
            is_hidden: true,
        }
    }

    pub fn offset_column() -> Self {
        Self {
            column_desc: offset_column_desc(),
            is_hidden: true,
        }
    }
}

impl From<PbColumnCatalog> for ColumnCatalog {
    fn from(prost: PbColumnCatalog) -> Self {
        Self {
            column_desc: prost.column_desc.unwrap().into(),
            is_hidden: prost.is_hidden,
        }
    }
}

impl ColumnCatalog {
    pub fn name_with_hidden(&self) -> Cow<'_, str> {
        if self.is_hidden {
            Cow::Owned(format!("{}(hidden)", self.column_desc.name))
        } else {
            Cow::Borrowed(&self.column_desc.name)
        }
    }
}

pub fn columns_extend(preserved_columns: &mut Vec<ColumnCatalog>, columns: Vec<ColumnCatalog>) {
    debug_assert_eq!(ROW_ID_COLUMN_ID.get_id(), 0);
    let mut max_incoming_column_id = ROW_ID_COLUMN_ID.get_id();
    columns.iter().for_each(|column| {
        let column_id = column.column_id().get_id();
        if column_id > max_incoming_column_id {
            max_incoming_column_id = column_id;
        }
    });
    preserved_columns.iter_mut().for_each(|column| {
        column
            .column_desc
            .column_id
            .apply_delta_if_not_row_id(max_incoming_column_id)
    });

    preserved_columns.extend(columns);
}

pub fn is_column_ids_dedup(columns: &[ColumnCatalog]) -> bool {
    let mut column_ids = columns
        .iter()
        .map(|column| column.column_id().get_id())
        .collect_vec();
    column_ids.sort();
    let original_len = column_ids.len();
    column_ids.dedup();
    column_ids.len() == original_len
}

#[cfg(test)]
pub mod tests {
    use risingwave_pb::plan_common::PbColumnDesc;

    use crate::catalog::ColumnDesc;
    use crate::test_prelude::*;
    use crate::types::DataType;

    pub fn build_prost_desc() -> PbColumnDesc {
        let city = vec![
            PbColumnDesc::new_atomic(DataType::Varchar.to_protobuf(), "country.city.address", 2),
            PbColumnDesc::new_atomic(DataType::Varchar.to_protobuf(), "country.city.zipcode", 3),
        ];
        let country = vec![
            PbColumnDesc::new_atomic(DataType::Varchar.to_protobuf(), "country.address", 1),
            // PbColumnDesc::new_struct("country.city", 4, ".test.City", city),
        ];
        // PbColumnDesc::new_struct("country", 5, ".test.Country", country)
        country[0].clone()
    }

    pub fn build_desc() -> ColumnDesc {
        let city = vec![
            ColumnDesc::new_atomic(DataType::Varchar, "country.city.address", 2),
            ColumnDesc::new_atomic(DataType::Varchar, "country.city.zipcode", 3),
        ];
        let country = vec![
            ColumnDesc::new_atomic(DataType::Varchar, "country.address", 1),
            // ColumnDesc::new_struct("country.city", 4, ".test.City", city),
        ];
        // ColumnDesc::new_struct("country", 5, ".test.Country", country)
        country[0].clone()
    }

    #[test]
    fn test_into_column_catalog() {
        let desc: ColumnDesc = build_prost_desc().into();
        assert_eq!(desc, build_desc());
    }
}
