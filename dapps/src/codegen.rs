// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

extern crate aster;
extern crate glob;
extern crate mime_guess;

use self::mime_guess::guess_mime_type;
use std::path::{self, Path, PathBuf};
use std::ops::Deref;

use syntax::ast::{MetaItem, Item};

use syntax::ast;
use syntax::attr;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;
use syntax::print::pprust::{lit_to_string};
use syntax::parse::token::{InternedString};


pub fn expand_webapp_implementation(
	cx: &mut ExtCtxt,
	span: Span,
	meta_item: &MetaItem,
	annotatable: &Annotatable,
	push: &mut FnMut(Annotatable)
	) {
		let item = match *annotatable {
			Annotatable::Item(ref item) => item,
			_ => {
				cx.span_err(meta_item.span, "`#[derive(WebAppFiles)]` may only be applied to struct implementations");
				return;
			},
		};

		let builder = aster::AstBuilder::new().span(span);

		implement_webapp(cx, &builder, &item, push);
	}

fn implement_webapp(cx: &ExtCtxt, builder: &aster::AstBuilder, item: &Item, push: &mut FnMut(Annotatable)) {

	let static_files_dir = extract_path(cx, item);

	let src = Path::new("src");
	let static_files = {
		let mut buf = src.to_path_buf();
		buf.push(static_files_dir.deref());
		buf
	};

	let search_location = {
		let mut buf = static_files.to_path_buf();
		buf.push("**");
		buf.push("*");
		buf
	};

	let files = glob::glob(search_location.to_str().expect("Valid UTF8 path"))
		.expect("The sources directory is missing.")
		.collect::<Result<Vec<PathBuf>, glob::GlobError>>()
		.expect("There should be no error when reading a list of files.");

	let statements = files
		.iter()
		.filter(|path_buf| path_buf.is_file())
		.map(|path_buf| {
			let path = path_buf.as_path();
			let filename = path.file_name().and_then(|s| s.to_str()).expect("Only UTF8 paths.");
			let mime_type = guess_mime_type(filename).to_string();
			let file_path = as_uri(path.strip_prefix(&static_files).ok().expect("Prefix is always there, cause it's absolute path;qed"));
			let file_path_in_source = path.to_str().expect("Only UTF8 paths.");

			let path_lit = builder.expr().str(file_path.as_str());
			let mime_lit = builder.expr().str(mime_type.as_str());
			let web_path_lit = builder.expr().str(file_path_in_source);
			let separator_lit = builder.expr().str(path::MAIN_SEPARATOR.to_string().as_str());
			let concat_id = builder.id("concat!");
			let env_id = builder.id("env!");
			let macro_id = builder.id("include_bytes!");

			let content = quote_expr!(cx,
									  $macro_id($concat_id($env_id("CARGO_MANIFEST_DIR"), $separator_lit, $web_path_lit))
									 );
			quote_stmt!(cx,
						files.insert($path_lit, File { path: $path_lit, content_type: $mime_lit, content: $content });
					   ).expect("The statement is always ok, because it just uses literals.")
		}).collect::<Vec<ast::Stmt>>();

	let type_name = item.ident;

	let files_impl = quote_item!(cx,
								   impl $type_name {
									   fn files() -> ::std::collections::HashMap<&'static str, File> {
										   let mut files = ::std::collections::HashMap::new();
										   $statements
										   files
									   }
								   }
								  ).unwrap();

	push(Annotatable::Item(files_impl));
}

fn extract_path(cx: &ExtCtxt, item: &Item) -> String {
	for meta_items in item.attrs().iter().filter_map(webapp_meta_items) {
		for meta_item in meta_items {
			match meta_item.node {
				ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"path" => {
					if let Some(s) = get_str_from_lit(cx, name, lit) {
						return s.deref().to_owned();
					}
				},
				_ => {},
			}
		}
	}

	// default
	"web".to_owned()
}

fn get_str_from_lit(cx: &ExtCtxt, name: &str, lit: &ast::Lit) -> Option<InternedString> {
	match lit.node {
		ast::LitKind::Str(ref s, _) => Some(s.clone()),
		_ => {
			cx.span_err(
				lit.span,
				&format!("webapp annotation `{}` must be a string, not `{}`",
						 name,
						 lit_to_string(lit)));

			return None;
		}
	}
}

fn webapp_meta_items(attr: &ast::Attribute) -> Option<&[P<ast::MetaItem>]> {
	match attr.node.value.node {
		ast::MetaItemKind::List(ref name, ref items) if name == &"webapp" => {
			attr::mark_used(&attr);
			Some(items)
		}
		_ => None
	}
}

fn as_uri(path: &Path) -> String {
	let mut s = String::new();
	for component in path.iter() {
		s.push_str(component.to_str().expect("Only UTF-8 filenames are supported."));
		s.push('/');
	}
	s[0..s.len()-1].into()
}


#[test]
fn should_convert_path_separators_on_all_platforms() {
	// given
	let p = {
		let mut p = PathBuf::new();
		p.push("web");
		p.push("src");
		p.push("index.html");
		p
	};

	// when
	let path = as_uri(&p);

	// then
	assert_eq!(path, "web/src/index.html".to_owned());
}
