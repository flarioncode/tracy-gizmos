#![deny(missing_docs)]

//! A procedural macro attribute for instrumenting functions with
//! [`tracy-gizmos`] zones.
//!
//! ## Usage
//!
//! In the `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tracy-gizmos-attributes = "0.0.1"
//! ```
//!
//! The [`#[instrument]`][instrument] attribute can now be added to a
//! function to automatically create and enter a `tracy-gizmos` [zone]
//! when that function is called. For example:
//!
//! ```no_run
//! #[tracy_gizmos_attributes::instrument]
//! fn work() {
//!     // do stuff
//! }
//! ```
//!
//! [`tracy-gizmos`]: https://crates.io/crates/tracy-gizmos
//! [zone]: https://docs.rs/tracy-gizmos/latest/tracy_gizmos/struct.Zone.html
//! [instrument]: macro@self::instrument

use proc_macro::{
	TokenStream,
	TokenTree,
	Span,
	Delimiter,
	Spacing,
	Group,
	Ident,
	Literal,
	Punct,
};

/// Instruments a function to create and start a profiling capture
/// session.
///
/// Session will end automatically at the end of the function' scope.
///
/// *Note*: This will also [`macro@instrument`] the function automatically.
///
/// ## Examples
///
/// ```
/// # use tracy_gizmos_attributes::{capture, instrument};
/// #[capture]
/// fn main() {
///     work();
/// }
///
/// #[instrument]
/// fn work() {
///    // do stuff
/// }
/// ```
#[proc_macro_attribute]
pub fn capture(_attr: TokenStream, item: TokenStream) -> TokenStream {
	// Cloning a `TokenStream` is cheap since it's reference counted
	// internally.
	let with_capture = try_capture(item.clone());
	// We chain both error and original item, to prevent the
	// generation of two compilation errors: one from us and another
	// one (or multiple, even) caused by original item being skipped.
	match with_capture {
		Ok(item) => item,
		Err(e)   => TokenStream::from_iter(e.to_compile_error().into_iter().chain(item)),
	}
}

fn try_capture(item: TokenStream) -> Result<TokenStream, Error> {
	let mut tokens: Vec<TokenTree> = item.into_iter().collect();
	let mut tokens_it              = tokens.iter();

	for t in tokens_it.by_ref() {
		if let TokenTree::Ident(i) = t {
			match i.to_string().as_str() {
				"const" => return Err(Error::new("Const functions can't be a capture scope.", t.span())),
				// Could be supported when fibers are implemented. Then, we can
				// just generate a fiber-zone or whatever.
				"async" => return Err(Error::new("Async functions can't be a capture scope, yet.", t.span())),
				"fn"    => break,
				_       => continue,
			}
		}
	}

	// Here, either iterator is empty now or we've just consumed the
	// `fn` and ready to get the function name.

	let Some(TokenTree::Ident(i)) = tokens_it.next() else {
		let span = tokens.first().unwrap().span();
		return Err(Error::new("Only functions can be a capture scope.", span));
	};

	let name = i.to_string();
	// r# is only important for the rustc, Tracy zone name can be
	// whatever.
	let name = name.strip_prefix("r#").unwrap_or(&name);

	// The function body should be the last token tree.
	let body = match tokens.pop() {
		Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g,
		Some(t) => return Err(Error::new("Function without a body can't be a capture scope.", t.span())),
		_ => unreachable!(),
	};

	let augmented_body = vec![
			make_start_capture(),
			// This should strictly go *after* the capture start,
			// behaviour is undefined, otherwise.
			make_zone(name),
			body.stream(),
		]
		.into_iter()
		.collect();
	tokens.push(TokenTree::Group(Group::new(Delimiter::Brace, augmented_body)));

	Ok(TokenStream::from_iter(tokens))
}

/// Instruments a function to create and enter a zone every time the
/// function is called.
///
/// The generated zone's name will be the name of the function.
///
/// ## Examples
///
/// ```
/// # use tracy_gizmos_attributes::instrument;
/// #[instrument]
/// fn work() {
///    // do stuff
/// }
/// ```
///
/// ### Zone customization
///
/// The generated zone's name could be prefixed:
///
/// ```
/// # use tracy_gizmos_attributes::instrument;
/// #[instrument("Heavy")]
/// fn work() {
///    // will contain a zone named "Heavy::work"
/// }
/// ```
///
/// ### Unsupported cases
///
/// `const fn` cannot be instrumented, and will result in a compilation
/// failure:
///
/// ```compile_fail
/// # use tracy_gizmos_attributes::instrument;
/// #[instrument]
/// const fn work() {
///    // do stuff
/// }
/// ```
///
/// `async fn` cannot be instrumented, *yet*, and will result in a
/// compilation failure:
///
/// ```compile_fail
/// # use tracy_gizmos_attributes::instrument
/// #[instrument]
/// async fn work() {
///    // do stuff
/// }
/// ```
#[proc_macro_attribute]
pub fn instrument(attr: TokenStream, item: TokenStream) -> TokenStream {
	// Cloning a `TokenStream` is cheap since it's reference counted
	// internally.
	let instrumented = try_instrument(attr, item.clone());
	// We chain both error and original item, to prevent the
	// generation of two compilation errors: one from us and another
	// one (or multiple, even) caused by original item being skipped.
	match instrumented {
		Ok(item) => item,
		Err(e)   => TokenStream::from_iter(e.to_compile_error().into_iter().chain(item)),
	}
}

fn try_instrument(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
	// Function item's grammar:
	// https://doc.rust-lang.org/reference/items/functions.html
	// Put simply, it boils down to:
	// ... const? async? fn $name:ident ... {}?

	let prefix = if let Some(TokenTree::Literal(s)) = attr.into_iter().next() {
		Some(s.to_string())
	} else {
		None
	};
	let prefix = prefix.as_ref().and_then(|p| try_parse_str_literal(p));

	let mut tokens: Vec<TokenTree> = item.into_iter().collect();
	let mut tokens_it              = tokens.iter();

	for t in tokens_it.by_ref() {
		if let TokenTree::Ident(i) = t {
			match i.to_string().as_str() {
				"const" => return Err(Error::new("Const functions can't be instrumented.", t.span())),
				// Could be supported when fibers are implemented. Then, we can
				// just generate a fiber-zone or whatever.
				"async" => return Err(Error::new("Async functions can't be instrumented, yet.", t.span())),
				"fn"    => break,
				_       => continue,
			}
		}
	}

	// Here, either iterator is empty now or we've just consumed the
	// `fn` and ready to get the function name.

	let Some(TokenTree::Ident(i)) = tokens_it.next() else {
		let span = tokens.first().unwrap().span();
		return Err(Error::new("Only functions can be instrumented.", span));
	};

	let name = i.to_string();
	// r# is only important for the rustc, Tracy zone name can be
	// whatever.
	let name = name.strip_prefix("r#").unwrap_or(&name);

	let prefixed_name = prefix.map(|p| format!("{p}::{name}"));
	let name = if let Some(ref name) = prefixed_name {
		name
	} else {
		name
	};

	// The function body should be the last token tree.
	let body = match tokens.pop() {
		Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g,
		Some(t) => return Err(Error::new("Function without a body can't be instrumented.", t.span())),
		_ => unreachable!(),
	};

	let instrumented_body = vec![make_zone(name), body.stream()]
		.into_iter()
		.collect();
	tokens.push(TokenTree::Group(Group::new(Delimiter::Brace, instrumented_body)));

	Ok(TokenStream::from_iter(tokens))
}

fn try_parse_str_literal(s: &str) -> Option<&str> {
	let s = s.as_bytes();
	if s.len() >= 2 && s[0] == b'"' {
		// SAFETY: Source of bytes is always a correct utf-8 string.
		Some(unsafe { std::str::from_utf8_unchecked(&s[1..s.len() - 1]) })
	} else {
		None
	}
}

// let _tracy = tracy_gizmos::start_capture();
fn make_start_capture() -> TokenStream {
	TokenStream::from_iter([
		TokenTree::Ident(Ident::new("let",    Span::call_site())),
		TokenTree::Ident(Ident::new("_tracy", Span::mixed_site())),
		TokenTree::Punct(Punct::new('=', Spacing::Alone)),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("tracy_gizmos", Span::call_site())),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("start_capture", Span::call_site())),
		TokenTree::Group(
			Group::new(
				Delimiter::Parenthesis,
				TokenStream::new(),
			)
		),
		TokenTree::Punct(Punct::new(';', Spacing::Alone)),
	])
}

// ::tracy_gizmos::zone!($text);
fn make_zone(name: &str) -> TokenStream {
	TokenStream::from_iter([
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("tracy_gizmos", Span::call_site())),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("zone", Span::call_site())),
		TokenTree::Punct(Punct::new('!', Spacing::Alone)),
		TokenTree::Group(
			Group::new(
				Delimiter::Parenthesis,
				TokenStream::from_iter([
					TokenTree::Literal(Literal::string(name)),
				])
			)
		),
		TokenTree::Punct(Punct::new(';', Spacing::Alone)),
	])
}

struct Error {
	text:  &'static str,
	start: Span,
	end:   Span,
}

impl Error {
	fn new(text: &'static str, s: Span) -> Self {
		Self { text, start: s, end: s }
	}

	fn to_compile_error(&self) -> TokenStream {
		fn punct(c: char, s: Spacing, span: Span) -> TokenTree {
			TokenTree::Punct({
				let mut p = Punct::new(c, s);
				p.set_span(span);
				p
			})
		}

		TokenStream::from_iter([
			punct(':', Spacing::Joint, self.start),
			punct(':', Spacing::Alone, self.start),
			TokenTree::Ident(Ident::new("core", self.start)),
			punct(':', Spacing::Joint, self.start),
			punct(':', Spacing::Alone, self.start),
			TokenTree::Ident(Ident::new("compile_error", self.start)),
			punct('!', Spacing::Alone, self.start),
			TokenTree::Group({
				let mut g = Group::new(
					Delimiter::Brace,
					TokenStream::from_iter([
						TokenTree::Literal({
							let mut s = Literal::string(self.text);
							s.set_span(self.end);
							s
						})
					])
				);
				g.set_span(self.end);
				g
			}),
		])
	}
}
