use std::sync::Arc;

use cairo_lang_defs::plugin::{MacroPlugin, PluginGeneratedFile};
use cairo_lang_diagnostics::{format_diagnostics, DiagnosticLocation};
use cairo_lang_filesystem::cfg::CfgSet;
use cairo_lang_filesystem::db::FilesGroup;
use cairo_lang_formatter::format_string;
use cairo_lang_parser::test_utils::create_virtual_file;
use cairo_lang_parser::utils::{get_syntax_file_and_diagnostics, SimpleParserDatabase};
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;

use crate::plugin::DojoPlugin;

cairo_lang_test_utils::test_file_test!(
    expand_plugin,
    "src/plugin_test_data",
    {
        model: "model",
        print: "print",
        introspect: "introspect",
        system: "system",
    },
    test_expand_plugin
);

pub fn test_expand_plugin(
    inputs: &OrderedHashMap<String, String>,
    _args: &OrderedHashMap<String, String>,
) -> Result<OrderedHashMap<String, String>, String> {
    let db = &mut SimpleParserDatabase::default();

    let cfg_set: Option<CfgSet> =
        inputs.get("cfg").map(|s| serde_json::from_str(s.as_str()).unwrap());
    if let Some(cfg_set) = cfg_set {
        db.set_cfg_set(Arc::new(cfg_set));
    }

    let cairo_code = &inputs["cairo_code"];
    let file_id = create_virtual_file(db, "dummy_file.cairo", cairo_code);

    let (syntax_file, diagnostics) = get_syntax_file_and_diagnostics(db, file_id, cairo_code);
    assert!(diagnostics.is_empty(), "Unexpected diagnostics:\n{}", diagnostics.format(db));

    let plugins: Vec<Arc<dyn MacroPlugin>> = vec![Arc::new(DojoPlugin)];
    let mut generated_items: Vec<String> = Vec::new();
    let mut diagnostic_items: Vec<String> = Vec::new();

    for item in syntax_file.items(db).elements(db).into_iter() {
        let mut remove_original_item = false;
        let mut local_generated_items = Vec::<String>::new();
        for plugin in &plugins {
            let result = plugin.generate_code(db, item.clone());

            diagnostic_items.extend(result.diagnostics.iter().map(|diag| {
                let syntax_node = diag.stable_ptr.lookup(db);

                let location =
                    DiagnosticLocation { file_id, span: syntax_node.span_without_trivia(db) };
                format_diagnostics(db, &diag.message, location)
            }));

            if result.remove_original_item {
                remove_original_item = true;
            }

            if let Some(PluginGeneratedFile { content, .. }) = result.code {
                local_generated_items.push(format_string(db, content));
                break;
            }

            if remove_original_item {
                break;
            }
        }

        if !remove_original_item {
            generated_items.push(item.as_syntax_node().get_text(db));
        }

        generated_items.extend(local_generated_items);
    }

    Ok(OrderedHashMap::from([
        ("generated_cairo_code".into(), generated_items.join("\n")),
        ("expected_diagnostics".into(), diagnostic_items.join("\n")),
    ]))
}
