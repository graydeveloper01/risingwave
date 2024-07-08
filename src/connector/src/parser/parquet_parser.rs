// Copyright 2024 RisingWave Labs
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
use std::sync::Arc;

use arrow_array::RecordBatch;
use futures_async_stream::try_stream;
use opendal::{FuturesAsyncReader, Operator};
use risingwave_common::array::{ArrayBuilderImpl, DataChunk, StreamChunk};
use risingwave_common::types::{Datum, ScalarImpl};

use crate::parser::ConnectorResult;
use crate::source::{SourceColumnDesc, SourceContextRef};

/// `ParquetParser` is responsible for converting the incoming `record_batch_stream`
/// into a `streamChunk`.
#[derive(Debug)]
pub struct ParquetParser {
    rw_columns: Vec<SourceColumnDesc>,
    source_ctx: SourceContextRef,
}

impl ParquetParser {
    pub fn new(
        rw_columns: Vec<SourceColumnDesc>,
        source_ctx: SourceContextRef,
    ) -> ConnectorResult<Self> {
        Ok(Self {
            rw_columns,
            source_ctx,
        })
    }

    #[try_stream(boxed, ok = StreamChunk, error = crate::error::ConnectorError)]
    pub async fn into_stream(
        self,
        record_batch_stream: parquet::arrow::async_reader::ParquetRecordBatchStream<
            FuturesAsyncReader,
        >,
        file_name: String,
    ) {
        #[for_await]
        for record_batch in record_batch_stream {
            let record_batch: RecordBatch = record_batch?;
            // Convert each record batch into a stream chunk according to user defined schema.
            let chunk: StreamChunk = convert_record_batch_to_stream_chunk(
                record_batch,
                self.rw_columns.clone(),
                file_name.clone(),
            )?;
            yield chunk;
        }
    }
}

/// The function `convert_record_batch_to_stream_chunk` is designed to transform the given `RecordBatch` into a `StreamChunk`.
///
/// For each column in the source column:
/// - If the column's schema matches a column in the `RecordBatch` (both the data type and column name are the same),
///   the corresponding records are converted into a column of the `StreamChunk`.
/// - If the column's schema does not match, null values are inserted.
/// - Hidden columns are handled separately by filling in the appropriate fields to ensure the data chunk maintains the correct format.
/// - If a column in the Parquet file does not exist in the source schema, it is skipped.
///
/// # Arguments
///
/// * `record_batch` - The `RecordBatch` to be converted into a `StreamChunk`.
/// * `source_columns` - User defined source schema.
///
/// # Returns
///
/// A `StreamChunk` containing the converted data from the `RecordBatch`.

// The hidden columns that must be included here are _rw_file and _rw_offset.
// Depending on whether the user specifies a primary key (pk), there may be an additional hidden column row_id.
// Therefore, the maximum number of hidden columns is three.
const MAX_HIDDEN_COLUMN_NUMS: usize = 3;
fn convert_record_batch_to_stream_chunk(
    record_batch: RecordBatch,
    source_columns: Vec<SourceColumnDesc>,
    file_name: String,
) -> Result<StreamChunk, crate::error::ConnectorError> {
    let size = source_columns.len();
    let mut chunk_columns = Vec::with_capacity(source_columns.len() + MAX_HIDDEN_COLUMN_NUMS);
    for source_column in source_columns {
        match source_column.column_type {
            crate::source::SourceColumnType::Normal => {
                match source_column.is_hidden_addition_col {
                    false => {
                        if let Some(parquet_column) =
                            record_batch.column_by_name(&source_column.name)
                        {
                            let converted_arrow_data_type =
                                arrow_schema::DataType::try_from(&source_column.data_type)?;

                            if &converted_arrow_data_type == parquet_column.data_type() {
                                let column = Arc::new(parquet_column.try_into()?);
                                chunk_columns.push(column);
                            } else {
                                // data type mismatch, this column is set to null.
                                let mut array_builder =
                                    ArrayBuilderImpl::with_type(size, source_column.data_type);

                                array_builder.append_n_null(record_batch.num_rows());
                                let res = array_builder.finish();
                                let column = Arc::new(res);
                                chunk_columns.push(column);
                            }
                        } else {
                            // For columns defined in the source schema but not present in the Parquet file, null values are filled in.
                            let mut array_builder =
                                ArrayBuilderImpl::with_type(size, source_column.data_type);

                            array_builder.append_n_null(record_batch.num_rows());
                            let res = array_builder.finish();
                            let column = Arc::new(res);
                            chunk_columns.push(column);
                        }
                    }
                    // handle hidden columns, for file source, the hidden columns are only `Offset` and `Filename`
                    true => {
                        if let Some(additional_column_type) =
                            source_column.additional_column.column_type
                        {
                            match additional_column_type{
                                risingwave_pb::plan_common::additional_column::ColumnType::Offset(_) =>{
                                    let mut array_builder =
                                    ArrayBuilderImpl::with_type(size, source_column.data_type);
                                    let datum: Datum =  Some(ScalarImpl::Utf8("0".into()));
                                    array_builder.append_n(record_batch.num_rows(), datum);
                                    let res = array_builder.finish();
                                    let column = Arc::new(res);
                                    chunk_columns.push(column);

                                },
                                risingwave_pb::plan_common::additional_column::ColumnType::Filename(_) => {
                                    let mut array_builder =
                                    ArrayBuilderImpl::with_type(size, source_column.data_type);
                                    let datum: Datum =  Some(ScalarImpl::Utf8(file_name.clone().into()));
                                    array_builder.append_n(record_batch.num_rows(), datum);
                                    let res = array_builder.finish();
                                    let column = Arc::new(res);
                                    chunk_columns.push(column);
                                },
                                _ => unreachable!()
                            }
                        }
                    }
                }
            }
            crate::source::SourceColumnType::RowId => {
                let mut array_builder = ArrayBuilderImpl::with_type(size, source_column.data_type);
                let datum: Datum = None;
                array_builder.append_n(record_batch.num_rows(), datum);
                let res = array_builder.finish();
                let column = Arc::new(res);
                chunk_columns.push(column);
            }
            // The following fields is ony used in CDC source
            crate::source::SourceColumnType::Offset | crate::source::SourceColumnType::Meta => {
                unreachable!()
            }
        }
    }

    let data_chunk = DataChunk::new(chunk_columns.clone(), record_batch.num_rows());
    Ok(data_chunk.into())
}
