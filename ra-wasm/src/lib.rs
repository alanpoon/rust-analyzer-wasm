#![cfg(target_arch = "wasm32")]
#![allow(non_snake_case)]

use std::sync::Arc;

use cfg::CfgOptions;
use ide::{
    Analysis, AnalysisHost, Change, CompletionConfig, CrateGraph, CrateId, DiagnosticsConfig,
    Edition, FileId, FilePosition, HoverConfig, HoverDocFormat, Indel, InlayHintsConfig, InlayKind,
    SourceRoot, TextSize,
};
use ide_db::{
    base_db::{CrateName, Dependency, Env, FileSet, VfsPath,AnchoredPath},
    helpers::{
        insert_use::{ImportGranularity, InsertUseConfig, PrefixKind},
        SnippetCap,
    },
    search::SearchScope,
};
use wasm_bindgen::prelude::*;

mod to_proto;
mod indexed_db;
mod return_types;
use return_types::*;
pub use indexed_db::example;
pub use wasm_bindgen_rayon::init_thread_pool;
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    log::info!("worker initialized")
}

#[wasm_bindgen]
pub struct WorldState {
    host: AnalysisHost,
    file_id: FileId,
    crates: js_sys::Array
}

pub fn create_source_root(name: &str, f: FileId) -> SourceRoot {
    let mut file_set = FileSet::default();
    file_set.insert(f, VfsPath::new_virtual_path(format!("/{}/src/lib.rs", name)));
    SourceRoot::new_library(file_set)
}
pub fn create_source_file(name: &str, f: FileId) -> SourceRoot {
    let mut file_set = FileSet::default();
    file_set.insert(f, VfsPath::new_virtual_path(name.to_owned()));
    SourceRoot::new_library(file_set)
}
pub fn create_crate(crate_graph: &mut CrateGraph, f: FileId) -> CrateId {
    let mut cfg = CfgOptions::default();
    cfg.insert_atom("unix".into());
    cfg.insert_key_value("target_arch".into(), "x86_64".into());
    cfg.insert_key_value("target_pointer_width".into(), "64".into());
    crate_graph.add_crate_root(
        f,
        Edition::Edition2018,
        None,
        None,
        cfg,
        Default::default(),
        Env::default(),
        Vec::new(),
    )
}
use std::collections::HashMap;
use toml::Value;
pub fn from_single_file(
    text: String,
    fake_std: String,
    fake_core: String,
    fake_alloc: String,
    crate_hash: js_sys::Array
) -> (AnalysisHost, FileId) {
    
    let mut host = AnalysisHost::default();
    
    let mut change = Change::new();
    let mut crate_graph = CrateGraph::default();
    
    let mut to_create_root_arr = vec![
    ];
    let mut dep_arr = vec![];
    let mut file_id_index = 1;
    for b in crate_hash.iter(){
        if b.is_object(){
            let r_crate_name = js_sys::Reflect::get(&b,&"file_name".into()).unwrap_or(JsValue::NULL);
            let r_anchored = js_sys::Reflect::get(&b,&"anchored".into()).unwrap_or(JsValue::NULL);
            let r_cargo_toml = js_sys::Reflect::get(&b,&"Cargo.toml".into()).unwrap_or(JsValue::NULL);
            if r_crate_name !=JsValue::NULL{
                if let Some(r_crate_name)= r_crate_name.as_string(){
                    //create_source_root_multiple
                    //Vec<(FileId,String)>
                    let mut array_fileid_string = vec![];
                    if r_anchored.is_object(){
                        if let Some(obj) = js_sys::Object::try_from(&r_anchored){
                            for y in js_sys::Object::entries(&obj).iter(){
                                let arr = js_sys::Array::from(&y);
                                if arr.length() >= 2{
                                    let file_id = FileId(file_id_index);
                                    let anchored_path = arr.get(0).as_string().unwrap();
                                    array_fileid_string.push((file_id,anchored_path.clone()));
                                    let text = arr.get(1).as_string().unwrap_or(String::from(""));
                                    change.change_file(file_id.clone(), Some(Arc::new(text)));
                                    if anchored_path.contains("/src/lib.rs"){
                                        let inner_crate = create_crate(&mut crate_graph,file_id);
                                        let inner_dep = Dependency::new(CrateName::new(&r_crate_name).unwrap(), inner_crate);
                                        dep_arr.push(inner_dep);
                                    }
                                    // let s = format!("!!!anchored_path {:?}",anchored_path);
                                    // log(&s);
                                    // let file_id  = FileId(file_id_index);
                                    // let anchored_path = anchored_path.unwrap().to_owned();
                                    // let anchored_path_arr :Vec<&str>= anchored_path.split("./").collect();
                                    // if anchored_path_arr.len() >1{
                                    //     let revert_filename = format!("/{}/src/{}",r_crate_name,anchored_path_arr.get(1).unwrap());
                                    //     to_create_root_arr.push(create_source_file(&anchored_path,file_id));
                                    //     file_set.insert(file_id.clone(), VfsPath::new_virtual_path(anchored_path));
                                    //     change.change_file(file_id.clone(), Some(Arc::new(text)));
                                    // }
                               
                                    file_id_index+=1;
                                }
                            }
                        }
                    }
                    to_create_root_arr.push(create_source_root_multiple(&r_crate_name,array_fileid_string));
                }

            }
            
        }
    }
    let mut file_set = FileSet::default();
    let file_id = FileId(0);
    file_set.insert(file_id, VfsPath::new_virtual_path("/my_crate/main.rs".to_string()));
    let my_crate = create_crate(&mut crate_graph, file_id);
    let source_root = SourceRoot::new_local(file_set.clone());
    let mut source_root_arr = vec![
        source_root
    ];
    source_root_arr.append(&mut to_create_root_arr);
    change.set_roots(source_root_arr);
    change.change_file(file_id, Some(Arc::new(text)));
    for dep in dep_arr{
        crate_graph.add_dep(my_crate, dep).unwrap();
    }
    
    change.set_crate_graph(crate_graph);
    host.apply_change(change);
    log("after apply-change");
    (host, file_id)
}

impl WorldState {
    fn analysis(&self) -> Analysis {
        self.host.analysis()
    }
}
#[wasm_bindgen]
impl WorldState {
    #[wasm_bindgen(constructor)]
    pub fn new(crates:js_sys::Array) -> Self {
        let s = format!("new self.crates.clone() {:?}",crates.clone());
        log(&s);
        let (host, file_id) =
            from_single_file("".to_owned(), "".to_owned(), "".to_owned(), "".to_owned(),crates.clone());
        Self { host, file_id ,crates}
    }
    // #[wasm_bindgen(constructor)]
    // pub async fn new_async() -> Self {
    //     let (host, file_id) =
    //         from_single_file("".to_owned(), "".to_owned(), "".to_owned(), "".to_owned());
    //     Self { host, file_id }
    // }
    pub fn init(&mut self, code: String, fake_std: String, fake_core: String, fake_alloc: String) {
        let (host, file_id) = from_single_file(code, fake_std, fake_core, fake_alloc,self.crates.clone());
        self.host = host;
        self.file_id = file_id;
    }

    pub fn update(&mut self, code: String) -> JsValue {
        log::warn!("update");
        let file_id = FileId(0);
        let mut change = Change::new();
        change.change_file(file_id, Some(Arc::new(code)));
        self.host.apply_change(change);

        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let highlights: Vec<_> = self
            .analysis()
            .highlight(file_id)
            .unwrap()
            .into_iter()
            .map(|hl| Highlight {
                tag: Some(hl.highlight.tag.to_string()),
                range: to_proto::text_range(hl.range, &line_index),
            })
            .collect();

        let config = DiagnosticsConfig::default();

        let diagnostics: Vec<_> = self
            .analysis()
            .diagnostics(&config, ide::AssistResolveStrategy::All, file_id)
            .unwrap()
            .into_iter()
            .map(|d| {
                let Range { startLineNumber, startColumn, endLineNumber, endColumn } =
                    to_proto::text_range(d.range, &line_index);
                Diagnostic {
                    message: d.message,
                    severity: to_proto::severity(d.severity),
                    startLineNumber,
                    startColumn,
                    endLineNumber,
                    endColumn,
                }
            })
            .collect();

        serde_wasm_bindgen::to_value(&UpdateResult { diagnostics, highlights }).unwrap()
    }

    pub fn inlay_hints(&self) -> JsValue {
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();
        let results: Vec<_> = self
            .analysis()
            .inlay_hints(
                &InlayHintsConfig {
                    type_hints: true,
                    parameter_hints: true,
                    chaining_hints: true,
                    max_length: Some(25),
                },
                self.file_id,
            )
            .unwrap()
            .into_iter()
            .map(|ih| InlayHint {
                label: Some(ih.label.to_string()),
                hint_type: match ih.kind {
                    InlayKind::TypeHint | InlayKind::ChainingHint => InlayHintType::Type,
                    InlayKind::ParameterHint => InlayHintType::Parameter,
                },
                range: to_proto::text_range(ih.range, &line_index),
            })
            .collect();
        serde_wasm_bindgen::to_value(&results).unwrap()
    }

    pub fn completions(&self, line_number: u32, column: u32) -> JsValue {
        const COMPLETION_CONFIG: CompletionConfig = CompletionConfig {
            enable_postfix_completions: true,
            enable_imports_on_the_fly: true,
            enable_self_on_the_fly: true,
            add_call_parenthesis: true,
            add_call_argument_snippets: true,
            snippet_cap: SnippetCap::new(true),
            insert_use: InsertUseConfig {
                granularity: ImportGranularity::Module,
                enforce_granularity: false,
                prefix_kind: PrefixKind::Plain,
                group: true,
                skip_glob_imports: false,
            },
            snippets: Vec::new(),
        };

        log::warn!("completions");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let pos = file_position(line_number, column, &line_index, self.file_id);
        let res = match self.analysis().completions(&COMPLETION_CONFIG, pos).unwrap() {
            Some(items) => items,
            None => return JsValue::NULL,
        };

        let items: Vec<_> =
            res.into_iter().map(|item| to_proto::completion_item(item, &line_index)).collect();
        serde_wasm_bindgen::to_value(&items).unwrap()
    }

    pub fn hover(&self, line_number: u32, column: u32) -> JsValue {
        log::warn!("hover");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let range = file_range(line_number, column, line_number, column, &line_index, self.file_id);
        let info = match self
            .analysis()
            .hover(
                &HoverConfig {
                    links_in_hover: true,
                    documentation: Some(HoverDocFormat::Markdown),
                },
                range,
            )
            .unwrap()
        {
            Some(info) => info,
            _ => return JsValue::NULL,
        };

        let value = info.info.markup.to_string();
        let hover = Hover {
            contents: vec![MarkdownString { value }],
            range: to_proto::text_range(info.range, &line_index),
        };

        serde_wasm_bindgen::to_value(&hover).unwrap()
    }

    pub fn code_lenses(&self) -> JsValue {
        log::warn!("code_lenses");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let results: Vec<_> = self
            .analysis()
            .file_structure(self.file_id)
            .unwrap()
            .into_iter()
            .filter(|it| match it.kind {
                ide::StructureNodeKind::SymbolKind(it) => matches!(
                    it,
                    ide_db::SymbolKind::Trait
                        | ide_db::SymbolKind::Struct
                        | ide_db::SymbolKind::Enum
                ),
                ide::StructureNodeKind::Region => true,
            })
            .filter_map(|it| {
                let position =
                    FilePosition { file_id: self.file_id, offset: it.node_range.start() };
                let nav_info = self.analysis().goto_implementation(position).unwrap()?;

                let title = if nav_info.info.len() == 1 {
                    "1 implementation".into()
                } else {
                    format!("{} implementations", nav_info.info.len())
                };

                let positions = nav_info
                    .info
                    .iter()
                    .map(|target| target.focus_range.unwrap_or(target.full_range))
                    .map(|range| to_proto::text_range(range, &line_index))
                    .collect();

                Some(CodeLensSymbol {
                    range: to_proto::text_range(it.node_range, &line_index),
                    command: Some(Command {
                        id: "editor.action.showReferences".into(),
                        title,
                        positions,
                    }),
                })
            })
            .collect();

        serde_wasm_bindgen::to_value(&results).unwrap()
    }

    pub fn references(&self, line_number: u32, column: u32, include_declaration: bool) -> JsValue {
        log::warn!("references");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let pos = file_position(line_number, column, &line_index, self.file_id);
        let search_scope = Some(SearchScope::single_file(self.file_id));
        let ref_results = match self.analysis().find_all_refs(pos, search_scope) {
            Ok(Some(info)) => info,
            _ => return JsValue::NULL,
        };

        let mut res = vec![];
        for ref_result in ref_results {
            if include_declaration {
                if let Some(r) = ref_result.declaration {
                    let r = r.nav.focus_range.unwrap_or(r.nav.full_range);
                    res.push(Highlight { tag: None, range: to_proto::text_range(r, &line_index) });
                }
            }
            ref_result.references.iter().for_each(|(_id, ranges)| {
                // FIXME: handle multiple files
                for (r, _) in ranges {
                    res.push(Highlight { tag: None, range: to_proto::text_range(*r, &line_index) });
                }
            });
        }

        serde_wasm_bindgen::to_value(&res).unwrap()
    }

    pub fn prepare_rename(&self, line_number: u32, column: u32) -> JsValue {
        log::warn!("prepare_rename");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let pos = file_position(line_number, column, &line_index, self.file_id);
        let range_info = match self.analysis().prepare_rename(pos).unwrap() {
            Ok(refs) => refs,
            _ => return JsValue::NULL,
        };

        let range = to_proto::text_range(range_info.range, &line_index);
        let file_text = self.analysis().file_text(self.file_id).unwrap();
        let text = file_text[range_info.range].to_owned();

        serde_wasm_bindgen::to_value(&RenameLocation { range, text }).unwrap()
    }

    pub fn rename(&self, line_number: u32, column: u32, new_name: &str) -> JsValue {
        log::warn!("rename");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let pos = file_position(line_number, column, &line_index, self.file_id);
        let change = match self.analysis().rename(pos, new_name).unwrap() {
            Ok(change) => change,
            Err(_) => return JsValue::NULL,
        };

        let result: Vec<_> = change
            .source_file_edits
            .iter()
            .flat_map(|(_, edit)| edit.iter())
            .map(|atom: &Indel| to_proto::text_edit(atom, &line_index))
            .collect();

        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    pub fn signature_help(&self, line_number: u32, column: u32) -> JsValue {
        log::warn!("signature_help");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let pos = file_position(line_number, column, &line_index, self.file_id);
        let call_info = match self.analysis().call_info(pos) {
            Ok(Some(call_info)) => call_info,
            _ => return JsValue::NULL,
        };

        let active_parameter = call_info.active_parameter;
        let sig_info = to_proto::signature_information(call_info);

        let result = SignatureHelp {
            signatures: [sig_info],
            activeSignature: 0,
            activeParameter: active_parameter,
        };
        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    pub fn definition(&self, line_number: u32, column: u32) -> JsValue {
        log::warn!("definition");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let pos = file_position(line_number, column, &line_index, self.file_id);
        let nav_info = match self.analysis().goto_definition(pos) {
            Ok(Some(nav_info)) => nav_info,
            _ => return JsValue::NULL,
        };

        let res = to_proto::location_links(nav_info, &line_index);
        serde_wasm_bindgen::to_value(&res).unwrap()
    }

    pub fn type_definition(&self, line_number: u32, column: u32) -> JsValue {
        log::warn!("type_definition");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let pos = file_position(line_number, column, &line_index, self.file_id);
        let nav_info = match self.analysis().goto_type_definition(pos) {
            Ok(Some(nav_info)) => nav_info,
            _ => return JsValue::NULL,
        };

        let res = to_proto::location_links(nav_info, &line_index);
        serde_wasm_bindgen::to_value(&res).unwrap()
    }

    pub fn document_symbols(&self) -> JsValue {
        log::warn!("document_symbols");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let struct_nodes = match self.analysis().file_structure(self.file_id) {
            Ok(struct_nodes) => struct_nodes,
            _ => return JsValue::NULL,
        };
        let mut parents: Vec<(DocumentSymbol, Option<usize>)> = Vec::new();

        for symbol in struct_nodes {
            let doc_symbol = DocumentSymbol {
                name: symbol.label.clone(),
                detail: symbol.detail.unwrap_or(symbol.label),
                kind: to_proto::symbol_kind(symbol.kind),
                range: to_proto::text_range(symbol.node_range, &line_index),
                children: None,
                tags: [if symbol.deprecated { SymbolTag::Deprecated } else { SymbolTag::None }],
                containerName: None,
                selectionRange: to_proto::text_range(symbol.navigation_range, &line_index),
            };
            parents.push((doc_symbol, symbol.parent));
        }
        let mut res = Vec::new();
        while let Some((node, parent)) = parents.pop() {
            match parent {
                None => res.push(node),
                Some(i) => {
                    let children = &mut parents[i].0.children;
                    if children.is_none() {
                        *children = Some(Vec::new());
                    }
                    children.as_mut().unwrap().push(node);
                }
            }
        }

        serde_wasm_bindgen::to_value(&res).unwrap()
    }

    pub fn type_formatting(&self, line_number: u32, column: u32, ch: char) -> JsValue {
        log::warn!("type_formatting");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let mut pos = file_position(line_number, column, &line_index, self.file_id);
        pos.offset -= TextSize::of('.');

        let edit = self.analysis().on_char_typed(pos, ch);

        let (_file, edit) = match edit {
            Ok(Some(it)) => it.source_file_edits.into_iter().next().unwrap(),
            _ => return JsValue::NULL,
        };

        let change: Vec<TextEdit> = to_proto::text_edits(edit, &line_index);
        serde_wasm_bindgen::to_value(&change).unwrap()
    }

    pub fn folding_ranges(&self) -> JsValue {
        log::warn!("folding_ranges");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();
        if let Ok(folds) = self.analysis().folding_ranges(self.file_id) {
            let res: Vec<_> =
                folds.into_iter().map(|fold| to_proto::folding_range(fold, &line_index)).collect();
            serde_wasm_bindgen::to_value(&res).unwrap()
        } else {
            JsValue::NULL
        }
    }

    pub fn goto_implementation(&self, line_number: u32, column: u32) -> JsValue {
        log::warn!("goto_implementation");
        let line_index = self.analysis().file_line_index(self.file_id).unwrap();

        let pos = file_position(line_number, column, &line_index, self.file_id);
        let nav_info = match self.analysis().goto_implementation(pos) {
            Ok(Some(it)) => it,
            _ => return JsValue::NULL,
        };
        let res = to_proto::location_links(nav_info, &line_index);
        serde_wasm_bindgen::to_value(&res).unwrap()
    }
}

// impl Default for WorldState {
//     fn default() -> Self {
//         Self::new()
//     }
// }

fn file_position(
    line_number: u32,
    column: u32,
    line_index: &ide::LineIndex,
    file_id: ide::FileId,
) -> ide::FilePosition {
    let line_col = ide::LineCol { line: line_number - 1, col: column - 1 };
    let offset = line_index.offset(line_col);
    ide::FilePosition { file_id, offset }
}

fn file_range(
    start_line_number: u32,
    start_column: u32,
    end_line_number: u32,
    end_column: u32,
    line_index: &ide::LineIndex,
    file_id: ide::FileId,
) -> ide::FileRange {
    let start_line_col = ide::LineCol { line: start_line_number - 1, col: start_column - 1 };
    let end_line_col = ide::LineCol { line: end_line_number - 1, col: end_column - 1 };
    ide::FileRange {
        file_id,
        range: ide::TextRange::new(
            line_index.offset(start_line_col),
            line_index.offset(end_line_col),
        ),
    }
}
pub fn create_source_root_multiple(name: &str, f: Vec<(FileId,String)>) -> SourceRoot {
    let mut file_set = FileSet::default();
    for (f,p) in f{
        file_set.insert(f, VfsPath::new_virtual_path(p));
    }
    SourceRoot::new_library(file_set)
}