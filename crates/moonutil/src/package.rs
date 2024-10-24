// moon: The build system and package manager for MoonBit.
// Copyright (C) 2024 International Digital Economy Academy
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// For inquiries, you can contact us via e-mail at jichuruanjian@idea.edu.cn.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use colored::Colorize;
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    common::{FileName, GeneratedTestDriver, MOONBITLANG_CORE},
    cond_expr::{CompileCondition, CondExpr, RawTargets},
    path::{ImportComponent, PathComponent},
};

#[derive(Debug, Clone)]
pub struct Package {
    pub is_main: bool,
    pub need_link: bool,
    pub is_third_party: bool,
    pub root_path: PathBuf,
    pub root: PathComponent,
    pub rel: PathComponent,
    // *.mbt (exclude the following)
    pub files: IndexMap<PathBuf, CompileCondition>,
    //  *_wbtest.mbt
    pub wbtest_files: IndexMap<PathBuf, CompileCondition>,
    //  *_test.mbt
    pub test_files: IndexMap<PathBuf, CompileCondition>,
    pub files_contain_test_block: Vec<PathBuf>,
    pub imports: Vec<ImportComponent>,
    pub wbtest_imports: Vec<ImportComponent>,
    pub test_imports: Vec<ImportComponent>,
    pub generated_test_drivers: Vec<GeneratedTestDriver>,
    pub artifact: PathBuf,

    pub link: Option<Link>,
    pub warn_list: Option<String>,
    pub alert_list: Option<String>,

    pub targets: Option<IndexMap<FileName, CondExpr>>,
    pub pre_build: Option<Vec<MoonPkgGenerate>>,
}

impl Package {
    pub fn full_name(&self) -> String {
        if self.rel.full_name().is_empty() {
            self.root.full_name()
        } else {
            format!("{}/{}", self.root.full_name(), self.rel.full_name())
        }
    }

    pub fn last_name(&self) -> &str {
        if self.rel.components.is_empty() {
            self.root.components.last().unwrap()
        } else {
            self.rel.components.last().unwrap()
        }
    }

    pub fn full_components(&self) -> PathComponent {
        let mut comps = self.root.components.clone();
        comps.extend(self.rel.components.iter().cloned());
        PathComponent { components: comps }
    }

    pub fn get_all_files(&self) -> Vec<String> {
        let mut files =
            Vec::with_capacity(self.files.len() + self.test_files.len() + self.wbtest_files.len());
        files.extend(
            self.files
                .keys()
                .chain(self.test_files.keys())
                .chain(self.wbtest_files.keys())
                .map(|x| x.file_name().unwrap().to_str().unwrap().to_string()),
        );
        files
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PackageJSON {
    pub is_main: bool,
    pub is_third_party: bool,
    pub root_path: String,
    pub root: String,
    pub rel: String,
    pub files: IndexMap<PathBuf, CompileCondition>,
    // white box test
    pub wbtest_files: IndexMap<PathBuf, CompileCondition>,
    // black box test
    pub test_files: IndexMap<PathBuf, CompileCondition>,
    pub deps: Vec<AliasJSON>,
    pub wbtest_deps: Vec<AliasJSON>,
    pub test_deps: Vec<AliasJSON>,
    pub artifact: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AliasJSON {
    pub path: String,
    pub alias: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum PkgJSONImport {
    /// Path and alias of an imported package
    #[schemars(with = "std::collections::HashMap<String, Option<String>>")]
    Map(IndexMap<String, Option<String>>),
    List(Vec<PkgJSONImportItem>),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum PkgJSONImportItem {
    String(String),
    Object { path: String, alias: String },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum BoolOrLink {
    Bool(bool),
    Link(Box<Link>),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(
    title = "JSON schema for MoonBit moon.pkg.json files",
    description = "A package of MoonBit language"
)]
pub struct MoonPkgJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Specify whether this package is a main package or not
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "is-main")]
    #[serde(alias = "is_main")]
    #[serde(rename(serialize = "is-main"))]
    #[schemars(rename = "is-main")]
    pub is_main: Option<bool>,

    /// Imported packages of the package
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<PkgJSONImport>,

    /// White box test imported packages of the package
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "wbtest_import")]
    #[serde(alias = "wbtest-import")]
    #[schemars(rename = "wbtest-import")]
    pub wbtest_import: Option<PkgJSONImport>,

    /// Black box test imported packages of the package
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "test_import")]
    #[serde(alias = "test-import")]
    #[schemars(rename = "test-import")]
    pub test_import: Option<PkgJSONImport>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<BoolOrLink>,

    /// Warn list setting of the package
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "warn-list")]
    #[serde(alias = "warn_list")]
    #[schemars(rename = "warn-list")]
    pub warn_list: Option<String>,

    /// Alert list setting of the package
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "alert-list")]
    #[serde(alias = "alert_list")]
    #[schemars(rename = "alert-list")]
    pub alert_list: Option<String>,

    /// Conditional compilation targets
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "targets")]
    #[schemars(rename = "targets")]
    #[schemars(with = "Option<std::collections::HashMap<String, StringOrArray>>")]
    pub targets: Option<RawTargets>,

    /// Command for moon generate
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "pre-build")]
    #[schemars(rename = "pre-build")]
    pub pre_build: Option<Vec<MoonPkgGenerate>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[schemars(rename = "import-memory")]
pub struct ImportMemory {
    pub module: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct WasmLinkConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exports: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "heap-start-address")]
    pub heap_start_address: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "import-memory")]
    pub import_memory: Option<ImportMemory>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "export-memory-name")]
    pub export_memory_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct NativeLinkConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct WasmGcLinkConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exports: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "import-memory")]
    pub import_memory: Option<ImportMemory>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "export-memory-name")]
    pub export_memory_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct JsLinkConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exports: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<JsFormat>,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[repr(u8)]
pub enum JsFormat {
    #[default]
    #[serde(rename = "esm")]
    ESM,
    #[serde(rename = "cjs")]
    CJS,
    #[serde(rename = "iife")]
    IIFE,
}

impl JsFormat {
    pub fn to_flag(&self) -> &'static str {
        match self {
            JsFormat::ESM => "esm",
            JsFormat::CJS => "cjs",
            JsFormat::IIFE => "iife",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Link {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wasm: Option<WasmLinkConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "wasm-gc")]
    pub wasm_gc: Option<WasmGcLinkConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub js: Option<JsLinkConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<NativeLinkConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct MoonPkgGenerate {
    pub input: StringOrArray,
    pub output: StringOrArray,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum StringOrArray {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoonPkg {
    pub name: Option<String>,
    pub is_main: bool,
    pub need_link: bool,
    pub imports: Vec<Import>,
    pub core_imports: Vec<Import>,
    pub wbtest_imports: Vec<Import>,
    pub core_wbtest_imports: Vec<Import>,
    pub test_imports: Vec<Import>,
    pub core_test_imports: Vec<Import>,

    pub link: Option<Link>,
    pub warn_list: Option<String>,
    pub alert_list: Option<String>,

    pub targets: Option<RawTargets>,

    pub pre_build: Option<Vec<MoonPkgGenerate>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Import {
    Simple(String),
    Alias { path: String, alias: String },
}

impl Import {
    pub fn get_path(&self) -> &str {
        match self {
            Self::Simple(v) => v,
            Self::Alias { path, alias: _ } => path,
        }
    }
}

pub fn convert_pkg_json_to_package(module_name: &str, j: MoonPkgJSON) -> anyhow::Result<MoonPkg> {
    let get_imports = |source: Option<PkgJSONImport>| -> Vec<Import> {
        let mut imports = vec![];
        if let Some(im) = source {
            match im {
                PkgJSONImport::Map(m) => {
                    for (k, v) in m.into_iter() {
                        match &v {
                            None => imports.push(Import::Simple(k)),
                            Some(p) => {
                                if p.is_empty() {
                                    imports.push(Import::Simple(k));
                                } else {
                                    imports.push(Import::Alias {
                                        path: k,
                                        alias: v.unwrap(),
                                    })
                                }
                            }
                        }
                    }
                }
                PkgJSONImport::List(l) => {
                    for item in l.into_iter() {
                        match item {
                            PkgJSONImportItem::String(s) => imports.push(Import::Simple(s)),
                            PkgJSONImportItem::Object { path, alias } => {
                                if alias.is_empty() {
                                    imports.push(Import::Simple(path));
                                } else {
                                    imports.push(Import::Alias { path, alias })
                                }
                            }
                        }
                    }
                }
            }
        };
        imports
    };

    let imports = get_imports(j.import);
    let wbtest_imports = get_imports(j.wbtest_import);
    let test_imports = get_imports(j.test_import);

    let mut is_main = j.is_main.unwrap_or(false);
    if let Some(name) = &j.name {
        if name == "main" {
            is_main = true;
            eprintln!(
                "{}",
                "Warning: The `name` field in `moon.pkg.json` is now deprecated. For the main package, please use `\"is-main\": true` instead. Refer to the latest documentation at https://www.moonbitlang.com/docs/build-system-tutorial for more information.".yellow()
                    .bold()
            );
        }
    }
    let need_link = match &j.link {
        None => false,
        Some(BoolOrLink::Bool(b)) => *b,
        Some(BoolOrLink::Link(_)) => true,
    };

    // TODO: check on the fly
    let mut alias_dedup: HashSet<String> = HashSet::new();
    for item in imports.iter() {
        let alias = match item {
            Import::Simple(p) => {
                let alias = Path::new(p)
                    .file_stem()
                    .context(format!("failed to get alias of `{}`", p))?
                    .to_str()
                    .unwrap()
                    .to_string();
                alias
            }
            Import::Alias { path: _path, alias } => alias.clone(),
        };
        if alias_dedup.contains(&alias) {
            bail!("Duplicate alias `{}`", alias);
        } else {
            alias_dedup.insert(alias.clone());
        }
    }
    let mut imports_with_core = vec![];
    let mut imports_without_core = vec![];
    for im in imports {
        let path = match &im {
            Import::Simple(p) => p,
            Import::Alias { path, alias: _ } => path,
        };
        if module_name != MOONBITLANG_CORE && path.starts_with(MOONBITLANG_CORE) {
            imports_with_core.push(im);
        } else {
            imports_without_core.push(im);
        }
    }

    // TODO: check on the fly
    let mut alias_dedup: HashSet<String> = HashSet::new();
    for item in wbtest_imports.iter() {
        let alias = match item {
            Import::Simple(p) => {
                let alias = Path::new(p)
                    .file_stem()
                    .context(format!("failed to get alias of `{}`", p))?
                    .to_str()
                    .unwrap()
                    .to_string();
                alias
            }
            Import::Alias { path: _path, alias } => alias.clone(),
        };
        if alias_dedup.contains(&alias) {
            bail!("Duplicate alias `{}`", alias);
        } else {
            alias_dedup.insert(alias.clone());
        }
    }
    let mut wbtest_imports_with_core = vec![];
    let mut wbtest_imports_without_core = vec![];
    for im in wbtest_imports {
        let path = match &im {
            Import::Simple(p) => p,
            Import::Alias { path, alias: _ } => path,
        };
        if module_name != MOONBITLANG_CORE && path.starts_with(MOONBITLANG_CORE) {
            wbtest_imports_with_core.push(im);
        } else {
            wbtest_imports_without_core.push(im);
        }
    }

    // TODO: check on the fly
    let mut alias_dedup: HashSet<String> = HashSet::new();
    for item in test_imports.iter() {
        let alias = match item {
            Import::Simple(p) => {
                let alias = Path::new(p)
                    .file_stem()
                    .context(format!("failed to get alias of `{}`", p))?
                    .to_str()
                    .unwrap()
                    .to_string();
                alias
            }
            Import::Alias { path: _path, alias } => alias.clone(),
        };
        if alias_dedup.contains(&alias) {
            bail!("Duplicate alias `{}`", alias);
        } else {
            alias_dedup.insert(alias.clone());
        }
    }
    let mut test_imports_with_core = vec![];
    let mut test_imports_without_core = vec![];
    for im in test_imports {
        let path = match &im {
            Import::Simple(p) => p,
            Import::Alias { path, alias: _ } => path,
        };
        if module_name != MOONBITLANG_CORE && path.starts_with(MOONBITLANG_CORE) {
            test_imports_with_core.push(im);
        } else {
            test_imports_without_core.push(im);
        }
    }

    let result = MoonPkg {
        name: None,
        is_main,
        need_link,
        imports: imports_without_core,
        core_imports: imports_with_core,
        wbtest_imports: wbtest_imports_without_core,
        core_wbtest_imports: wbtest_imports_with_core,
        test_imports: test_imports_without_core,
        core_test_imports: test_imports_with_core,
        link: match j.link {
            None => None,
            Some(BoolOrLink::Bool(_)) => None,
            Some(BoolOrLink::Link(l)) => Some(*l),
        },
        warn_list: j.warn_list,
        alert_list: j.alert_list,
        targets: j.targets,
        pre_build: j.pre_build,
    };
    Ok(result)
}

#[test]
fn validate_pkg_json_schema() {
    let schema = schemars::schema_for!(MoonPkgJSON);
    let actual = &serde_json_lenient::to_string_pretty(&schema).unwrap();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../moonbuild/template/pkg.schema.json"
    );
    expect_test::expect_file![path].assert_eq(actual);

    let html_template_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../moonbuild/template/pkg_json_schema.html"
    );
    let html_template = std::fs::read_to_string(html_template_path).unwrap();
    let content = html_template.replace("const schema = {}", &format!("const schema = {}", actual));
    let html_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/manual/src/source/pkg_json_schema.html"
    );
    std::fs::write(html_path, &content).unwrap();

    let zh_html_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/manual-zh/src/source/pkg_json_schema.html"
    );
    std::fs::write(zh_html_path, content).unwrap();
}
