use cairo_lang_debug::DebugWithDb;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::expr::fmt::ExprFormatter;
use cairo_lang_semantic::test_utils::setup_test_expr;
use cairo_lang_semantic::Expr;
use cairo_lang_test_utils::test_file_test;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;

use crate::semantics::test_utils::DojoSemanticDatabase;

test_file_test!(
    dojo_semantics,
    "src/semantics/test_data",
    {
        get: "get",

        set: "set",
    },
    test_semantics
);

pub fn test_semantics(
    inputs: &OrderedHashMap<String, String>,
    _args: &OrderedHashMap<String, String>,
) -> Result<OrderedHashMap<String, String>, String> {
    let mut db = DojoSemanticDatabase::default();
    let (expr, diagnostics, expr_formatter) = semantics_test_setup(inputs, &mut db);

    if inputs.get("no_diagnostics").is_some() && "" != diagnostics.as_str() {
        return Err("Expanded get!() shouldn't have diagnostic issues.".into());
    }

    if let Some(dojo_semantic) = inputs.get("dojo_semantic") {
        if dojo_semantic.as_str() == "get_success" {
            if let Expr::Block(blk) = &expr {
                blk.tail.expect("Expanded get!() should have a tail");
            } else {
                return Err("Expression should be block".into());
            }
        }
    }

    Ok(OrderedHashMap::from([
        ("expected".into(), format!("{:#?}", expr.debug(&expr_formatter))),
        ("semantic_diagnostics".into(), diagnostics),
    ]))
}

pub fn semantics_test_setup<'a>(
    inputs: &OrderedHashMap<String, String>,
    db: &'a mut DojoSemanticDatabase,
) -> (Expr, String, ExprFormatter<'a>) {
    let (test_expr, diagnostics) = setup_test_expr(
        db,
        inputs["expression"].as_str(),
        inputs.get("setup_code").map(|s| s.as_str()).unwrap_or(""),
        inputs.get("function_code").map(|s| s.as_str()).unwrap_or(""),
    )
    .split();
    let expr = db.expr_semantic(test_expr.function_id, test_expr.expr_id);
    let expr_formatter = ExprFormatter { db, function_id: test_expr.function_id };

    (expr, diagnostics, expr_formatter)
}
