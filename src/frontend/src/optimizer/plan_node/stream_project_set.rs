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

use fixedbitset::FixedBitSet;
use itertools::Itertools;
use risingwave_pb::stream_plan::stream_node::PbNodeBody;
use risingwave_pb::stream_plan::ProjectSetNode;

use super::stream::prelude::*;
use super::utils::impl_distill_by_unit;
use super::{generic, ExprRewritable, PlanBase, PlanRef, PlanTreeNodeUnary, StreamNode};
use crate::expr::{ExprRewriter, ExprVisitor};
use crate::optimizer::plan_node::expr_visitable::ExprVisitable;
use crate::optimizer::property::{analyze_monotonicity, monotonicity_variants, MonotonicityMap};
use crate::stream_fragmenter::BuildFragmentGraphState;
use crate::utils::ColIndexMappingRewriteExt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamProjectSet {
    pub base: PlanBase<Stream>,
    core: generic::ProjectSet<PlanRef>,
    /// All the watermark derivations, (`input_column_idx`, `expr_idx`). And the
    /// derivation expression is the `project_set`'s expression itself.
    watermark_derivations: Vec<(usize, usize)>,
    /// Nondecreasing expression indices. `ProjectSet` can produce watermarks for these
    /// expressions.
    nondecreasing_exprs: Vec<usize>,
}

impl StreamProjectSet {
    pub fn new(core: generic::ProjectSet<PlanRef>) -> Self {
        let input = core.input.clone();
        let distribution = core
            .i2o_col_mapping()
            .rewrite_provided_distribution(input.distribution());

        let mut out_watermark_columns = FixedBitSet::with_capacity(core.output_len());
        let mut out_monotonicity_map = MonotonicityMap::new();
        let mut watermark_derivations = vec![];
        let mut nondecreasing_exprs = vec![];
        for (expr_idx, expr) in core.select_list.iter().enumerate() {
            let out_expr_idx = expr_idx + 1;

            use monotonicity_variants::*;
            match analyze_monotonicity(expr) {
                Inherent(monotonicity) => {
                    if matches!(monotonicity, NonDecreasing | Constant) {
                        // We can only propagate non-decreasing/constant monotonicity, because we will produce
                        // NULLs after all values of the column are consumed. Only non-decreasing/constant
                        // monotonicity can hold even after appending several NULLs.
                        out_monotonicity_map.insert(out_expr_idx, monotonicity);
                    }
                    if monotonicity.is_non_decreasing() {
                        nondecreasing_exprs.push(expr_idx); // to produce watermarks
                        out_watermark_columns.insert(out_expr_idx);
                    }
                }
                FollowingInput(input_idx) => {
                    let in_monotonicity = input.columns_monotonicity()[input_idx];
                    if matches!(in_monotonicity, NonDecreasing | Constant) {
                        // same as above
                        out_monotonicity_map.insert(out_expr_idx, in_monotonicity);
                    }
                    if input.watermark_columns().contains(input_idx) {
                        watermark_derivations.push((input_idx, expr_idx)); // to propagate watermarks
                        out_watermark_columns.insert(out_expr_idx);
                    }
                }
                _FollowingInputInversely(_) => {}
            }
        }

        // ProjectSet executor won't change the append-only behavior of the stream, so it depends on
        // input's `append_only`.
        let base = PlanBase::new_stream_with_core(
            &core,
            distribution,
            input.append_only(),
            input.emit_on_window_close(),
            out_watermark_columns,
            out_monotonicity_map,
        );
        StreamProjectSet {
            base,
            core,
            watermark_derivations,
            nondecreasing_exprs,
        }
    }
}
impl_distill_by_unit!(StreamProjectSet, core, "StreamProjectSet");
impl_plan_tree_node_for_unary! { StreamProjectSet }

impl PlanTreeNodeUnary for StreamProjectSet {
    fn input(&self) -> PlanRef {
        self.core.input.clone()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        let mut core = self.core.clone();
        core.input = input;
        Self::new(core)
    }
}

impl StreamNode for StreamProjectSet {
    fn to_stream_prost_body(&self, _state: &mut BuildFragmentGraphState) -> PbNodeBody {
        let (watermark_input_cols, watermark_expr_indices) = self
            .watermark_derivations
            .iter()
            .map(|(i, o)| (*i as u32, *o as u32))
            .unzip();
        PbNodeBody::ProjectSet(ProjectSetNode {
            select_list: self
                .core
                .select_list
                .iter()
                .map(|select_item| select_item.to_project_set_select_item_proto())
                .collect_vec(),
            watermark_input_cols,
            watermark_expr_indices,
            nondecreasing_exprs: self.nondecreasing_exprs.iter().map(|i| *i as _).collect(),
        })
    }
}

impl ExprRewritable for StreamProjectSet {
    fn has_rewritable_expr(&self) -> bool {
        true
    }

    fn rewrite_exprs(&self, r: &mut dyn ExprRewriter) -> PlanRef {
        let mut core = self.core.clone();
        core.rewrite_exprs(r);
        Self::new(core).into()
    }
}

impl ExprVisitable for StreamProjectSet {
    fn visit_exprs(&self, v: &mut dyn ExprVisitor) {
        self.core.visit_exprs(v);
    }
}
