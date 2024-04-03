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

use pgwire::pg_response::{PgResponse, StatementType};
use risingwave_sqlparser::ast::ObjectName;

use super::RwPgResponse;
use crate::error::Result;
use crate::handler::HandlerArgs;

pub async fn handle_close_cursor(
    handler_args: HandlerArgs,
    cursor_name: Option<ObjectName>,
) -> Result<RwPgResponse> {
    if let Some(name) = cursor_name {
        handler_args.session.drop_cursor(name).await?;
    } else {
        handler_args.session.drop_all_cursors().await;
    }
    Ok(PgResponse::empty_result(StatementType::CLOSE_CURSOR))
}
