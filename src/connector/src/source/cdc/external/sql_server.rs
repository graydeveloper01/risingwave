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

use std::cmp::Ordering;

use anyhow::Context;
use futures::stream::BoxStream;
use futures::{pin_mut, StreamExt, TryStreamExt};
use futures_async_stream::try_stream;
use itertools::Itertools;
use risingwave_common::bail;
use risingwave_common::catalog::{ColumnDesc, ColumnId, Schema};
use risingwave_common::row::OwnedRow;
use risingwave_common::types::{DataType, ScalarImpl};
use serde_derive::{Deserialize, Serialize};
use tiberius::error::Error;
use tiberius::{ColumnType, Config, Query, QueryItem};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::error::{ConnectorError, ConnectorResult};
use crate::parser::sql_server_row_to_owned_row;
use crate::source::cdc::external::{
    CdcOffset, CdcOffsetParseFunc, DebeziumOffset, ExternalTableConfig, ExternalTableReader,
    SchemaTableName,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SqlServerOffset {
    // https://learn.microsoft.com/en-us/answers/questions/1328359/how-to-accurately-sequence-change-data-capture-dat
    pub change_lsn: String,
    pub commit_lsn: String,
}

// only compare the lsn field
impl PartialOrd for SqlServerOffset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.change_lsn.partial_cmp(&other.change_lsn) {
            Some(Ordering::Equal) => self.commit_lsn.partial_cmp(&other.commit_lsn),
            other => other,
        }
    }
}

impl SqlServerOffset {
    pub fn parse_debezium_offset(offset: &str) -> ConnectorResult<Self> {
        let dbz_offset: DebeziumOffset = serde_json::from_str(offset)
            .with_context(|| format!("invalid upstream offset: {}", offset))?;

        Ok(Self {
            change_lsn: dbz_offset
                .source_offset
                .change_lsn
                .context("invalid sql server change_lsn")?,
            commit_lsn: dbz_offset
                .source_offset
                .commit_lsn
                .context("invalid sql server commit_lsn")?,
        })
    }
}

pub struct SqlServerExternalTable {
    column_descs: Vec<ColumnDesc>,
    pk_names: Vec<String>,
}

impl SqlServerExternalTable {
    pub async fn connect(config: ExternalTableConfig) -> ConnectorResult<Self> {
        tracing::debug!("connect to sql server");

        let mut client_config = Config::new();

        client_config.host(&config.host);
        client_config.database(&config.database);
        client_config.port(config.port.parse::<u16>().unwrap());
        client_config.authentication(tiberius::AuthMethod::sql_server(
            &config.username,
            &config.password,
        ));
        client_config.trust_cert();

        let tcp = TcpStream::connect(client_config.get_addr()).await?;
        tcp.set_nodelay(true)?;

        let mut client: tiberius::Client<Compat<TcpStream>> =
            match tiberius::Client::connect(client_config, tcp.compat_write()).await {
                // Connection successful.
                Ok(client) => Ok(client),
                // The server wants us to redirect to a different address
                Err(Error::Routing { host, port }) => {
                    let mut config = Config::new();

                    config.host(&host);
                    config.port(port);
                    config
                        .authentication(tiberius::AuthMethod::sql_server("sa", "YourPassword123"));

                    let tcp = TcpStream::connect(config.get_addr()).await?;
                    tcp.set_nodelay(true)?;

                    // we should not have more than one redirect, so we'll short-circuit here.
                    tiberius::Client::connect(config, tcp.compat_write()).await
                }
                Err(e) => Err(e),
            }?;

        let mut column_descs = vec![];
        let mut pk_names = vec![];
        {
            let sql = Query::new(format!(
                "SELECT * FROM {} WHERE 1 = 0",
                SqlServerExternalTableReader::get_normalized_table_name(&SchemaTableName {
                    schema_name: config.schema.clone(),
                    table_name: config.table.clone(),
                }),
            ));

            let mut stream = sql.query(&mut client).await?;
            while let Some(item) = stream.try_next().await? {
                match item {
                    QueryItem::Metadata(meta) => {
                        for col in meta.columns() {
                            column_descs.push(ColumnDesc::named(
                                col.name(),
                                ColumnId::placeholder(),
                                type_to_rw_type(&col.column_type())?,
                            ));
                        }
                    }
                    QueryItem::Row(row) => {
                        unreachable!("Unexpected row: {:?}, `SELECT * FROM {} WHERE 1 = 0` should never return rows", row, config.table.clone());
                    }
                }
            }
        }
        {
            let sql = Query::new(format!(
                "SELECT kcu.COLUMN_NAME
                FROM
                    INFORMATION_SCHEMA.TABLE_CONSTRAINTS AS tc
                JOIN
                    INFORMATION_SCHEMA.KEY_COLUMN_USAGE AS kcu
                    ON tc.CONSTRAINT_NAME = kcu.CONSTRAINT_NAME AND
                    tc.TABLE_SCHEMA = kcu.TABLE_SCHEMA AND
                    tc.TABLE_NAME = kcu.TABLE_NAME
                WHERE
                    tc.CONSTRAINT_TYPE = 'PRIMARY KEY' AND
                    tc.TABLE_SCHEMA = '{}' AND tc.TABLE_NAME = '{}'",
                config.schema, config.table,
            ));

            let mut stream = sql.query(&mut client).await?;
            while let Some(item) = stream.try_next().await? {
                match item {
                    QueryItem::Metadata(_) => {}
                    QueryItem::Row(row) => {
                        let pk_name: &str = row.try_get(0)?.unwrap();
                        pk_names.push(pk_name.to_string());
                    }
                }
            }
        }

        Ok(Self {
            column_descs,
            pk_names,
        })
    }

    pub fn column_descs(&self) -> &Vec<ColumnDesc> {
        &self.column_descs
    }

    pub fn pk_names(&self) -> &Vec<String> {
        &self.pk_names
    }
}

fn type_to_rw_type(col_type: &ColumnType) -> ConnectorResult<DataType> {
    let dtype = match col_type {
        ColumnType::Bit => DataType::Boolean,
        ColumnType::Bitn => DataType::Bytea,
        ColumnType::Int1 => DataType::Int16,
        ColumnType::Int2 => DataType::Int16,
        ColumnType::Int4 => DataType::Int32,
        ColumnType::Int8 => DataType::Int64,
        ColumnType::Float4 => DataType::Float32,
        ColumnType::Float8 => DataType::Float64,
        ColumnType::Decimaln | ColumnType::Numericn => DataType::Decimal,
        ColumnType::Daten => DataType::Date,
        ColumnType::Timen => DataType::Time,
        ColumnType::Datetime
        | ColumnType::Datetimen
        | ColumnType::Datetime2
        | ColumnType::Datetime4 => DataType::Timestamp,
        ColumnType::DatetimeOffsetn => DataType::Timestamptz,
        ColumnType::NVarchar | ColumnType::NChar | ColumnType::NText | ColumnType::Text => {
            DataType::Varchar
        }
        // Null, Guid, Image, Money, Money4, Intn, Bitn, Floatn, Xml, Udt, SSVariant, BigVarBin, BigVarChar, BigBinary, BigChar
        mssql_type => {
            // NOTES: user-defined enum type is classified as `Unknown`
            tracing::warn!(
                "Unknown Sql Server data type: {:?}, map to varchar",
                mssql_type
            );
            DataType::Varchar
        }
    };
    Ok(dtype)
}

#[derive(Debug)]
pub struct SqlServerExternalTableReader {
    rw_schema: Schema,
    field_names: String,
    client: tokio::sync::Mutex<tiberius::Client<tokio_util::compat::Compat<TcpStream>>>,
}

impl ExternalTableReader for SqlServerExternalTableReader {
    async fn current_cdc_offset(&self) -> ConnectorResult<CdcOffset> {
        let mut client = self.client.lock().await;
        // start a transaction to read max start_lsn.
        let row = client
            .simple_query(String::from("SELECT sys.fn_cdc_get_max_lsn()"))
            .await?
            .into_row()
            .await?
            .expect("No result returned by `SELECT sys.fn_cdc_get_max_lsn()`");
        // An example of change_lsn or commit_lsn: "00000027:00000ac0:0002" from debezium
        // sys.fn_cdc_get_max_lsn() returns a 10 bytes array, we convert it to a hex string here.
        let max_lsn = match row.try_get::<&[u8], usize>(0)? {
            Some(bytes) => {
                let mut hex_string = String::with_capacity(bytes.len() * 2 + 2);
                assert_eq!(
                    bytes.len(),
                    10,
                    "sys.fn_cdc_get_max_lsn() should return a 10 bytes array."
                );
                for byte in &bytes[0..4] {
                    hex_string.push_str(&format!("{:02x}", byte));
                }
                hex_string.push(':');
                for byte in &bytes[4..8] {
                    hex_string.push_str(&format!("{:02x}", byte));
                }
                hex_string.push(':');
                for byte in &bytes[8..10] {
                    hex_string.push_str(&format!("{:02x}", byte));
                }
                hex_string
            }
            None => bail!("None is returned by `SELECT sys.fn_cdc_get_max_lsn()`, please ensure Sql Server Agent is running."),
        };

        tracing::debug!("current max_lsn: {}", max_lsn);

        Ok(CdcOffset::SqlServer(SqlServerOffset {
            change_lsn: max_lsn,
            commit_lsn: String::from("ffffffff:ffffffff:ffff"),
        }))
    }

    fn snapshot_read(
        &self,
        table_name: SchemaTableName,
        start_pk: Option<OwnedRow>,
        primary_keys: Vec<String>,
        limit: u32,
    ) -> BoxStream<'_, ConnectorResult<OwnedRow>> {
        self.snapshot_read_inner(table_name, start_pk, primary_keys, limit)
    }
}

impl SqlServerExternalTableReader {
    pub async fn new(
        config: ExternalTableConfig,
        rw_schema: Schema,
        pk_indices: Vec<usize>,
        _scan_limit: u32,
    ) -> ConnectorResult<Self> {
        tracing::info!(
            ?rw_schema,
            ?pk_indices,
            "create sql server external table reader"
        );
        let mut client_config = Config::new();

        client_config.host(&config.host);
        client_config.database(&config.database);
        client_config.port(config.port.parse::<u16>().unwrap());
        client_config.authentication(tiberius::AuthMethod::sql_server(
            &config.username,
            &config.password,
        ));
        client_config.trust_cert();
        // TODO(kexiang): add ssl support
        // TODO(kexiang): use trust_cert_ca, trust_cert is not secure
        let tcp = TcpStream::connect(client_config.get_addr()).await?;
        tcp.set_nodelay(true)?;

        let client: tiberius::Client<Compat<TcpStream>> =
            match tiberius::Client::connect(client_config, tcp.compat_write()).await {
                // Connection successful.
                Ok(client) => Ok(client),
                // The server wants us to redirect to a different address
                Err(Error::Routing { host, port }) => {
                    let mut config = Config::new();

                    config.host(&host);
                    config.port(port);
                    config
                        .authentication(tiberius::AuthMethod::sql_server("sa", "YourPassword123"));

                    let tcp = TcpStream::connect(config.get_addr()).await?;
                    tcp.set_nodelay(true)?;

                    // we should not have more than one redirect, so we'll short-circuit here.
                    tiberius::Client::connect(config, tcp.compat_write()).await
                }
                Err(e) => Err(e),
            }?;

        let field_names = rw_schema
            .fields
            .iter()
            .map(|f| Self::quote_column(&f.name))
            .join(",");

        Ok(Self {
            rw_schema,
            field_names,
            client: tokio::sync::Mutex::new(client),
        })
    }

    pub fn get_cdc_offset_parser() -> CdcOffsetParseFunc {
        Box::new(move |offset| {
            Ok(CdcOffset::SqlServer(
                SqlServerOffset::parse_debezium_offset(offset)?,
            ))
        })
    }

    #[try_stream(boxed, ok = OwnedRow, error = ConnectorError)]
    async fn snapshot_read_inner(
        &self,
        table_name: SchemaTableName,
        start_pk_row: Option<OwnedRow>,
        primary_keys: Vec<String>,
        limit: u32,
    ) {
        let order_key = primary_keys
            .iter()
            .map(|col| Self::quote_column(col))
            .join(",");
        let mut sql = Query::new(if start_pk_row.is_none() {
            format!(
                "SELECT {} FROM {} ORDER BY {} OFFSET 0 ROWS FETCH NEXT {limit} ROWS ONLY",
                self.field_names,
                Self::get_normalized_table_name(&table_name),
                order_key,
            )
        } else {
            let filter_expr = Self::filter_expression(&primary_keys);
            format!(
                "SELECT {} FROM {} WHERE {} ORDER BY {} OFFSET 0 ROWS FETCH LIMIT {limit} ROWS ONLY",
                self.field_names,
                Self::get_normalized_table_name(&table_name),
                filter_expr,
                order_key,
            )
        });

        let mut client = self.client.lock().await;

        // FIXME(kexiang): Set session timezone to UTC
        if let Some(pk_row) = start_pk_row {
            let params: Vec<Option<ScalarImpl>> = pk_row.into_iter().collect();
            for param in params {
                // primary key should not be null, so it's safe to unwrap
                sql.bind(param.unwrap());
            }
        }

        let stream = sql.query(&mut client).await?.into_row_stream();

        let row_stream = stream.map(|res| {
            // convert sql server row into OwnedRow
            let mut row = res?;
            Ok::<_, ConnectorError>(sql_server_row_to_owned_row(&mut row, &self.rw_schema))
        });

        pin_mut!(row_stream);

        #[for_await]
        for row in row_stream {
            let row = row?;
            yield row;
        }
    }

    pub fn get_normalized_table_name(table_name: &SchemaTableName) -> String {
        format!(
            "\"{}\".\"{}\"",
            table_name.schema_name, table_name.table_name
        )
    }

    // sql server cannot leverage the given key to narrow down the range of scan,
    // we need to rewrite the comparison conditions by our own.
    // (a, b) > (x, y) => ("a" > @P1) OR (("a" = @P1) AND ("b" > @P2))
    fn filter_expression(columns: &[String]) -> String {
        let mut conditions = vec![];
        // push the first condition
        conditions.push(format!("({} > @P{})", Self::quote_column(&columns[0]), 1));
        for i in 2..=columns.len() {
            // '=' condition
            let mut condition = String::new();
            for (j, col) in columns.iter().enumerate().take(i - 1) {
                if j == 0 {
                    condition.push_str(&format!("({} = @P{})", Self::quote_column(col), j + 1));
                } else {
                    condition.push_str(&format!(
                        " AND ({} = @P{})",
                        Self::quote_column(col),
                        j + 1
                    ));
                }
            }
            // '>' condition
            condition.push_str(&format!(
                " AND ({} > @P{})",
                Self::quote_column(&columns[i - 1]),
                i
            ));
            conditions.push(format!("({})", condition));
        }
        if columns.len() > 1 {
            conditions.join(" OR ")
        } else {
            conditions.join("")
        }
    }

    fn quote_column(column: &str) -> String {
        format!("\"{}\"", column)
    }
}

#[cfg(test)]
mod tests {
    use crate::source::cdc::external::SqlServerExternalTableReader;

    #[test]
    fn test_sql_server_filter_expr() {
        let cols = vec!["id".to_string()];
        let expr = SqlServerExternalTableReader::filter_expression(&cols);
        assert_eq!(expr, "(\"id\" > @P1)");

        let cols = vec!["aa".to_string(), "bb".to_string(), "cc".to_string()];
        let expr = SqlServerExternalTableReader::filter_expression(&cols);
        assert_eq!(
            expr,
            "(\"aa\" > @P1) OR ((\"aa\" = @P1) AND (\"bb\" > @P2)) OR ((\"aa\" = @P1) AND (\"bb\" = @P2) AND (\"cc\" > @P3))"
        );
    }
}
